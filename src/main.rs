use aws_sdk_sqs::{model::DeleteMessageBatchRequestEntry, Client, Endpoint};
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
        .message_body("Hello, world! 3")
        .send()
        .await?;
    println!("Response from sending a message: {response:#?}");

    let rcv_message_output = client
        .receive_message()
        .max_number_of_messages(10)
        .queue_url("http://localhost:4566/queue/demo-event-stream")
        .send()
        .await?;
    for message in &rcv_message_output.messages {
        println!("Got the message: {message:#?}");
    }

    let entries_to_delete = rcv_message_output
        .messages
        .unwrap_or_default()
        .iter()
        .filter_map(|message| match message.receipt_handle() {
            Some(receipt_handle) => {
                let req = DeleteMessageBatchRequestEntry::builder()
                    .receipt_handle(receipt_handle)
                    .id(message.message_id().unwrap_or_default())
                    .build();
                Some(req)
            }
            None => None,
        })
        .collect();

    let delete_message_output = client
        .delete_message_batch()
        .queue_url("http://localhost:4566/queue/demo-event-stream")
        .set_entries(Some(entries_to_delete))
        .send()
        .await?;

    println!("{delete_message_output:#?}");
    Ok(())
}
