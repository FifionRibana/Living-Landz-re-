use bevy::prelude::*;

use super::resources::HeraldryLayer;

/// Root marker for the coat of arms creation screen.
#[derive(Component)]
pub struct CoatOfArmsCreationPanel;

/// Arrow button (prev/next) for a given heraldry layer.
#[derive(Component)]
pub struct HeraldryArrowButton {
    pub layer: HeraldryLayer,
    pub direction: HeraldryArrowDirection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeraldryArrowDirection {
    Prev,
    Next,
}

/// Counter label text (e.g. "3 / 8") for a heraldry layer.
#[derive(Component)]
pub struct HeraldryCounterText {
    pub layer: HeraldryLayer,
}

/// Preview image node for a specific heraldry layer.
#[derive(Component)]
pub struct HeraldryPreviewImage {
    pub layer: HeraldryLayer,
}

/// Validate (create coat of arms) button.
#[derive(Component)]
pub struct CoaValidateButton;

/// Back button (return to login).
#[derive(Component)]
pub struct CoaBackButton;

/// Motto text input marker.
#[derive(Component)]
pub struct MottoInput;
