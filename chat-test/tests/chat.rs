use std::{net::SocketAddr, time::Duration};

use anyhow::Result;
use chat_core::{Chat, ChatType, Message};
use futures::StreamExt;
use reqwest::{
    StatusCode,
    multipart::{Form, Part},
};
use reqwest_eventsource::{Event, EventSource};
use secrecy::{ExposeSecret, SecretBox};
use serde::Deserialize;
use serde_json::json;
use tokio::{net::TcpListener, time::sleep};

const WILD_ADDR: &str = "0.0.0.0:0";

#[derive(Debug, Deserialize)]
struct AuthToken {
    token: String,
}

struct ChatServer {
    addr: SocketAddr,
    token: String,
    client: reqwest::Client,
}

impl ChatServer {
    async fn new(state: chat_server::AppState) -> Result<Self> {
        let app = chat_server::get_router(state).await?;
        let listener = TcpListener::bind(WILD_ADDR).await?;
        let addr = listener.local_addr()?;

        tokio::spawn(async move {
            axum::serve(listener, app.into_make_service())
                .await
                .unwrap();
        });

        let client = reqwest::Client::new();

        let mut ret = Self {
            addr,
            client,
            token: "".to_string(),
        };

        ret.token = ret.signin().await?;

        Ok(ret)
    }

    async fn signin(&self) -> Result<String> {
        let res = self
            .client
            .post(&format!("http://{}/api/signin", self.addr))
            .header("Content-Type", "application/json")
            .body(r#"{"email": "test@acme.org","password":"123456"}"#)
            .send()
            .await?;

        assert_eq!(res.status(), 200);
        let ret: AuthToken = res.json().await?;
        Ok(ret.token)
    }

    async fn create_chat(&self) -> Result<Chat> {
        let res = self
            .client
            .post(&format!("http://{}/api/chats", self.addr))
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Content-Type", "application/json")
            .body(r#"{"name": "test", "members": [1, 2], "public": false}"#)
            .send()
            .await?;

        assert_eq!(res.status(), StatusCode::CREATED);
        let chat: Chat = res.json().await?;
        assert_eq!(chat.name, Some("test".to_owned()));
        assert_eq!(chat.members, vec![1, 2]);
        assert_eq!(chat.r#type, ChatType::PrivateChannel);

        Ok(chat)
    }

    async fn create_message(&self, chat_id: u64) -> Result<Message> {
        let data = include_bytes!("../Cargo.toml");
        let files = Part::bytes(data)
            .file_name("Cargo.toml")
            .mime_str("text/plain")?;
        let form = Form::new().part("file", files);

        let res = self
            .client
            .post(format!("http://{}/api/upload", self.addr))
            .header("Authorization", format!("Bearer {}", self.token))
            .multipart(form)
            .send()
            .await?;

        assert_eq!(res.status(), StatusCode::OK);
        let ret: Vec<String> = res.json().await?;

        let body = serde_json::to_string(&json!({"content": "Hello, World!", "files": ret,}))?;

        let res = self
            .client
            .post(&format!("http://{}/api/chats/{}", self.addr, chat_id))
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Content-Type", "application/json")
            .body(body)
            .send()
            .await?;

        assert_eq!(res.status(), StatusCode::CREATED);

        let message: Message = res.json().await?;
        assert_eq!(message.content, "Hello, World!");
        assert_eq!(message.chat_id, chat_id as i64);
        assert_eq!(message.files, ret);
        assert_eq!(message.sender_id, 1);
        Ok(message)
    }
}

struct NotifyServer;

impl NotifyServer {
    async fn new(
        username: &str,
        port: u16,
        password: &SecretBox<String>,
        host: &str,
        database_name: &str,
        token: &str,
    ) -> Result<Self> {
        let mut config = chat_notify_server::get_configuration()?;
        let password = password.expose_secret().to_owned();
        config.database.username = username.to_owned();
        config.database.port = port;
        config.database.password = SecretBox::new(Box::new(password));
        config.database.host = host.to_owned();
        config.database.database_name = database_name.to_owned();
        let app = chat_notify_server::get_router(config).await?;
        let listener = TcpListener::bind(WILD_ADDR).await?;
        let addr = listener.local_addr()?;

        tokio::spawn(async move {
            axum::serve(listener, app.into_make_service())
                .await
                .unwrap()
        });

        let mut es = EventSource::get(format!("http://{}/events?access_token={}", addr, token));

        tokio::spawn(async move {
            while let Some(event) = es.next().await {
                match event {
                    Ok(Event::Open) => println!("Connection Open!"),
                    Ok(Event::Message(message)) => match message.event.as_str() {
                        "NewChat" => {
                            let chat: Chat = serde_json::from_str(&message.data).unwrap();
                            assert_eq!(chat.name.as_ref().unwrap(), "test");
                            assert_eq!(chat.members, vec![1, 2]);
                            assert_eq!(chat.r#type, ChatType::PrivateChannel);
                        }

                        "NewMessage" => {
                            let msg: Message = serde_json::from_str(&message.data).unwrap();
                            assert_eq!(msg.content, "Hello World!");
                            assert_eq!(msg.files.len(), 1);
                            assert_eq!(msg.sender_id, 1);
                        }
                        _ => {
                            panic!("unexpected event: {:?}", message);
                        }
                    },
                    Err(err) => {
                        println!("Error: {}", err);
                        es.close();
                    }
                }
            }
        });
        Ok(Self)
    }
}

#[tokio::test]
async fn chat_server_should_work() -> Result<()> {
    let (tdb, state) = chat_server::AppState::new_for_test().await?;
    let chat_server = ChatServer::new(state.clone()).await?;
    NotifyServer::new(
        &state.config.database.username,
        state.config.database.port,
        &state.config.database.password,
        &state.config.database.host,
        &tdb.dbname,
        &chat_server.token,
    )
    .await?;
    let chat = chat_server.create_chat().await?;
    let _message = chat_server.create_message(chat.id as u64).await?;

    sleep(Duration::from_secs(10)).await;
    Ok(())
}
