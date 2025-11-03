use bevy::prelude::*;
use hexx::Hex;

#[derive(Component)]
pub struct HexSelectIndicator {
    pub hex: Hex, // Case active
}