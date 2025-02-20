use anyhow::Result;
use crm_server::pb::{user_service_server::{UserService, UserServiceServer}, CreateUserRequest, GetUserRequest, User};
use prost::Message;
use tonic::{async_trait, transport::Server, Request, Response, Status};

#[derive(Default)]
pub struct UserServer {}

#[async_trait]
impl UserService for UserServer {
    async fn get_user(&self, request: Request<GetUserRequest>) -> Result<Response<User>, Status> {
        let user = request.into_inner();
        println!("get_user: {:?}", user);
        Ok(Response::new(User::default()))
    }

    async fn create_user(
        &self,
        request: Request<CreateUserRequest>,
    ) -> Result<Response<User>, tonic::Status> {
        let user = request.into_inner();
        println!("create_user: {:?}", user);
        let user = User::new(1, &user.name, &user.email);
        Ok(tonic::Response::new(user))
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let user = User::new(1, "shiina", "shiina@acme.org");
    println!("user: {:?}", user);
    let encoded = user.encode_to_vec();
    println!("user encoded: {:?}", encoded);
    let decoded = User::decode(&encoded[..])?;
    println!("user decoded: {:?}", decoded);

    let addr = "127.0.0.1:50051".parse().unwrap();
    let svc = UserServer::default();

    println!("UserServce ligtening on {}", addr);

    Server::builder()
        .add_service(UserServiceServer::new(svc))
        .serve(addr)
        .await?;

    Ok(())
}
