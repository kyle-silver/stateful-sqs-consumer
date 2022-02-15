use std::sync::Arc;

use aws_sdk_sqs::{Client, Endpoint};
use http::Uri;

mod client;

use client::{Event, SqsEventClient};

async fn localstack_client() -> Client {
    let config = aws_config::from_env().load().await;
    let uri = Uri::from_static("http://localhost:4566/");
    let endpoint = Endpoint::immutable(uri);
    let config = aws_sdk_sqs::config::Builder::from(&config)
        .endpoint_resolver(endpoint)
        .build();
    Client::from_conf(config)
}

#[tokio::main]
async fn main() -> Result<(), aws_sdk_sqs::Error> {
    let client = SqsEventClient::new(
        localstack_client().await,
        "http://localhost:4566/queue/demo-event-stream".to_string(),
    );
    let response = client
        .send(&vec![Event {
            text: "Hello, world!".to_string(),
        }])
        .await?;
    println!("Response from sending a message: {response:#?}");

    let client = Arc::new(client);

    let mut polling_handles = Vec::new();

    for id in 0..10 {
        let client = client.clone();
        let handle = tokio::spawn(async move {
            loop {
                println!("[{id}] Receiving messages...");
                let messages = client.receive().await.unwrap();
                for message in &messages {
                    let event: Event = serde_json::from_str(message.body().unwrap()).unwrap();
                    println!("[{id}] Got the message: {event:#?}");
                }
                let delete_message_output = client.delete(&messages).await.unwrap();
                println!("[{id}] {delete_message_output:#?}");
            }
        });
        polling_handles.push(handle);
    }

    // let handle = tokio::spawn(async move {
    //     loop {
    //         println!("Receiving messages...");
    //         let messages = client.receive().await.unwrap();
    //         for message in &messages {
    //             let event: Event = serde_json::from_str(message.body().unwrap()).unwrap();
    //             println!("Got the message: {event:#?}");
    //         }
    //         let delete_message_output = client.delete(&messages).await.unwrap();
    //         println!("{delete_message_output:#?}");
    //     }
    // });

    let _ = futures::future::join_all(polling_handles).await;

    // let _ = tokio::join!(handle);

    Ok(())
}
