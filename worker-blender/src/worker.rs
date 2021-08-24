use std::convert::{TryFrom};
use std::process::{Command, Stdio};

use log::{debug, error, info, trace};

use fabseal_micro_common::*;

use redis::{Commands, Value, streams::{StreamId, StreamReadOptions, StreamReadReply}};

use anyhow::Result;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

mod file_context;
use crate::worker::file_context::CommandFileContext;
mod util;
use crate::worker::util::Context;

pub(crate) struct Worker {
    conn: redis::Connection,
    ctx: Context,
    should_exit: Arc<AtomicBool>
}

impl Drop for Worker {
    fn drop(&mut self) {
        let res1: Result<Value, redis::RedisError> = self.conn.xgroup_destroy(
            FABSEAL_SUBMISSION_QUEUE,
            FABSEAL_SUBMISSION_CONSUMER_GROUP);
        match res1 {
            Ok(_) => {},
            Err(e) => {
                error!("Error while dropping Worker: {}", e);
            }
        }
    }
}

impl Worker {
    pub(crate) fn create() -> Result<Worker> {
        let dmstl_env = std::env::var("DMSTL_DIR").expect("Specify displacementMapToStl directory with DMSTL_DIR");
        let ctx = Context::from_dmstl_dir(dmstl_env.as_str());

        info!("will launch");
        let redis_addr = std::env::var("REDIS_ADDR").unwrap_or_else(|_| "redis://127.0.0.1/".into());

        let client = redis::Client::open(redis_addr)?;
        let mut redis_conn = client.get_connection()?;

        let res1 = redis_conn.xgroup_create_mkstream(
            FABSEAL_SUBMISSION_QUEUE,
            FABSEAL_SUBMISSION_CONSUMER_GROUP,
            "$"
            );
        match res1 {
            Ok(redis::Value::Okay) => {},
            Ok(_) => {},
            Err(rerr) => {
                error!("redis error: {:?}", rerr);
            },
        }
        // assert!(res1 == redis::Value::Okay);

        let should_exit = Arc::new(AtomicBool::new(false));
        signal_hook::flag::register(signal_hook::consts::SIGTERM, Arc::clone(&should_exit))?;
        signal_hook::flag::register(signal_hook::consts::SIGINT, Arc::clone(&should_exit))?;

        Ok(Worker {
            conn: redis_conn,
            ctx,
            should_exit
        })
    }

    pub(crate) fn run(&mut self) {
        while !self.should_exit.load(Ordering::Relaxed) {
            const READ_TIMEOUT: usize = 1000;
            let opts = StreamReadOptions::default()
                .count(1)
                .block(READ_TIMEOUT)
                .group(FABSEAL_SUBMISSION_CONSUMER_GROUP, "fixme_consumer_1");
            let results: StreamReadReply =
                self.conn.xread_options(
                    &[FABSEAL_SUBMISSION_QUEUE],
                    &[">"],
                    opts).unwrap();
            for sk in results.keys {
                debug_assert!(sk.key == FABSEAL_SUBMISSION_QUEUE);
                for msg in sk.ids {
                    self.handle_stream_msg(msg);
                }
            }
        }
        info!("received signal, shutting down");
    }

    fn handle_stream_msg(
        &mut self,
        msg: StreamId
    )
    {
        debug!("msg={:?}", msg);

        let request_id: RequestId = match msg.map.get("request_id").unwrap() {
            Value::Data(v) => {
                debug_assert_eq!(v.len(), 4);
                RequestId::try_from(v.as_slice()).unwrap()
            },
            _ => {
                panic!("panic");
            }
        };
        let image_data: Vec<u8> = match self.conn.get(image_key(request_id)).unwrap() {
            Value::Data(v) => {
                v
            },
            v => {
                error!("unexpected Redis response value: {:?}", v);
                panic!("panic");
            }
        };

        match self.try_handle(&image_data, request_id) {
            Ok(_) => {
                debug!("ack-ing message");

                let res2: Value = self.conn.xack(
                    FABSEAL_SUBMISSION_QUEUE,
                    FABSEAL_SUBMISSION_CONSUMER_GROUP,
                    &[msg.id]
                ).unwrap();
                debug!("resp: {:?}", res2);

                assert!(res2 == redis::Value::Int(1));
            },
            Err(e) => {
                error!("error while processing: {}", e);

                let res2: Value = self.conn.xack(
                    FABSEAL_SUBMISSION_QUEUE,
                    FABSEAL_SUBMISSION_CONSUMER_GROUP,
                    &[msg.id]
                ).unwrap();
                assert!(res2 == redis::Value::Okay);
            }
        };
    }

    fn try_handle(
        &mut self,
        payload: &[u8],
        request_id: RequestId
    ) -> Result<()>
    {
        let fctx = CommandFileContext::create(payload)?;

        let mut comm = Command::new("/usr/bin/blender");

        comm
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            // .arg("--threads").arg("1")
            .arg("--background")
            .arg(self.ctx.blend_path.as_os_str())
            // .arg("src/empty.blend")
            .arg("--python")
            // .arg("src/displacementMapToStl.py")
            .arg(self.ctx.python_path.as_os_str())
            .arg("--log-level").arg("0")
            // .arg("--debug")
            // .arg("--verbose").arg("0")
            .arg("--")
            .arg(fctx.input_file_path())
            .arg(fctx.output_file_path());

        debug!("the command: {:?}", comm);

        let status = comm.status()?;
        if !status.success() {
            trace!("explicitly closing temporary files");
            drop(fctx);
            anyhow::bail!("Blender command failed");
        }

        let result_data = fctx.finish()?;

        let key = result_key(request_id);
        trace!("setting key={}", key);
        let _ : () = self.conn.set_ex(key, result_data, RESULT_EXPIRATION_SECONDS)?;

        Ok(())
    }
}
