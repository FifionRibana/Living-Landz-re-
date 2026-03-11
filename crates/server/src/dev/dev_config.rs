use std::sync::Arc;

/// Server-side development configuration.
/// When dev_mode is false, all gameplay behaves normally.
#[derive(Debug, Clone)]
pub struct DevConfig {
    /// Whether dev mode is active
    pub dev_mode: bool,
}

impl DevConfig {
    /// Load from environment variables.
    /// DEV_MODE=true enables dev mode
    pub fn from_env() -> Self {
        let dev_mode = std::env::var("DEV_MODE")
            .map(|v| v == "true" || v == "1")
            .unwrap_or(false);

        Self {
            dev_mode,
        }
    }
}