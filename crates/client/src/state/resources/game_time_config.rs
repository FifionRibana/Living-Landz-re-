use bevy::prelude::*;

#[derive(Resource)]
pub struct GameTimeConfig {
    pub months: Vec<String>,
    pub hours_offset: i32,
}

impl Default for GameTimeConfig {
    fn default() -> Self {
        Self {
            months: vec![
                "January",
                "February",
                "March",
                "April",
                "May",
                "June",
                "July",
                "August",
                "September",
                "October",
                "November",
                "December",
            ]
            .iter()
            .map(|s| s.to_string())
            .collect(),
            hours_offset: 0,
        }
    }
}
