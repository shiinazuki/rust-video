use tonic::Status;
use tracing::warn;

use crate::{
    NotificationService,
    pb::{SendRequest, SendResponse, SmsMessage, send_request::Msg},
};

use super::{Sender, to_ts};

impl Sender for SmsMessage {
    async fn send(self, svc: NotificationService) -> Result<SendResponse, Status> {
        let message_id = self.message_id.clone();
        let _ = svc.sender.send(Msg::Sms(self)).await.map_err(|e| {
            warn!("Failed to send sms message: {:?}", e);
            Status::internal("Failed to send sms message")
        });
        Ok(SendResponse {
            message_id,
            timestamp: Some(to_ts()),
        })
    }
}

impl From<SmsMessage> for Msg {
    fn from(value: SmsMessage) -> Self {
        Msg::Sms(value)
    }
}

impl From<SmsMessage> for SendRequest {
    fn from(value: SmsMessage) -> Self {
        let msg: Msg = value.into();
        SendRequest { msg: Some(msg) }
    }
}

#[cfg(test)]
impl SmsMessage {
    pub fn fake() -> Self {
        use fake::Fake;
        use uuid::Uuid;
        SmsMessage {
            message_id: Uuid::new_v4().to_string(),
            sender: fake::faker::phone_number::en::PhoneNumber().fake(),
            recipients: vec![fake::faker::phone_number::en::PhoneNumber().fake()],
            body: "Hello, world".to_string(),
        }
    }
}
