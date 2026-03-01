use bevy::prelude::*;

/// Categories of heraldic layers, rendered bottom-to-top.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HeraldryLayer {
    /// Shield outline / shape (écu).
    Shield,
    /// Field division pattern (champ: plain, parti, coupé, écartelé…).
    Field,
    /// Primary charge / figure (meuble: lion, aigle, croix…).
    Charge,
    /// Border / ornament (bordure). Future extension.
    Bordure,
}

impl HeraldryLayer {
    /// All layers in render order (bottom first).
    pub const ALL: &'static [HeraldryLayer] = &[
        HeraldryLayer::Shield,
        HeraldryLayer::Field,
        HeraldryLayer::Charge,
        HeraldryLayer::Bordure,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            Self::Shield => "Écu",
            Self::Field => "Champ",
            Self::Charge => "Meuble",
            Self::Bordure => "Bordure",
        }
    }

    /// Asset folder name under `sprites/coat_of_arms/layers/`.
    pub fn folder(&self) -> &'static str {
        match self {
            Self::Shield => "shield",
            Self::Field => "field",
            Self::Charge => "charge",
            Self::Bordure => "bordure",
        }
    }

    /// Icon letter for the selector row.
    pub fn icon(&self) -> &'static str {
        match self {
            Self::Shield => "E",
            Self::Field => "C",
            Self::Charge => "M",
            Self::Bordure => "B",
        }
    }

    /// Whether this layer is available or greyed-out for future.
    pub fn is_available(&self) -> bool {
        !matches!(self, Self::Bordure)
    }
}

/// One heraldic layer's selection state.
#[derive(Debug, Clone)]
pub struct HeraldryLayerState {
    pub layer: HeraldryLayer,
    pub current: usize,
    pub total: usize,
}

impl HeraldryLayerState {
    pub fn new(layer: HeraldryLayer, total: usize) -> Self {
        Self {
            layer,
            current: 0,
            total,
        }
    }

    pub fn next(&mut self) {
        if self.total > 0 {
            self.current = (self.current + 1) % self.total;
        }
    }

    pub fn prev(&mut self) {
        if self.total > 0 {
            self.current = (self.current + self.total - 1) % self.total;
        }
    }

    /// Asset path, e.g. `sprites/coat_of_arms/layers/shield/shield_01.png`.
    pub fn asset_path(&self) -> String {
        format!(
            "sprites/coat_of_arms/layers/{}/{}_{:02}.png",
            self.layer.folder(),
            self.layer.folder(),
            self.current + 1
        )
    }

    /// Display string, e.g. "3 / 8".
    pub fn counter_text(&self) -> String {
        if self.total == 0 {
            "—".to_string()
        } else {
            format!("{} / {}", self.current + 1, self.total)
        }
    }
}

/// Resource holding the full coat of arms creation state.
#[derive(Resource, Debug, Clone)]
pub struct CoatOfArmsCreationState {
    pub layers: Vec<HeraldryLayerState>,
    pub motto: String,
}

impl Default for CoatOfArmsCreationState {
    fn default() -> Self {
        Self {
            layers: vec![
                HeraldryLayerState::new(HeraldryLayer::Shield, 6),
                HeraldryLayerState::new(HeraldryLayer::Field, 10),
                HeraldryLayerState::new(HeraldryLayer::Charge, 12),
                HeraldryLayerState::new(HeraldryLayer::Bordure, 0), // unavailable for now
            ],
            motto: String::new(),
        }
    }
}

impl CoatOfArmsCreationState {
    pub fn layer_mut(&mut self, layer: HeraldryLayer) -> Option<&mut HeraldryLayerState> {
        self.layers.iter_mut().find(|l| l.layer == layer)
    }

    pub fn layer(&self, layer: HeraldryLayer) -> Option<&HeraldryLayerState> {
        self.layers.iter().find(|l| l.layer == layer)
    }
}
