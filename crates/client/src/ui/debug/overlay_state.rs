use bevy::prelude::*;

/// Shader base visualization mode (mutually exclusive).
/// Encoded as debug_params.x in the terrain shader.
#[derive(Debug, Default, PartialEq, Clone, Copy)]
pub enum ShaderBaseMode {
    #[default]
    Normal,
    SlopeClasses,
    AltitudeBands,
    RawHeightmap,
    BiomeIds,
    SdfCoastal,
    Shoreline,
}

impl ShaderBaseMode {
    pub const ALL: &[ShaderBaseMode] = &[
        Self::Normal,
        Self::SlopeClasses,
        Self::AltitudeBands,
        Self::RawHeightmap,
        Self::BiomeIds,
        Self::SdfCoastal,
        Self::Shoreline,
    ];

    pub fn shader_value(&self) -> f32 {
        match self {
            Self::Normal => 0.0,
            Self::SlopeClasses => 1.0,
            Self::AltitudeBands => 2.0,
            Self::RawHeightmap => 3.0,
            Self::BiomeIds => 4.0,
            Self::SdfCoastal => 5.0,
            Self::Shoreline => 6.0,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Normal => "Normal",
            Self::SlopeClasses => "Slopes",
            Self::AltitudeBands => "Altitude",
            Self::RawHeightmap => "Heightmap",
            Self::BiomeIds => "Biomes",
            Self::SdfCoastal => "SDF Coast",
            Self::Shoreline => "Shoreline",
        }
    }
}

/// Central debug overlay state. Controls all debug visualizations.
#[derive(Resource)]
pub struct DebugOverlayState {
    /// Whether the debug panel UI is visible (F4 toggles).
    pub panel_visible: bool,

    /// Shader base mode (mutually exclusive).
    pub shader_base: ShaderBaseMode,

    // -- Shader overlays (additive on top of base mode) --
    /// Level lines (contour lines at regular height intervals).
    pub level_lines: bool,

    // -- Gizmos overlays (all independent, additive) --
    /// Hex outlines for ShoreType::Shoreline / Lakebank.
    pub hex_shore_types: bool,
    /// Chunk boundary rectangles + labels.
    pub chunk_boundaries: bool,
    /// Organization domain hex outlines (territory cells).
    pub domain_hexes: bool,
    /// Organization voronoi borders.
    pub voronoi_domains: bool,
    /// Mist/exploration voronoi borders.
    pub voronoi_mist: bool,
}

impl Default for DebugOverlayState {
    fn default() -> Self {
        Self {
            panel_visible: false,
            shader_base: ShaderBaseMode::Normal,
            level_lines: false,
            hex_shore_types: false,
            chunk_boundaries: false,
            domain_hexes: false,
            voronoi_domains: false,
            voronoi_mist: false,
        }
    }
}
