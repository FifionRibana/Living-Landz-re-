use bevy::prelude::*;
use bincode::{Decode, Encode};
use hexx::*;

#[derive(Component, Default, Debug, Clone, Copy, PartialEq, Eq, Hash, Encode, Decode)]
pub struct GridCell {
    pub q: i32,
    pub r: i32,
}

impl GridCell {
    pub fn from_hex(hex_cell: &Hex) -> Self {
        Self {
            q: hex_cell.x,
            r: hex_cell.y,
        }
    }
    pub fn to_hex(&self) -> Hex {
        Hex::new(self.q, self.r)
    }

    pub fn neighbors(&self) -> Vec<GridCell> {
        let hex_cell = self.to_hex();
        hex_cell
            .all_neighbors()
            .iter()
            .map(Self::from_hex)
            .collect::<Vec<_>>()
    }
}
