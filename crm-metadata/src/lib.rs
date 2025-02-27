mod abi;
mod configuration;
pub mod pb;

use std::pin::Pin;

pub use configuration::{AppConfig, get_configuration, get_configuration_test};
use futures::Stream;
use pb::{
    Content, MaterializeRequest,
    metadata_server::{Metadata, MetadataServer},
};
use tonic::{Request, Response, Status, Streaming, async_trait};

type ServiceResult<T> = Result<Response<T>, Status>;

type ResponseStream = Pin<Box<dyn Stream<Item = Result<Content, Status>> + Send>>;

#[async_trait]
impl Metadata for MetadataService {
    type MaterializeStream = ResponseStream;

    async fn materialize(
        &self,
        request: Request<Streaming<MaterializeRequest>>,
    ) -> ServiceResult<Self::MaterializeStream> {
        let query = request.into_inner();
        self.materialize(query).await
    }
}

#[allow(unused)]
pub struct MetadataService {
    config: AppConfig,
}

impl MetadataService {
    pub fn new(config: AppConfig) -> Self {
        MetadataService { config }
    }

    pub fn into_server(self) -> MetadataServer<Self> {
        MetadataServer::new(self)
    }
}
