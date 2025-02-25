mod abi;
mod configuration;
pub mod pb;

pub use configuration::{AppConfig, get_configuration};
use secrecy::ExposeSecret;
use sqlx::{PgPool, query};

use std::{ops::Deref, pin::Pin, sync::Arc};

use anyhow::Result;
use futures::Stream;
use pb::{
    QueryRequest, RawQueryRequest, User,
    user_stats_server::{UserStats, UserStatsServer},
};
use tonic::{Request, Response, Status, async_trait};

type ServiceResult<T> = Result<Response<T>, Status>;

type ResponseStream = Pin<Box<dyn Stream<Item = Result<User, Status>> + Send>>;

#[derive(Clone)]
pub struct UserStatsService {
    inner: Arc<UserStatsServiceInner>,
}

pub struct UserStatsServiceInner {
    config: AppConfig,
    pool: PgPool,
}

#[async_trait]
impl UserStats for UserStatsService {
    type QueryStream = ResponseStream;
    type RawQueryStream = ResponseStream;

    async fn query(&self, request: Request<QueryRequest>) -> ServiceResult<Self::QueryStream> {
        let query = request.into_inner();
        self.query(query).await
    }

    async fn raw_query(
        &self,
        request: Request<RawQueryRequest>,
    ) -> ServiceResult<Self::RawQueryStream> {
        let query = request.into_inner();
        self.raw_query(query).await
    }
}

impl UserStatsService {
    pub async fn new(config: AppConfig) -> Self {
        let db_url = config
            .database
            .connection_string()
            .expose_secret()
            .to_owned();
        let pool = PgPool::connect(&db_url)
            .await
            .expect("failed to connect to db");
        let inner = UserStatsServiceInner { config, pool };
        Self {
            inner: Arc::new(inner),
        }
    }

    pub fn into_server(self) -> UserStatsServer<Self> {
        UserStatsServer::new(self)
    }

    #[cfg(test)]
    pub async fn new_test() -> Self {
        use configuration::get_configuration_test;

        let config = get_configuration_test().expect("Failed to load config");
        let db_url = config
            .database
            .connection_string()
            .expose_secret()
            .to_owned();
        let pool = PgPool::connect(&db_url)
            .await
            .expect("failed to connect to db");
        let inner = UserStatsServiceInner { config, pool };
        Self {
            inner: Arc::new(inner),
        }
    }
}

impl Deref for UserStatsService {
    type Target = UserStatsServiceInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
