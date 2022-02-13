use aws_sdk_sqs::{Client, Endpoint};
use http::Uri;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Event {
    text: String,
}

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
    let client = localstack_client().await;
    let response = client
        .send_message()
        .queue_url("http://localhost:4566/queue/demo-event-stream")
        .message_body("Hello, world! 2")
        .send()
        .await?;
    println!("Response from sending a message: {response:#?}");

    let rcv_message_output = client
        // .receive_message_b()
        .receive_message()
        .max_number_of_messages(10)
        .queue_url("http://localhost:4566/queue/demo-event-stream")
        .send()
        .await?;
    for message in rcv_message_output.messages.unwrap_or_default() {
        println!("Got the message: {message:#?}");
    }
    Ok(())
}
