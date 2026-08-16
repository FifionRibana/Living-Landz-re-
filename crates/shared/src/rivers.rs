//! Ymir river-network reader (LL-E).
//!
//! Parses `rivers.json` (a directed river graph, `coordinate_space =
//! "erosion_grid_cells"`) into a mirror [`RiverNetwork`]. Lives in `shared` so
//! both the server (gameplay navigability / basin connectivity) and the client
//! (river rendering + debug overlay) reuse one definition. Coordinates are kept
//! verbatim in Ymir cell space (y=0 = south); the caller applies the cell→world
//! transform + Y-flip. Navigability is **consumed as authored by Ymir**, never
//! re-derived from Strahler order / flow.

use serde::Deserialize;

/// Per-segment navigability, exactly as Ymir authors it (do not recompute).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
pub enum RiverNavigability {
    #[default]
    NonNavigable,
    SmallBoat,
    Barge,
    Ship,
}

impl RiverNavigability {
    /// True for any class a vessel can traverse (SmallBoat and larger).
    #[inline]
    pub fn is_navigable(self) -> bool {
        !matches!(self, RiverNavigability::NonNavigable)
    }
}

/// One directed river reach (a run between confluences).
#[derive(Debug, Clone, Deserialize)]
pub struct RiverSegment {
    /// Polyline in Ymir cell space (`[x, y]`, y=0 = south).
    #[serde(default)]
    pub points: Vec<[f32; 2]>,
    /// Strahler order (1 = headwater; grows at confluences). Drives render width.
    #[serde(default)]
    pub strahler_order: u32,
    #[serde(default)]
    pub avg_flow: f32,
    #[serde(default)]
    pub max_flow: f32,
    #[serde(default)]
    pub basin_id: u32,
    /// Segment indices flowing into this one.
    #[serde(default)]
    pub upstream: Vec<u32>,
    /// The segment this flows into, or `None` at a sink (mouth / endorheic end).
    #[serde(default)]
    pub downstream: Option<u32>,
    #[serde(default)]
    pub drainage_km2: f32,
    /// Ymir-authored navigability class (authoritative — consumed, not recomputed).
    #[serde(default)]
    pub navigability: RiverNavigability,
}

/// The parsed river graph, index-aligned (`upstream`/`downstream` index into
/// `segments`).
#[derive(Debug, Clone, Default, Deserialize)]
pub struct RiverNetwork {
    #[serde(default)]
    pub coordinate_space: String,
    #[serde(default)]
    pub segments: Vec<RiverSegment>,
}

impl RiverNetwork {
    /// Parse `rivers.json`. Returns an empty network on malformed input (callers
    /// treat empty as "no rivers" rather than an error), mirroring the GeoJSON
    /// reader's contract.
    pub fn parse(raw: &str) -> Self {
        serde_json::from_str(raw).unwrap_or_default()
    }

    /// Walk `downstream` from `seg` to its terminal segment (the sink), so callers
    /// can classify the sink (sea vs lake) via `water_class`/`lake_mask`. Guards
    /// against out-of-range links and cycles. Returns the sink segment index, or
    /// `None` if `seg` is out of range.
    pub fn sink_of(&self, seg: usize) -> Option<usize> {
        let n = self.segments.len();
        let mut cur = seg;
        for _ in 0..=n {
            let s = self.segments.get(cur)?;
            match s.downstream {
                Some(next) => {
                    let next = next as usize;
                    if next >= n || next == cur {
                        return Some(cur); // dangling/self link → treat cur as sink
                    }
                    cur = next;
                }
                None => return Some(cur),
            }
        }
        Some(cur) // cycle guard: bail out after n hops
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_and_walks_downstream() {
        let raw = r#"{"coordinate_space":"erosion_grid_cells","segments":[
            {"points":[[0,0],[1,1]],"strahler_order":1,"avg_flow":10.0,"max_flow":20.0,
             "basin_id":1,"upstream":[],"downstream":1,"drainage_km2":5.0,"navigability":"NonNavigable"},
            {"points":[[1,1],[2,2]],"strahler_order":3,"avg_flow":30.0,"max_flow":40.0,
             "basin_id":1,"upstream":[0],"downstream":null,"drainage_km2":15.0,"navigability":"SmallBoat"}
        ]}"#;
        let net = RiverNetwork::parse(raw);
        assert_eq!(net.segments.len(), 2);
        assert_eq!(net.segments[0].strahler_order, 1);
        assert_eq!(net.segments[1].strahler_order, 3);
        assert_eq!(net.segments[1].navigability, RiverNavigability::SmallBoat);
        assert!(net.segments[1].navigability.is_navigable());
        assert!(!net.segments[0].navigability.is_navigable());
        // downstream walk reaches the sink (segment 1) from either start.
        assert_eq!(net.sink_of(0), Some(1));
        assert_eq!(net.sink_of(1), Some(1));
    }

    #[test]
    fn empty_on_garbage() {
        assert!(RiverNetwork::parse("not json").segments.is_empty());
    }
}
