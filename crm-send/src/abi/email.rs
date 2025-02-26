use tonic::Status;
use tracing::warn;

use crate::{
    NotificationService,
    pb::{EmailMessage, SendRequest, SendResponse, send_request::Msg},
};

use super::{Sender, to_ts};
impl Sender for EmailMessage {
    async fn send(self, svc: NotificationService) -> Result<SendResponse, Status> {
        let message_id = self.message_id.clone();
        let _ = svc.sender.send(Msg::Email(self)).await.map_err(|e| {
            warn!("Failed to send email message: {:?}", e);
            Status::internal("Failed to send email message")
        });
        Ok(SendResponse {
            message_id,
            timestamp: Some(to_ts()),
        })
    }
}

impl From<EmailMessage> for Msg {
    fn from(value: EmailMessage) -> Self {
        Msg::Email(value)
    }
}

impl From<EmailMessage> for SendRequest {
    fn from(value: EmailMessage) -> Self {
        let msg: Msg = value.into();
        SendRequest { msg: Some(msg) }
    }
}

#[cfg(test)]
impl EmailMessage {
    pub fn fake() -> Self {
        use fake::{Fake, faker::internet::en::SafeEmail};
        use uuid::Uuid;
        EmailMessage {
            message_id: Uuid::new_v4().to_string(),
            subject: "Hello".to_string(),
            sender: SafeEmail().fake(),
            recipients: vec![SafeEmail().fake()],
            body: "Hello, world!".to_string(),
        }
    }
}
