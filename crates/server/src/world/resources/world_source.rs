//! World-source seam (LL-A): selects between the legacy Azgaar PNG bundle and a
//! native Ymir `.ymir` continent.
//!
//! Selection precedence (highest first):
//!   1. `--world-source=<azgaar|ymir>` CLI flag (passed in as `cli_flag`)
//!   2. `WORLD_SOURCE` env var (`azgaar` | `ymir`)
//!   3. default: Azgaar
//!
//! The Azgaar path (`WorldMaps`) is kept fully intact and reachable so the two
//! sources can be A/B-compared during validation.

use super::{WorldMaps, YmirMap};

/// Which world source to load. Cheap, `Copy`, resolved once at startup.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorldSourceKind {
    Azgaar,
    Ymir,
}

impl WorldSourceKind {
    /// Parse a source name (case-insensitive). Accepts `azgaar`/`png` and
    /// `ymir`. Returns `None` for anything else.
    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "azgaar" | "png" => Some(Self::Azgaar),
            "ymir" => Some(Self::Ymir),
            _ => None,
        }
    }

    /// Resolve the effective source from (in order): the CLI flag, the
    /// `WORLD_SOURCE` env var, then the default (Azgaar). Unknown values log a
    /// warning and fall through to the next tier.
    pub fn resolve(cli_flag: Option<&str>) -> Self {
        if let Some(flag) = cli_flag {
            match Self::parse(flag) {
                Some(kind) => return kind,
                None => tracing::warn!(
                    "Unknown --world-source='{flag}', ignoring (expected azgaar|ymir)"
                ),
            }
        }
        if let Ok(env) = std::env::var("WORLD_SOURCE") {
            match Self::parse(&env) {
                Some(kind) => return kind,
                None => tracing::warn!(
                    "Unknown WORLD_SOURCE='{env}', ignoring (expected azgaar|ymir)"
                ),
            }
        }
        Self::Azgaar
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Azgaar => "azgaar",
            Self::Ymir => "ymir",
        }
    }
}

/// A loaded world source. Holds the source data for the chosen backend.
pub enum WorldSource {
    AzgaarPng(WorldMaps),
    Ymir(YmirMap),
}

impl WorldSource {
    /// Load the source for `map_name` according to `kind`.
    pub fn load(
        kind: WorldSourceKind,
        map_name: &str,
        seed: u32,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        match kind {
            WorldSourceKind::Azgaar => Ok(Self::AzgaarPng(WorldMaps::load(map_name, seed)?)),
            WorldSourceKind::Ymir => Ok(Self::Ymir(YmirMap::load(map_name)?)),
        }
    }
}
