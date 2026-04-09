use bevy::prelude::*;
use std::collections::HashMap;

#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
pub enum DebugLayer {
    Ocean,
    Lake,
    Terrain,
    Mist,
    GroundFog,
    Trees,
    Buildings,
    DomainBorders,
}

impl DebugLayer {
    pub fn all() -> &'static [DebugLayer] {
        &[
            Self::Ocean,
            Self::Lake,
            Self::Terrain,
            Self::Mist,
            Self::GroundFog,
            Self::Trees,
            Self::Buildings,
            Self::DomainBorders,
        ]
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Ocean => "Ocean",
            Self::Lake => "Lake",
            Self::Terrain => "Terrain",
            Self::Mist => "Mist",
            Self::GroundFog => "Ground Fog",
            Self::Trees => "Trees",
            Self::Buildings => "Buildings",
            Self::DomainBorders => "Domain Borders",
        }
    }

    pub fn group(&self) -> Option<&'static str> {
        match self {
            Self::Mist | Self::GroundFog => Some("All Fog"),
            Self::Trees | Self::Buildings => Some("All Sprites"),
            _ => None,
        }
    }
}

#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
pub enum DebugOverride {
    LoadUnexploredChunks,
}

impl DebugOverride {
    pub fn all() -> &'static [DebugOverride] {
        &[Self::LoadUnexploredChunks]
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::LoadUnexploredChunks => "Load unexplored chunks",
        }
    }
}

#[derive(Resource)]
pub struct DebugLayerVisibility {
    pub panel_visible: bool,
    pub layers: HashMap<DebugLayer, bool>,
    pub overrides: HashMap<DebugOverride, bool>,
}

impl Default for DebugLayerVisibility {
    fn default() -> Self {
        let mut layers = HashMap::new();
        for &layer in DebugLayer::all() {
            layers.insert(layer, true);
        }
        let mut overrides = HashMap::new();
        for &ov in DebugOverride::all() {
            overrides.insert(ov, false);
        }
        Self {
            panel_visible: false,
            layers,
            overrides,
        }
    }
}

impl DebugLayerVisibility {
    pub fn is_visible(&self, layer: DebugLayer) -> bool {
        self.layers.get(&layer).copied().unwrap_or(true)
    }

    pub fn is_override_active(&self, ov: DebugOverride) -> bool {
        self.overrides.get(&ov).copied().unwrap_or(false)
    }

    pub fn toggle_layer(&mut self, layer: DebugLayer) {
        let entry = self.layers.entry(layer).or_insert(true);
        *entry = !*entry;
    }

    pub fn toggle_override(&mut self, ov: DebugOverride) {
        let entry = self.overrides.entry(ov).or_insert(false);
        *entry = !*entry;
    }

    /// Set all layers in a group to the given value.
    pub fn set_group(&mut self, group_name: &str, value: bool) {
        for &layer in DebugLayer::all() {
            if layer.group() == Some(group_name) {
                self.layers.insert(layer, value);
            }
        }
    }

    /// Check if all layers in a group are visible.
    pub fn is_group_all_on(&self, group_name: &str) -> bool {
        DebugLayer::all()
            .iter()
            .filter(|l| l.group() == Some(group_name))
            .all(|l| self.is_visible(*l))
    }

    /// Check if any layer in a group is visible.
    pub fn is_group_any_on(&self, group_name: &str) -> bool {
        DebugLayer::all()
            .iter()
            .filter(|l| l.group() == Some(group_name))
            .any(|l| self.is_visible(*l))
    }
}
