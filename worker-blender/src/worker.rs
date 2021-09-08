use std::{
    convert::{TryFrom, TryInto},
    process::{Command, Stdio},
};

use log::{debug, error, info, trace, warn};

use fabseal_micro_common::*;

use rand::Rng;
use redis::{
    streams::{StreamId, StreamInfoGroupsReply, StreamReadOptions, StreamReadReply},
    Commands, Value,
};

use color_eyre::eyre::Result;

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

mod file_context;
use crate::worker::file_context::CommandFileContext;
mod util;
use crate::worker::util::WorkerContext;

use crate::Settings;

pub(crate) struct Worker {
    settings: Settings,
    conn: redis::Connection,
    ctx: WorkerContext,
    should_exit: Arc<AtomicBool>,
    consumer_name: String,
    read_options: StreamReadOptions,
}

impl Drop for Worker {
    fn drop(&mut self) {
        let res1: redis::RedisResult<Value> = self.conn.xgroup_delconsumer(
            FABSEAL_SUBMISSION_QUEUE,
            FABSEAL_SUBMISSION_CONSUMER_GROUP,
            &self.consumer_name,
        );
        match res1 {
            Ok(redis::Value::Int(_pending_messages)) => {}
            Ok(resp) => {
                warn!(
                    "Unexpected Redis response while deleting consumer: {:?}",
                    resp
                );
            }
            Err(e) => {
                error!("Error while dropping Worker: {}", e);
            }
        }

        // Don't call XGROUP DESTROY, since other workers might be active
    }
}

impl Worker {
    pub(crate) fn create(settings: Settings) -> Result<Worker> {
        let ctx = WorkerContext::from_dmstl_dir(&settings.dmstl_directory)?;

        let redis_addr = format!("redis://{}/", settings.redis.address);

        Self::create_from(settings, ctx, &redis_addr)
    }

    fn setup_stream(redis_conn: &mut redis::Connection) -> redis::RedisResult<()> {
        let group_exists: bool =
            match redis_conn.xinfo_groups::<_, StreamInfoGroupsReply>(FABSEAL_SUBMISSION_QUEUE) {
                Ok(reply) => reply
                    .groups
                    .iter()
                    .any(|g| g.name == FABSEAL_SUBMISSION_CONSUMER_GROUP),
                Err(e) => {
                    warn!(
                        "Error while fetching stream information (likely harmless): {}",
                        e
                    );

                    // Assume the stream does not exist yet
                    false
                }
            };

        if group_exists {
            return Ok(());
        }

        // NOTE: Possible race condition here (someone could have created the consumer group in the meantime)
        // But that's not a big issue, since we ignore consumer group creation errors anyway
        // (The redis crate does not allow differentiating between BUSYGROUP and other errors)

        let res1 = redis_conn.xgroup_create_mkstream(
            FABSEAL_SUBMISSION_QUEUE,
            FABSEAL_SUBMISSION_CONSUMER_GROUP,
            "$",
        );
        match res1 {
            Ok(redis::Value::Okay) => {
                // Everything fine
                Ok(())
            }
            Ok(resp) => {
                // Possibly BUSYGROUP (or another error)
                warn!(
                    "Unexpected Redis response (possibly BUSYGROUP?), ignoring: {:?}",
                    resp
                );
                Ok(())
            }
            Err(rerr) => {
                error!("redis error: {:?}", rerr);
                Err(rerr)
            }
        }
    }

    fn consumer_name() -> String {
        let id: u32 = rand::thread_rng().gen();

        format!("fs_consumer_{:08X}", id)
    }

    fn setup_exit_flag() -> Result<Arc<AtomicBool>> {
        let should_exit = Arc::new(AtomicBool::new(false));
        signal_hook::flag::register(signal_hook::consts::SIGTERM, Arc::clone(&should_exit))?;
        signal_hook::flag::register(signal_hook::consts::SIGINT, Arc::clone(&should_exit))?;
        Ok(should_exit)
    }

