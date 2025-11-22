use bevy::prelude::*;
use chrono::{Datelike, Local, Timelike};
use shared::atlas::MoonAtlas;

use crate::state::resources::GameTimeConfig;
use crate::ui::components::{ClockText, DateText, MoonPhaseImage, MoonText};

pub fn update_clock(
    mut query: Query<(
        &mut Text,
        Option<&ClockText>,
        Option<&DateText>,
        Option<&MoonText>,
    )>,
    game_time_config: Res<GameTimeConfig>,
) {
    let now = Local::now();

    let hours = ((now.hour() as i32 + game_time_config.hours_offset) % 24) as u32;
    let minutes = now.minute();
    let seconds = now.second();

    let day = format_day_ordinal(now.day());
    let month = &game_time_config.months[(now.month() - 1) as usize];

    for (mut text, clock_query, date_query, moon_query) in &mut query {
        if clock_query.is_some() {
            **text = format!("{:02}:{:02}:{:02}", hours, minutes, seconds);
        } else if date_query.is_some() {
            **text = format!("{} {}", month, day);
        } else if moon_query.is_some() {
            let (phase, moon_name) = get_lunar_phase();
            **text = format!("{} ({:.0}%)", moon_name, phase * 100.0);
        }
    }
}

fn get_lunar_phase() -> (f32, String) {
    let now = Local::now();

    // Nombre de jours depuis la nouvelle lune de référence (2000-01-06)
    let known_new_moon = chrono::NaiveDate::from_ymd_opt(2000, 1, 6).unwrap();
    let current_date = now.date_naive();
    let days_since = (current_date - known_new_moon).num_days() as f32;

    // Cycle lunaire: ~29.53 jours
    let lunar_cycle = 29.53;
    let phase = (days_since % lunar_cycle) / lunar_cycle; // 0.0 - 1.0

    // Nom de la phase (8 phases)
    let phase_name = match (phase * 8.0) as u32 {
        0 => "New moon",        //"Nouvelle lune",
        1 => "Waxing crescent", //"Croissant",
        2 => "First quarter",   //"Premier quartier",
        3 => "Waxing gibbous",  //"Gibbeuse croissante",
        4 => "Full moon",       //"Pleine lune",
        5 => "Waning gibbous",  //"Gibbeuse décroissante",
        6 => "Last quarter",    //"Dernier quartier",
        7 => "Waning crescent", //"Croissant décroissant",
        _ => "New moon",        //"Nouvelle lune",
    };

    (phase, phase_name.to_string())
}

pub fn get_lunar_phase_index() -> u32 {
    let now = Local::now();

    // Nombre de jours depuis la nouvelle lune de référence (2000-01-06)
    let known_new_moon = chrono::NaiveDate::from_ymd_opt(2000, 1, 6).unwrap();
    let current_date = now.date_naive();
    let days_since = (current_date - known_new_moon).num_days() as f32;

    // Cycle lunaire: ~29.53 jours
    let lunar_cycle = 29.53;
    let phase = (days_since % lunar_cycle) / lunar_cycle; // 0.0 - 1.0

    // Retourne l'index de la phase (0-7)
    (phase * 8.0) as u32 % 8
}

pub fn update_moon_phase_image(
    mut query: Query<&mut ImageNode, With<MoonPhaseImage>>,
    moon_atlas: Res<MoonAtlas>,
) {
    let phase_index = get_lunar_phase_index();

    for mut image in &mut query {
        if let Some(moon_handle) = moon_atlas.get_handle(phase_index) {
            image.image = moon_handle.clone();
        }
    }
}

fn format_day_ordinal(day: u32) -> String {
    let suffix = match day {
        1 | 21 | 31 => "st",
        2 | 22 => "nd",
        3 | 23 => "rd",
        _ => "th",
    };
    format!("{}{}", day, suffix)
}