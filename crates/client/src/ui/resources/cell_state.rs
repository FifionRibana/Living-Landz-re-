use bevy::prelude::*;
use shared::{
    BiomeTypeEnum, BuildingData, SlotConfiguration,
    grid::{CellData, GridCell},
};

use crate::ui::components::SlotIndicator;

#[derive(Resource, Default)]
pub struct CellState {
    pub cell_data: Option<CellData>,
    pub building_data: Option<BuildingData>,

    pub cell_units: Vec<u64>,
    pub cell_slots: Vec<SlotIndicator>,
    
    pub dragging_unit: Option<u64>,
}

impl CellState {
    pub fn enter_view(&mut self, cell_data: Option<CellData>, building_data: Option<BuildingData>) {
        self.cell_data = cell_data;
        self.building_data = building_data;
    }

    pub fn exit_view(&mut self) {
        self.cell_data = None;
        self.building_data = None;

        self.dragging_unit = None;
    }

    pub fn cell(&self) -> Option<GridCell> {
        self.cell_data.map(|cell_data| cell_data.cell)
    }

    pub fn biome(&self) -> BiomeTypeEnum {
        if let Some(cell_data) = self.cell_data {
            cell_data.biome
        } else {
            BiomeTypeEnum::Undefined
        }
    }

    pub fn is_dragging(&self) -> bool {
        self.dragging_unit.is_some()
    }

    pub fn is_building(&self) -> bool {
        self.building_data.is_some()
    }

    pub fn slot_configuration(&self) -> SlotConfiguration {
        // Determine slot configuration based on building type or terrain
        if let Some(building_data) = &self.building_data {
            // Try to get slot config from building type first
            if let Some(building_type) = building_data.to_building_type() {
                SlotConfiguration::for_building_type(building_type)
            } else {
                // Fallback to terrain type for trees or unknown buildings
                SlotConfiguration::for_terrain_type(self.biome())
            }
        } else {
            // No building, use terrain type
            SlotConfiguration::for_terrain_type(self.biome())
        }
    }

    pub fn has_interior(&self) -> bool {
        self.slot_configuration().has_interior()
    }

}
