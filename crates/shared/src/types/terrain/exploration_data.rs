use bincode::{Decode, Encode};

#[derive(Debug, Clone, Encode, Decode)]
pub struct ExplorationData {
    pub width: i32,
    pub height: i32,
    pub data: Vec<u8>,
}