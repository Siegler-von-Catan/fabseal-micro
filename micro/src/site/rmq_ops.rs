use lapin::{Result as LAResult, protocol::basic::AMQPProperties};
use amq_protocol_types::{FieldTable};

use log::info;

use fabseal_micro_common::{FABSEAL_EXCHANGE, FABSEAL_QUEUE};

pub(crate) async fn rmq_declare(
    pool: bb8::Pool<bb8_lapin::LapinConnectionManager>
) -> LAResult<()> {
    let conn = pool.get().await.unwrap();
    let channel = conn.create_channel()
        .await
        .unwrap();

    info!("CONNECTED");

    let queue = channel
        .queue_declare(
            FABSEAL_QUEUE,
            lapin::options::QueueDeclareOptions::default(),
            FieldTable::default(),
        )
        .await?;

    info!("Declared queue {:?}", queue);

    Ok(())
}


lazy_static::lazy_static! {
    static ref PROPS: AMQPProperties = {
        lapin::BasicProperties::default()
            .with_content_type("application/json".into())
    };
    static ref PUBLISH_OPTIONS: lapin::options::BasicPublishOptions = {
        lapin::options::BasicPublishOptions::default()
    };
}

pub(crate) async fn rmq_publish(
    conn: bb8::PooledConnection<'_, bb8_lapin::LapinConnectionManager>,
    payload: Vec<u8>,
) -> LAResult<()> {
    let channel = conn.create_channel()
        .await
        .unwrap();

    let confirm = channel.basic_publish(
            FABSEAL_EXCHANGE,
            FABSEAL_QUEUE,
            *PUBLISH_OPTIONS,
            payload,
            PROPS.clone()
        )
        .await?
        .await?;

    debug_assert_eq!(confirm, lapin::publisher_confirm::Confirmation::NotRequested);


    Ok(())
}