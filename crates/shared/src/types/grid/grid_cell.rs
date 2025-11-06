use bevy::prelude::*;
use bincode::{Decode, Encode};

#[derive(Component, Default, Debug, Clone, Copy, PartialEq, Eq, Hash, Encode, Decode)]
pub struct GridCell {
    pub q: i32,
    pub r: i32,
}
