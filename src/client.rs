use aws_sdk_sqs::{
    error::{DeleteMessageBatchError, ReceiveMessageError, SendMessageBatchError},
    model::{DeleteMessageBatchRequestEntry, Message, SendMessageBatchRequestEntry},
    output::{DeleteMessageBatchOutput, SendMessageBatchOutput},
    Client, SdkError,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Event {
    pub text: String,
}

impl From<&Event> for Option<SendMessageBatchRequestEntry> {
    fn from(event: &Event) -> Self {
        let body = serde_json::to_string(&event).ok()?;
        let entry = SendMessageBatchRequestEntry::builder()
            .message_body(body)
            .id(Uuid::new_v4().to_string())
            .build();
        Some(entry)
    }
}

#[derive(Debug)]
pub struct SqsEventClient {
    client: Client,
    url: String,
}

impl SqsEventClient {
    pub fn new(client: Client, url: String) -> Self {
        SqsEventClient { client, url }
    }
}

impl SqsEventClient {
    pub async fn send(
        &self,
        events: &[Event],
    ) -> Result<SendMessageBatchOutput, SdkError<SendMessageBatchError>> {
        let entries = events.iter().filter_map(|event| event.into()).collect();
        self.client
            .send_message_batch()
            .queue_url(&self.url)
            .set_entries(Some(entries))
            .send()
            .await
    }

    pub async fn receive(&self) -> Result<Vec<Message>, SdkError<ReceiveMessageError>> {
        self.client
            .receive_message()
            .max_number_of_messages(10)
            .wait_time_seconds(20)
            .queue_url(&self.url)
            .send()
            .await
            .map(|output| output.messages.unwrap_or_default())
    }

    pub async fn delete(
        &self,
        messages: &[Message],
    ) -> Result<DeleteMessageBatchOutput, SdkError<DeleteMessageBatchError>> {
        let entries = messages
            .iter()
            .filter_map(|message| match message.receipt_handle {
                Some(ref receipt_handle) => {
                    let req = DeleteMessageBatchRequestEntry::builder()
                        .receipt_handle(receipt_handle)
                        .id(message
                            .message_id()
                            .map(String::from)
                            .unwrap_or_else(|| Uuid::new_v4().to_string()))
                        .build();
                    Some(req)
                }
                None => None,
            })
            .collect();
        self.client
            .delete_message_batch()
            .queue_url(&self.url)
            .set_entries(Some(entries))
            .send()
            .await
    }
}
