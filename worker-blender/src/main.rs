use futures::{StreamExt};

use std::process::Command;

use lapin::{
    options::*, types::FieldTable, Connection,
    ConnectionProperties,
};
use log::{debug, info};

use fabseal_micro_common::*;

use tempfile::NamedTempFile;

use std::io::{Read, Write};

use std::path::PathBuf;

use redis::Commands;

use anyhow::Result;

mod settings;
use settings::Settings;

struct Context {
    blend_path: PathBuf,
    python_path: PathBuf,
}
impl Context {
    fn from_dmstl_dir(dmstl_path: &str) -> Context {
        let dmstl: PathBuf  = PathBuf::from(dmstl_path).canonicalize().unwrap();

        let blend_path: PathBuf = {
            let mut p = dmstl.clone();
            p.push(r"src/empty.blend");
            p
        };
        let python_path: PathBuf = {
            let mut p = dmstl;
            p.push(r"src/displacementMapToStl.py");
            p
        };

        Context {
            blend_path,
            python_path
        }
    }
}

async fn try_handle(
    payload: &ImageRequest,
    conn: &mut redis::Connection,
    ctx: &Context
) -> Result<()>
{
    let mut input_file = NamedTempFile::new()?;
    let mut output_file = NamedTempFile::new()?;

    debug!("input temp file: {:?}", input_file);
    debug!("output temp file: {:?}", output_file);

    input_file.write_all(&payload.image.image)?;

    let mut comm = Command::new("blender");
    comm
            .arg("--threads").arg("4")
            .arg("--background")
            .arg(ctx.blend_path.as_os_str())
            .arg("--python")
            .arg(ctx.python_path.as_os_str())
            .arg("--")
            .arg(input_file.path())
            .arg(output_file.path());

    debug!("the command: {:?}", comm);

    let _status = comm.status()?;

    let mut result_data: Vec<u8> = Vec::new();
    output_file.as_file_mut().read_to_end(&mut result_data)?;

    let key = result_key(payload.request_id);
    debug!("setting key={}", key);
    let _ : () = conn.set_ex(key, result_data, RESULT_EXPIRATION_SECONDS)?;

    debug!("closing temporary files");
    input_file.close()?;
    output_file.close()?;

    Ok(())
}

async fn handle_work_item(
    delivery: lapin::message::Delivery,
    conn: &mut redis::Connection,
    ctx: &Context
)
{
    let payload: ImageRequest = serde_json::from_slice(&delivery.data).unwrap();
    info!("payload={:?}", payload.image.image_type);

    match try_handle(&payload, conn, ctx).await {
        Ok(_) => {
            debug!("ack-ing message");

            delivery
                .ack(BasicAckOptions::default())
                .await
                .expect("ack failed");
        },
        Err(e) => {
            info!("error: {}", e);
            debug!("nack-ing message");

            delivery
                .nack(BasicNackOptions::default())
                .await
                .expect("nack failed");
        }
    }

}

fn run() {
    let settings = Settings::new().unwrap();
    debug!("settings: {:?}", settings);

    let dmstl_env = std::env::var("DMSTL_DIR").expect("Specify displacementMapToStl directory with DMSTL_DIR");
    let ctx = Context::from_dmstl_dir(dmstl_env.as_str());

    info!("will launch");
    let rmq_addr = std::env::var("AMQP_ADDR").unwrap_or_else(|_| "amqp://127.0.0.1:5672/%2f".into());
    let redis_addr = std::env::var("REDIS_ADDR").unwrap_or_else(|_| "redis://127.0.0.1/".into());

    async_global_executor::block_on(async {
        let conn = Connection::connect(
            &rmq_addr,
            ConnectionProperties::default(),
        )
        .await.unwrap();

        let client = redis::Client::open(redis_addr).unwrap();
        let mut redis_conn = client.get_connection().unwrap();

        let channel_b = conn.create_channel().await.unwrap();

        info!("CONNECTED");
        let mut consumer = channel_b
            .basic_consume(
                FABSEAL_QUEUE,
                "worker_blender",
                BasicConsumeOptions::default(),
                FieldTable::default(),
            )
            .await.unwrap();


        async_global_executor::spawn(async move {
            info!("will consume");
            while let Some(delivery) = consumer.next().await {
                info!("received delivery");
                let (_, delivery) = delivery.expect("error in consumer");
                handle_work_item(delivery, &mut redis_conn, &ctx).await;
            }
        }).await;

    });
}

// #[tokio::main]
fn main() {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("debug"));

    run();
}