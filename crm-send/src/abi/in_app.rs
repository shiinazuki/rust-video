use tonic::Status;
use tracing::warn;

use crate::{
    NotificationService,
    pb::{InAppMessage, SendRequest, SendResponse, send_request::Msg},
};

use super::{Sender, to_ts};

impl Sender for InAppMessage {
    async fn send(self, svc: NotificationService) -> Result<SendResponse, Status> {
        let message_id = self.message_id.clone();
        let _ = svc.sender.send(Msg::InApp(self)).await.map_err(|e| {
            warn!("Failed to send inapp message: {:?}", e);
            Status::internal("Failed to send inapp message")
        });
        Ok(SendResponse {
            message_id,
            timestamp: Some(to_ts()),
        })
    }
}

impl From<InAppMessage> for Msg {
    fn from(value: InAppMessage) -> Self {
        Msg::InApp(value)
    }
}

impl From<InAppMessage> for SendRequest {
    fn from(value: InAppMessage) -> Self {
        let msg: Msg = value.into();
        SendRequest { msg: Some(msg) }
    }
}

#[cfg(feature = "test_utils")]
impl InAppMessage {
    pub fn fake() -> Self {
        use uuid::Uuid;
        InAppMessage {
            message_id: Uuid::new_v4().to_string(),
            device_id: Uuid::new_v4().to_string(),
            title: "Hello".to_string(),
            body: "Hello, world!".to_string(),
        }
    }
}
