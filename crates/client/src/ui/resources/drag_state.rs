use bevy::prelude::*;
use shared::SlotPosition;

#[derive(Resource, Default)]
pub struct DragState {
    pub active: Option<DragInfo>,
    pub hovered_slot: Option<Entity>,
}

pub struct DragInfo {
    pub source_slot: Entity,
    pub unit_entity: Entity,
    pub source_position: SlotPosition,
}