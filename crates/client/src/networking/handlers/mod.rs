// =============================================================================
// NETWORKING - Handlers (split by domain)
// =============================================================================

pub mod actions;
pub mod auth;
pub mod debug;
pub mod territory;
pub mod units;
pub mod world;

use shared::{SlotPosition, SlotType};

/// Helper: convert database slot strings to SlotPosition.
pub(crate) fn db_to_slot_position(
    slot_type: Option<String>,
    slot_index: Option<i32>,
) -> Option<SlotPosition> {
    match (slot_type, slot_index) {
        (Some(type_str), Some(index)) if index >= 0 => {
            let slot_type_enum = match type_str.as_str() {
                "interior" => SlotType::Interior,
                "exterior" => SlotType::Exterior,
                _ => return None,
            };
            Some(SlotPosition {
                slot_type: slot_type_enum,
                index: index as usize,
            })
        }
        _ => None,
    }
}
