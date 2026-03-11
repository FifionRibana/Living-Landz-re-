use std::sync::Arc;

/// Server-side development configuration.
/// When dev_mode is false, all gameplay behaves normally.
#[derive(Debug, Clone)]
pub struct DevConfig {
    /// Whether dev mode is active
    pub dev_mode: bool,
    /// Duration divisor: all action durations are divided by this factor.
    /// 1 = normal, 10 = 10x faster, 1000 = near-instant
    pub speed_factor: u64,
    /// Skip resource validation (allow crafting/building without ingredients)
    pub bypass_resources: bool,
}

impl DevConfig {
    /// Load from environment variables.
    /// DEV_MODE=true enables dev mode
    /// DEV_SPEED_FACTOR=10 (default: 10 when dev mode, 1 otherwise)
    /// DEV_BYPASS_RESOURCES=true (default: same as DEV_MODE)
    pub fn from_env() -> Self {
        let dev_mode = std::env::var("DEV_MODE")
            .map(|v| v == "true" || v == "1")
            .unwrap_or(false);

        let speed_factor = if dev_mode {
            std::env::var("DEV_SPEED_FACTOR")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(10)
        } else {
            1
        };

        let bypass_resources = if dev_mode {
            std::env::var("DEV_BYPASS_RESOURCES")
                .map(|v| v == "true" || v == "1")
                .unwrap_or(true) // default: bypass when dev mode
        } else {
            false
        };

        Self {
            dev_mode,
            speed_factor,
            bypass_resources,
        }
    }

    /// Apply speed factor to a duration in milliseconds.
    pub fn apply_speed(&self, duration_ms: u64) -> u64 {
        if self.speed_factor <= 1 {
            duration_ms
        } else {
            (duration_ms / self.speed_factor).max(500) // minimum 500ms
        }
    }

    /// Whether resource checks should be skipped.
    pub fn skip_resource_check(&self) -> bool {
        self.bypass_resources
    }
}