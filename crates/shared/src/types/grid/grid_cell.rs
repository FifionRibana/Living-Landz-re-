use bevy::prelude::*;
use bincode::{Decode, Encode};
use hexx::*;

use crate::{TerrainChunkId, constants};

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

    pub fn to_chunk_id(&self, layout: &HexLayout) -> TerrainChunkId {
        let world_pos = layout.hex_to_world_pos(self.to_hex());
        TerrainChunkId {
            x: world_pos.x.div_euclid(constants::CHUNK_SIZE.x).ceil() as i32,
            y: world_pos.y.div_euclid(constants::CHUNK_SIZE.y).ceil() as i32,
        }
    }

    pub fn neighbors(&self) -> Vec<GridCell> {
        let hex_cell = self.to_hex();
        hex_cell
            .all_neighbors()
            .iter()
            .map(Self::from_hex)
            .collect::<Vec<_>>()
    }

    /// Retourne les voisins indirects (distance 2) de la cellule
    /// Pour une cellule (q:0, r:0), les voisins indirects sont:
    /// (2,-1), (1,-2), (-1,-1), (-2,1), (-1,2), (1,1)
    pub fn indirect_neighbors(&self) -> Vec<GridCell> {
        vec![
            GridCell { q: self.q + 2, r: self.r - 1 },
            GridCell { q: self.q + 1, r: self.r - 2 },
            GridCell { q: self.q - 1, r: self.r - 1 },
            GridCell { q: self.q - 2, r: self.r + 1 },
            GridCell { q: self.q - 1, r: self.r + 2 },
            GridCell { q: self.q + 1, r: self.r + 1 },
        ]
    }

    /// Retourne tous les voisins (directs + indirects)
    pub fn all_extended_neighbors(&self) -> Vec<GridCell> {
        let mut all = self.neighbors();
        all.extend(self.indirect_neighbors());
        all
    }
}