    fn create_from(settings: Settings, ctx: WorkerContext, redis_addr: &str) -> Result<Worker> {
        let client = redis::Client::open(redis_addr)?;
        let conn = {
            let mut conn = client.get_connection()?;
            Self::setup_stream(&mut conn)?;
            conn
        };

        let should_exit = Self::setup_exit_flag()?;

        let consumer_name = Self::consumer_name();

        const READ_TIMEOUT: usize = 1000;
        let read_options = StreamReadOptions::default()
            .count(1)
            .block(READ_TIMEOUT)
            .group(FABSEAL_SUBMISSION_CONSUMER_GROUP, &consumer_name);

        Ok(Worker {
            settings,
            conn,
            ctx,
            should_exit,
            read_options,
            consumer_name,
        })
    }

    fn read_stream(&mut self) -> Result<StreamReadReply> {
        let results: StreamReadReply =
            self.conn
                .xread_options(&[FABSEAL_SUBMISSION_QUEUE], &[">"], &self.read_options)?;
        Ok(results)
    }

    pub(crate) fn run(&mut self) {
        while !self.should_exit.load(Ordering::Relaxed) {
            match self.read_stream() {
                Ok(results) => {
                    for sk in results.keys {
                        debug_assert!(sk.key == FABSEAL_SUBMISSION_QUEUE);
                        for msg in sk.ids {
                            self.handle_stream_msg(msg);
                        }
                    }
                }
                Err(e) => {
                    error!("redis error, ignoring message: {:?}", e);
                }
            }
        }

        info!("received signal, shutting down");
    }

    fn send_ack(&mut self, msg: StreamId) {
        debug!("ack-ing message");

        let resp: Value = self
            .conn
            .xack(
                FABSEAL_SUBMISSION_QUEUE,
                FABSEAL_SUBMISSION_CONSUMER_GROUP,
                &[msg.id],
            )
            .unwrap();
        debug!("resp: {:?}", resp);

        debug_assert!(resp == redis::Value::Int(1));
        if (resp != redis::Value::Int(1)) {
            error!("error while sending XACK, response: {:?}", resp);
        }
    }


    fn handle_stream_msg(&mut self, msg: StreamId) {
        trace!("msg={:?}", msg);

        let request_id: RequestId = match msg.map.get("request_id").unwrap() {
            Value::Data(v) => {
                debug_assert_eq!(v.len(), 4);
                RequestId::try_from(v.as_slice()).unwrap()
            }
            _ => {
                panic!("panic");
            }
        };
        let image_data: Vec<u8> = match self.conn.get(image_key(request_id)).unwrap() {
            Value::Data(v) => v,
            Value::Nil => {
                warn!("Tried reading image for request {}, but it is gone (probably expired)", request_id);

                self.send_ack(msg);
                return;
            },
            v => {
                error!("unexpected Redis response value: {:?}", v);
                panic!("panic");
            },
        };

        match self.try_handle(&image_data, request_id) {
            Ok(_) => {
                debug!("ack-ing message");

                self.send_ack(msg);
            }
            Err(e) => {
                error!("error while processing: {}", e);

                self.send_ack(msg);
            }
        };
    }

    fn try_handle(&mut self, payload: &[u8], request_id: RequestId) -> Result<()> {
        let fctx = CommandFileContext::create(payload)?;

        let mut comm = Command::new("blender");

        comm.stdin(Stdio::null())
            .stdout(Stdio::piped())
            .arg("--background")
            .arg(self.ctx.blend_path.as_os_str())
            .arg("--python")
            .arg(self.ctx.python_path.as_os_str())
            .arg("--log-level")
            .arg("0")
            .arg("--")
            .arg(fctx.input_file_path())
            .arg(fctx.output_file_path());

        debug!("the command: {:?}", comm);

        let status = comm.status()?;
        if !status.success() {
            trace!("explicitly closing temporary files");
            drop(fctx);
            color_eyre::eyre::bail!("Blender command failed");
        }

        let result_data = fctx.finish()?;

        let key = result_key(request_id);
        trace!("setting key={}", key);
        let _: () = self.conn.set_ex(
            key,
            result_data,
            self.settings.limits.result_ttl.try_into().unwrap(),
        )?;

        Ok(())
    }
}
