use bevy::prelude::*;
use shared::SlotType;

/// Utility to get the background color for a slot type
pub fn get_slot_background_color(slot_type: SlotType) -> Color {
    match slot_type {
        SlotType::Interior => Color::srgba(0.3, 0.4, 0.8, 0.6), // Blue tint
        SlotType::Exterior => Color::srgba(0.3, 0.7, 0.4, 0.6), // Green tint
    }
}
