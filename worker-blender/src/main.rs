// use futures_util::stream::stream::StreamExt;
use futures::future::Future;

use bb8_lapin::prelude::*;

use lapin::{
    options::*, publisher_confirm::Confirmation, types::FieldTable, BasicProperties, Connection,
    ConnectionProperties, Result,
};
use log::info;

async fn run_conn(
	conn: &Connection
) -> Result<()> {
    let channel_a = conn.create_channel().await?;
    let channel_b = conn.create_channel().await?;

    let queue = channel_a
        .queue_declare(
            "hello",
            QueueDeclareOptions::default(),
            FieldTable::default(),
        )
        .await?;

    info!("Declared queue {:?}", queue);

    let mut consumer = channel_b
        .basic_consume(
            "hello",
            "my_consumer",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await?;
    tokio::spawn(async move {
        info!("will consume");
        while let Some(delivery) = consumer.next().await {
            let (_, delivery) = delivery.expect("error in consumer");
            delivery
                .ack(BasicAckOptions::default())
                .await
                .expect("ack");
        }
    }).detach();

    let payload = b"Hello world!";

    loop {
        let confirm = channel_a
            .basic_publish(
                "",
                "hello",
                BasicPublishOptions::default(),
                payload.to_vec(),
                BasicProperties::default(),
            )
            .await?
            .await?;
        assert_eq!(confirm, Confirmation::NotRequested);
    }

    Ok(())
}

async fn example() {
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
            let conn = pool.get().await.unwrap();
            // use the connection
            run_conn(&conn).await;
            // it will be returned to the pool when it falls out of scope.
        });
    }
}

fn main() -> Result<()> {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }

    env_logger::init();

    tokio::spawn(async {
    	example().await
    });

    Ok(())
}