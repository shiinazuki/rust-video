mod chat;
mod file;
mod messsage;
mod user;
mod workspace;

pub use chat::{CreateChat, UpdateChat};
pub use messsage::{CreateMessage, ListMessages};
use serde::{Deserialize, Serialize};
pub use user::{CreateUser, SigninUser};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatFile {
    pub ws_id: u64,
    pub ext: String,
    pub hash: String,
}
