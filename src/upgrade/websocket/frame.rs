//! WebSocket frame handling

use crate::upgrade::websocket::Message;

/// Frame handler
pub struct FrameHandler;

impl FrameHandler {
    /// Encode message to frame
    pub fn encode(msg: &Message) -> Vec<u8> {
        // TODO: Encode message to WebSocket frame
        todo!("Implement FrameHandler::encode")
    }

    /// Decode frame to message
    pub fn decode(data: &[u8]) -> Option<Message> {
        // TODO: Decode WebSocket frame to message
        todo!("Implement FrameHandler::decode")
    }
}
