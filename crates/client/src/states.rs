// =============================================================================
// STATES - Application state machine
// =============================================================================
//
// Hierarchy:
//
// AppState (top-level)
// ├── Login
// │   └── AuthScreen (SubState)
// │       ├── Login (default)
// │       └── Register
// ├── CharacterCreation
// ├── CoatOfArmsCreation
// └── InGame
//     ├── GameView (SubState) ← active view
//     │   ├── Map (default)
//     │   ├── Cell
//     │   ├── CityManagement
//     │   ├── Messages
//     │   ├── Rankings
//     │   ├── Calendar
//     │   ├── Records
//     │   ├── Search
//     │   └── Settings
//     │
//     └── Overlay (SubState) ← layered on top
//         ├── None (default)
//         ├── PauseMenu
//         └── Settings

use bevy::prelude::*;

/// Top-level application state
#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum AppState {
    #[default]
    Login,
    CharacterCreation,
    CoatOfArmsCreation,
    InGame,
}

/// Authentication sub-screen — only exists when AppState::Login
#[derive(SubStates, Default, Debug, Clone, PartialEq, Eq, Hash)]
#[source(AppState = AppState::Login)]
pub enum AuthScreen {
    #[default]
    Login,
    Register,
}

/// Main game view — only exists when AppState::InGame
#[derive(SubStates, Default, Debug, Clone, PartialEq, Eq, Hash)]
#[source(AppState = AppState::InGame)]
pub enum GameView {
    #[default]
    Map,
    Cell,
    CityManagement,
    Messages,
    Rankings,
    Calendar,
    Records,
    Search,
    Settings,
}

/// Overlay (pause menu, etc.) — orthogonal to GameView, only in InGame
#[derive(SubStates, Default, Debug, Clone, PartialEq, Eq, Hash)]
#[source(AppState = AppState::InGame)]
pub enum Overlay {
    #[default]
    None,
    PauseMenu,
}
