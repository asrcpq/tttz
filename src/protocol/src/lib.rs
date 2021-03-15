mod board_msg;
pub use board_msg::{BoardMsg, BoardReply};
mod client_msg;
pub use client_msg::ClientMsg;
mod msg_encoding;
pub use msg_encoding::MsgEncoding;
mod server_msg;
pub use server_msg::ServerMsg;
mod key_type;
pub use key_type::KeyType;
mod game_type;
pub use game_type::GameType;
mod display;
pub use display::Display;
mod piece;
pub use piece::Piece;

pub type IdType = i32;
