// use futures_util::stream::stream::StreamExt;
use futures::{StreamExt};

use bb8_lapin::prelude::*;

use serde::{Deserialize, Serialize};

use lapin::{
    options::*, types::FieldTable, Connection,
    ConnectionProperties, Result,
};
use log::info;

#[derive(Serialize, Deserialize, Debug)]
struct BBReq {
    request_id: i32,
    image_id: i32,
}

const FABSEAL_QUEUE: &'static str = "fabseal";

async fn handle_work_item(delivery: lapin::message::Delivery)
{
    let payload: BBReq = serde_json::from_slice(&delivery.data).unwrap();
    info!("payload={:?}", payload);

    delivery
        .ack(BasicAckOptions::default())
        .await
        .expect("ack");

}

async fn run_conn(
	conn: &Connection
) -> Result<()> {
    let channel_a = conn.create_channel().await?;
    let channel_b = conn.create_channel().await?;

    let queue = channel_a
        .queue_declare(
            FABSEAL_QUEUE,
            QueueDeclareOptions::default(),
            FieldTable::default(),
        )
        .await?;

    info!("Declared queue {:?}", queue);

    let mut consumer = channel_b
        .basic_consume(
            FABSEAL_QUEUE,
            "worker_blender",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await?;


    tokio::spawn(async move {
        info!("will consume");
        while let Some(delivery) = consumer.next().await {
            info!("received delivery");
            let (_, delivery) = delivery.expect("error in consumer");
            handle_work_item(delivery).await;
        }
    }).await.unwrap();

    Ok(())
}

async fn example() {
    info!("will launch");
    let addr = std::env::var("AMQP_ADDR").unwrap_or_else(|_| "amqp://127.0.0.1:5672/%2f".into());
    let manager = LapinConnectionManager::new(&addr, ConnectionProperties::default());
    let pool = bb8::Pool::builder()
        .max_size(15)
        .build(manager)
        .await
        .unwrap();
    for _ in 0..20 {
        let pool = pool.clone();
        tokio::spawn(async move {
            info!("will pool");
            let conn = pool.get().await.unwrap();
            // use the connection
            run_conn(&conn).await.unwrap();
            // it will be returned to the pool when it falls out of scope.
        });
    }
}

fn example2() {
    info!("will launch");
    let addr = std::env::var("AMQP_ADDR").unwrap_or_else(|_| "amqp://127.0.0.1:5672/%2f".into());

    async_global_executor::block_on(async {
        let conn = Connection::connect(
            &addr,
            ConnectionProperties::default(),
        )
        .await.unwrap();
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
                handle_work_item(delivery).await;
            }
        }).await;

    });
}

// #[tokio::main]
fn main() {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }

    env_logger::init();

    info!("will po1ol");
    example2();
    /*
    tokio::spawn(async {
        info!("will po2ol");
    	example().await
    }).await.unwrap();
    */
}