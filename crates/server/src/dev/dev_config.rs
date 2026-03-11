use std::sync::Arc;

/// Server-side development configuration.
/// When dev_mode is false, all gameplay behaves normally.
#[derive(Debug, Clone)]
pub struct DevConfig {
    /// Whether dev mode is active
    pub dev_mode: bool,
    /// Skip resource validation (allow crafting/building without ingredients)
    pub bypass_resources: bool,
}

impl DevConfig {
    /// Load from environment variables.
    /// DEV_MODE=true enables dev mode
    /// DEV_BYPASS_RESOURCES=true (default: same as DEV_MODE)
    pub fn from_env() -> Self {
        let dev_mode = std::env::var("DEV_MODE")
            .map(|v| v == "true" || v == "1")
            .unwrap_or(false);

        let bypass_resources = if dev_mode {
            std::env::var("DEV_BYPASS_RESOURCES")
                .map(|v| v == "true" || v == "1")
                .unwrap_or(true) // default: bypass when dev mode
        } else {
            false
        };

        Self {
            dev_mode,
            bypass_resources,
        }
    }

    /// Whether resource checks should be skipped.
    pub fn skip_resource_check(&self) -> bool {
        self.bypass_resources
    }
}