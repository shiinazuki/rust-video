use chrono::{DateTime, TimeZone, Utc};
use itertools::Itertools;
use prost_types::Timestamp;
use tonic::{Response, Status};

use crate::{
    ResponseStream, ServiceResult, UserStatsService,
    pb::{QueryRequest, RawQueryRequest, User},
};

impl UserStatsService {
    pub async fn query(&self, query: QueryRequest) -> ServiceResult<ResponseStream> {
        let mut sql = "SELECT email, name FROM user_stats WHERE ".to_string();
        let time_conditions = query
            .timestamps
            .into_iter()
            .map(|(k, v)| timestamp_query(&k, v.lower, v.upper))
            .join(" AND ");

        sql.push_str(&time_conditions);

        let id_conditions = query
            .ids
            .into_iter()
            .map(|(k, v)| ids_query(&k, v.ids))
            .join(" AND ");

        sql.push_str(" AND ");
        sql.push_str(&id_conditions);

        self.raw_query(RawQueryRequest { query: sql }).await
    }

    pub async fn raw_query(&self, req: RawQueryRequest) -> ServiceResult<ResponseStream> {
        let Ok(ret) = sqlx::query_as::<_, User>(&req.query)
            .fetch_all(&self.pool)
            .await
        else {
            return Err(Status::internal(format!(
                "Failed to fetch data with query: {}",
                req.query
            )));
        };

        Ok(Response::new(Box::pin(futures::stream::iter(
            ret.into_iter().map(Ok),
        ))))
    }
}

fn ids_query(name: &str, ids: Vec<u32>) -> String {
    if ids.is_empty() {
        return "TURE".to_string();
    }
    format!("array{:?} <@ ({})", ids, name)
}

fn timestamp_query(name: &str, before: Option<Timestamp>, after: Option<Timestamp>) -> String {
    if before.is_none() && after.is_none() {
        return "TRUE".to_owned();
    }

    if before.is_none() {
        let after = ts_to_utc(after.unwrap());
        return format!("{} <= '{}'", name, after.to_rfc3339());
    }

    if after.is_none() {
        let before = ts_to_utc(before.unwrap());
        return format!("{} >= '{}'", name, before.to_rfc3339());
    }

    format!(
        "{} BETWEEN '{}' AND '{}'",
        name,
        ts_to_utc(after.unwrap()).to_rfc3339(),
        ts_to_utc(before.unwrap()).to_rfc3339(),
    )
}

fn ts_to_utc(ts: Timestamp) -> DateTime<Utc> {
    Utc.timestamp_opt(ts.seconds, ts.nanos as u32).unwrap()
}

#[cfg(test)]
mod tests {
    use crate::pb::{IdQuery, QueryRequestBuilder, TimeQuery};

    use super::*;
    use anyhow::Result;
    use futures::StreamExt;

    #[tokio::test]
    async fn raw_query_should_word() -> Result<()> {
        let svc = UserStatsService::new_test().await;
        let mut stream = svc
            .raw_query(RawQueryRequest {
                query:
                    "SELECT name, email FROM user_stats WHERE created_at > '2024-11-01' LIMIT 10"
                        .to_string(),
            })
            .await?
            .into_inner();

        while let Some(res) = stream.next().await {
            println!("{:?}", res);
        }

        Ok(())
    }

    #[tokio::test]
    async fn query_should_work() -> Result<()> {
        let svc = UserStatsService::new_test().await;
        let query = QueryRequestBuilder::default()
            .timestamp(("created_at".to_string(), tq(None, Some(120))))
            .timestamp(("last_visited_at".to_string(), tq(Some(30), None)))
            .id(("viewed_but_not_started".to_string(), id(&[23054])))
            .build()
            .unwrap();
        let mut stream = svc.query(query).await?.into_inner();
        while let Some(res) = stream.next().await {
            println!("{:?}", res);
        }
        Ok(())
    }

    fn to_ts(days: i64) -> Timestamp {
        let dt = Utc::now()
            .checked_sub_signed(chrono::Duration::days(days))
            .unwrap();
        Timestamp {
            seconds: dt.timestamp(),
            nanos: dt.timestamp_subsec_nanos() as i32,
        }
    }

    fn tq(upper: Option<i64>, lower: Option<i64>) -> TimeQuery {
        TimeQuery {
            upper: upper.map(to_ts),
            lower: lower.map(to_ts),
        }
    }

    fn id(id: &[u32]) -> IdQuery {
        IdQuery { ids: id.to_vec() }
    }
}
