use anyhow::Result;
use crm_send::{
    NotificationService, get_configuration_test,
    pb::{
        EmailMessage, InAppMessage, SendRequest, SmsMessage,
        notification_client::NotificationClient,
    },
};
use std::net::SocketAddr;
use tokio_stream::StreamExt;
use tonic::{Request, transport::Server};

#[tokio::test]
async fn test_send() -> Result<()> {
    let addr = start_server().await?;
    let mut client = NotificationClient::connect(format!("http://{}", addr)).await?;
    let stream = tokio_stream::iter(vec![
        SendRequest {
            msg: Some(EmailMessage::fake().into()),
        },
        SendRequest {
            msg: Some(SmsMessage::fake().into()),
        },
        SendRequest {
            msg: Some(InAppMessage::fake().into()),
        },
    ]);

    let request = Request::new(stream);
    let response = client.send(request).await?.into_inner();
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
        let svc = NotificationService::new(config).into_server();
        Server::builder()
            .add_service(svc)
            .serve(addr)
            .await
            .unwrap();
    });

    Ok(addr)
}
