use anyhow::Result;
use crm_server::pb::{CreateUserRequest, user_service_client::UserServiceClient};
use tonic::Request;

#[tokio::main]
async fn main() -> Result<()> {
    let mut client = UserServiceClient::connect("http://127.0.0.1:50051").await?;

    let request = Request::new(CreateUserRequest {
        name: "shiina".into(),
        email: "shiina@acme.org".into(),
    });

    let response = client.create_user(request).await?;
    println!("response={:#?}", response);
    let user = response.into_inner();
    println!("user={:#?}", user);
    Ok(())
}
