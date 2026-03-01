use bevy::prelude::*;

use crate::states::AppState;

use super::components::*;
use super::resources::CoatOfArmsCreationState;

// ─── Palette (hover colors) ─────────────────────────────────────────────────

const ARROW_BG: Color = Color::srgba(0.79, 0.66, 0.30, 0.10);
const ARROW_HOVER: Color = Color::srgba(0.79, 0.66, 0.30, 0.25);
const BTN_BG: Color = Color::srgb(0.23, 0.18, 0.12);
const BTN_HOVER: Color = Color::srgb(0.30, 0.24, 0.16);
const BTN_PRIMARY_BG: Color = Color::srgba(0.79, 0.66, 0.30, 0.20);
const BTN_PRIMARY_HOVER: Color = Color::srgba(0.79, 0.66, 0.30, 0.35);

// ─── Arrow button clicks ────────────────────────────────────────────────────

pub fn handle_heraldry_arrow_clicks(
    query: Query<(&Interaction, &HeraldryArrowButton), Changed<Interaction>>,
    mut creation_state: ResMut<CoatOfArmsCreationState>,
) {
    for (interaction, arrow) in query.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }

        if let Some(layer_state) = creation_state.layer_mut(arrow.layer) {
            match arrow.direction {
                HeraldryArrowDirection::Prev => layer_state.prev(),
                HeraldryArrowDirection::Next => layer_state.next(),
            }
            info!(
                "Heraldry layer {:?} changed to {} / {}",
                arrow.layer,
                layer_state.current + 1,
                layer_state.total
            );
        }
    }
}

// ─── Arrow hover ────────────────────────────────────────────────────────────

pub fn update_heraldry_arrow_hover(
    mut query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<HeraldryArrowButton>),
    >,
) {
    for (interaction, mut bg) in query.iter_mut() {
        *bg = match interaction {
            Interaction::Hovered | Interaction::Pressed => BackgroundColor(ARROW_HOVER),
            Interaction::None => BackgroundColor(ARROW_BG),
        };
    }
}

// ─── Back button ────────────────────────────────────────────────────────────

pub fn handle_coa_back_click(
    query: Query<&Interaction, (Changed<Interaction>, With<CoaBackButton>)>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for interaction in query.iter() {
        if *interaction == Interaction::Pressed {
            info!("Back to login from coat of arms creation");
            next_state.set(AppState::Login);
        }
    }
}

pub fn update_coa_back_hover(
    mut query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<CoaBackButton>),
    >,
) {
    for (interaction, mut bg) in query.iter_mut() {
        *bg = match interaction {
            Interaction::Hovered | Interaction::Pressed => BackgroundColor(BTN_HOVER),
            Interaction::None => BackgroundColor(BTN_BG),
        };
    }
}

// ─── Validate button ────────────────────────────────────────────────────────

pub fn handle_coa_validate_click(
    query: Query<&Interaction, (Changed<Interaction>, With<CoaValidateButton>)>,
    creation_state: Res<CoatOfArmsCreationState>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for interaction in query.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }

        info!("Validating coat of arms creation");

        let selections: Vec<(String, usize)> = creation_state
            .layers
            .iter()
            .filter(|l| l.layer.is_available())
            .map(|l| (l.layer.folder().to_string(), l.current))
            .collect();

        let motto = creation_state.motto.trim().to_string();
        info!(
            "Coat of arms selections: {:?}, motto: '{}'",
            selections, motto
        );

        // TODO: Send CreateCoatOfArms message to server when protocol is ready
        // For now, go back to login
        next_state.set(AppState::Login);
    }
}

pub fn update_coa_validate_hover(
    mut query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<CoaValidateButton>),
    >,
) {
    for (interaction, mut bg) in query.iter_mut() {
        *bg = match interaction {
            Interaction::Hovered | Interaction::Pressed => BackgroundColor(BTN_PRIMARY_HOVER),
            Interaction::None => BackgroundColor(BTN_PRIMARY_BG),
        };
    }
}

// ─── Sync motto input → resource ────────────────────────────────────────────

pub fn sync_motto_input(
    query: Query<&bevy_ui_text_input::TextInputBuffer, With<MottoInput>>,
    mut creation_state: ResMut<CoatOfArmsCreationState>,
) {
    for buffer in query.iter() {
        let text = buffer.get_text().to_string();
        if text != creation_state.motto {
            creation_state.motto = text;
        }
    }
}
