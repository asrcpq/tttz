mod board_msg;
pub use board_msg::{BoardMsg, BoardReply};
mod client_msg;
pub use client_msg::ClientMsg;
mod server_msg;
pub use server_msg::ServerMsg;
mod key_type;
pub use key_type::KeyType;
mod ai_type;
pub use ai_type::AiType;

mod display;
pub use display::Display;

pub type IdType = i32;
