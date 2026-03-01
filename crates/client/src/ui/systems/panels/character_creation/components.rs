use bevy::prelude::*;

use super::resources::{Gender, LayerCategory};

/// Root marker for the character creation screen.
#[derive(Component)]
pub struct CharacterCreationPanel;

/// Arrow button (prev/next) for a given layer.
#[derive(Component)]
pub struct LayerArrowButton {
    pub category: LayerCategory,
    pub direction: ArrowDirection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArrowDirection {
    Prev,
    Next,
}

/// Counter label text (e.g. "3 / 8") for a layer.
#[derive(Component)]
pub struct LayerCounterText {
    pub category: LayerCategory,
}

/// Preview image node for a specific layer.
#[derive(Component)]
pub struct LayerPreviewImage {
    pub category: LayerCategory,
}

/// Validate (create character) button.
#[derive(Component)]
pub struct ValidateButton;

/// Back button (return to login).
#[derive(Component)]
pub struct BackToLoginButton;

/// First name text input marker.
#[derive(Component)]
pub struct FirstNameInput;

// ─── Gender selection ───────────────────────────────────────────────────────

/// Gender toggle button.
#[derive(Component)]
pub struct GenderButton {
    pub gender: Gender,
}

/// Text/visual for the active gender indicator.
#[derive(Component)]
pub struct GenderActiveIndicator {
    pub gender: Gender,
}

// ─── Random buttons ─────────────────────────────────────────────────────────

/// Randomize all layers + name.
#[derive(Component)]
pub struct RandomAllButton;

/// Randomize first name only.
#[derive(Component)]
pub struct RandomNameButton;
