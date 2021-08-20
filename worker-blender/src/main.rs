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

async fn handle_work_item(
    delivery: lapin::message::Delivery,
    conn: &mut redis::Connection
)
{
    let payload: ImageRequest = serde_json::from_slice(&delivery.data).unwrap();
    info!("payload={:?}", payload.image.image_type);

    let dmstl_env = std::env::var("DMSTL_DIR").expect("Specify displacementMapToStl directory with DMSTL_DIR");
    let dmstl: PathBuf  = PathBuf::from(dmstl_env).canonicalize().unwrap();

    let mut input_file = NamedTempFile::new().unwrap();
    let mut output_file = NamedTempFile::new().unwrap();

    debug!("input temp file: {:?}", input_file);
    debug!("output temp file: {:?}", output_file);

    input_file.write_all(&payload.image.image).unwrap();

    let mut blend_path: PathBuf = dmstl.clone();
    blend_path.push(r"src/empty.blend");
    let mut python_path: PathBuf = dmstl.clone();
    python_path.push(r"src/displacementMapToStl.py");

    let mut comm = Command::new("blender");
    comm
            .arg("--threads").arg("4")
            .arg("--background")
            .arg(blend_path.as_os_str())
            .arg("--python")
            .arg(python_path.as_os_str())
            .arg("--")
            .arg(input_file.path())
            .arg(output_file.path());

    debug!("the command: {:?}", comm);

    let _status = comm
            .status()
            .expect("failed to execute process");

    let mut result_data: Vec<u8> = Vec::new();
    output_file.as_file_mut().read_to_end(&mut result_data).unwrap();

    let key = result_key(payload.request_id);
    debug!("setting key={}", key);
    let _ : () = conn.set_ex(key, result_data, RESULT_EXPIRATION_SECONDS).unwrap();

    debug!("closing temporary files");
    input_file.close().unwrap();
    output_file.close().unwrap();

    debug!("ack-ing message");

    delivery
        .ack(BasicAckOptions::default())
        .await
        .expect("ack failed");
}

fn run() {
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
                handle_work_item(delivery, &mut redis_conn).await;
            }
        }).await;

    });
}

// #[tokio::main]
fn main() {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("debug"));

    run();
}