mod abi;
mod configuration;
pub mod pb;

pub use configuration::{AppConfig, get_configuration, get_configuration_test};

use secrecy::ExposeSecret;
use sqlx::PgPool;

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

#[allow(unused)]
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
}

impl Deref for UserStatsService {
    type Target = UserStatsServiceInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[cfg(feature = "test_utils")]
pub mod test_utils {
    use std::sync::Arc;

    use anyhow::Result;
    use chrono::Utc;
    use prost_types::Timestamp;
    use secrecy::ExposeSecret;
    use sqlx::PgPool;
    use sqlx_db_tester::TestPg;

    use crate::{
        UserStatsService, UserStatsServiceInner, get_configuration_test,
        pb::{IdQuery, TimeQuery},
    };

    impl UserStatsService {
        pub async fn new_for_test() -> Result<(TestPg, Self)> {
            let config = get_configuration_test().unwrap();

            let db_url = &config
                .database
                .connection_string()
                .expose_secret()
                .to_string();
            let (tdb, pool) = get_test_pool(Some(db_url)).await;

            // let redis_client =
            //     redis::Client::open(config.redis.connection_url().expose_secret().as_ref())?;

            // let redis_pool = r2d2::Pool::builder().build(redis_client)?;

            let svc = Self {
                inner: Arc::new(UserStatsServiceInner {
                    config,
                    pool,
                    // redis_pool,
                }),
            };
            Ok((tdb, svc))
        }
    }

    pub async fn get_test_pool(db_url: Option<&str>) -> (TestPg, PgPool) {
        use sqlx::Executor;

        let db_url = match db_url {
            Some(v) => v.to_string(),
            None => "postgres://postgres:postgres@127.0.0.1:5432/stats".to_string(),
        };
        let tdb = TestPg::new(db_url, std::path::Path::new("../migrations"));
        let pool = tdb.get_pool().await;

        // run prepared sql to insert test data
        let sql = include_str!("../fixtures/data.sql").split(';');
        let mut ts = pool.begin().await.expect("begin transaction failed");

        for s in sql {
            if s.trim().is_empty() {
                continue;
            }
            ts.execute(s).await.unwrap();
        }

        ts.commit().await.expect("commit transaction failed");

        (tdb, pool)
    }

    pub fn to_ts(days: i64) -> Timestamp {
        let dt = Utc::now()
            .checked_sub_signed(chrono::Duration::days(days))
            .unwrap();
        Timestamp {
            seconds: dt.timestamp(),
            nanos: dt.timestamp_subsec_nanos() as i32,
        }
    }

    pub fn tq(upper: Option<i64>, lower: Option<i64>) -> TimeQuery {
        TimeQuery {
            upper: upper.map(to_ts),
            lower: lower.map(to_ts),
        }
    }

    pub fn id(id: &[u32]) -> IdQuery {
        IdQuery { ids: id.to_vec() }
    }
}
