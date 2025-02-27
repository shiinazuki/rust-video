use std::net::SocketAddr;

use anyhow::Result;
use crm_metadata::{
    MetadataService, get_configuration_test,
    pb::{MaterializeRequest, metadata_client::MetadataClient},
};
use tokio_stream::StreamExt;
use tonic::{Request, transport::Server};

#[tokio::test]
async fn test_metadata() -> Result<()> {
    let addr = start_server().await?;
    let mut client = MetadataClient::connect(format!("http://{}", addr)).await?;
    let stream = tokio_stream::iter(vec![
        MaterializeRequest { id: 1 },
        MaterializeRequest { id: 2 },
        MaterializeRequest { id: 3 },
    ]);
    let request = Request::new(stream);

    let response = client.materialize(request).await?.into_inner();

    let ret = response
        .then(|res| async { res.unwrap() })
        .collect::<Vec<_>>()
        .await;

    println!("{:#?}", ret);
    assert_eq!(ret.len(), 3);

    Ok(())
}

async fn start_server() -> Result<SocketAddr> {
    let config = get_configuration_test()?;
    let addr = format!("{}:{}", config.application.host, config.application.port).parse()?;
    tokio::spawn(async move {
        // maybe connecton error please move to tokio::spawn before
        let svc = MetadataService::new(config).into_server();
        Server::builder()
            .add_service(svc)
            .serve(addr)
            .await
            .unwrap();
    });

    Ok(addr)
}
