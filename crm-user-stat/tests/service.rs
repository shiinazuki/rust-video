use std::net::SocketAddr;

use anyhow::Result;
use crm_user_stat::{
    UserStatsService, get_configuration_test,
    pb::{QueryRequestBuilder, RawQueryRequestBuilder, user_stats_client::UserStatsClient},
    test_utils::{id, tq},
};
use futures::StreamExt;
use sqlx_db_tester::TestPg;
use tonic::transport::Server;

const PORT_BASE: u32 = 60000;

#[tokio::test]
async fn raw_query_should_work() -> Result<()> {
    let (_tdb, addr) = start_server(PORT_BASE).await?;
    let mut client = UserStatsClient::connect(format!("http://{}", addr)).await?;
    let req = RawQueryRequestBuilder::default()
        .query("SELECT * FROM user_stats WHERE created_at > '2024-01-01' LIMIT 5")
        .build()?;
    let stream = client.raw_query(req).await?.into_inner();
    let ret = stream
        .then(|res| async { res.unwrap() })
        .collect::<Vec<_>>()
        .await;

    println!("{:#?}", ret);
    assert_eq!(ret.len(), 5);
    Ok(())
}

#[tokio::test]
async fn query_should_word() -> Result<()> {
    let (_tbd, addr) = start_server(PORT_BASE + 1).await?;
    let mut client = UserStatsClient::connect(format!("http://{}", addr)).await?;
    let query = QueryRequestBuilder::default()
        .timestamp(("created_at".to_string(), tq(None, Some(1200))))
        .timestamp(("last_visited_at".to_string(), tq(Some(30), None)))
        // 23054
        .id(("viewed_but_not_started".to_string(), id(&[252790])))
        .build()?;

    let stream = client.query(query).await?.into_inner();
    let ret = stream.collect::<Vec<_>>().await;

    println!("{:#?}", ret);
    assert!(ret.len() > 0);

    Ok(())
}

async fn start_server(port: u32) -> Result<(TestPg, SocketAddr)> {
    let config = get_configuration_test()?;
    let addr = format!("{}:{}", config.application.host, port).parse()?;
    let (tdb, svc) = UserStatsService::new_for_test().await?;
    tokio::spawn(async move {
        Server::builder()
            .add_service(svc.into_server())
            .serve(addr)
            .await
            .unwrap();
    });

    Ok((tdb, addr))
}
