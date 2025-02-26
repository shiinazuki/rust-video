mod abi;
mod configuration;
pub mod pb;
pub use configuration::{AppConfig, get_configuration};
use std::{pin::Pin, sync::Arc};
use tokio::sync::mpsc;

use futures::Stream;
use pb::{SendRequest, SendResponse, notification_server::Notification, send_request::Msg};
use tonic::{Request, Response, Status, Streaming, async_trait};

type ServiceResult<T> = Result<Response<T>, Status>;

type ResponseStream = Pin<Box<dyn Stream<Item = Result<SendResponse, Status>> + Send>>;

#[async_trait]
impl Notification for NotificationService {
    type SendStream = ResponseStream;

    async fn send(
        &self,
        request: Request<Streaming<SendRequest>>,
    ) -> Result<Response<Self::SendStream>, Status> {
        let stream = request.into_inner();
        self.send(stream).await
    }
}

#[allow(unused)]
#[derive(Clone)]
pub struct NotificationService {
    inner: Arc<NotifationServiceInner>,
}

#[allow(unused)]
pub struct NotifationServiceInner {
    config: AppConfig,
    sender: mpsc::Sender<Msg>,
}
