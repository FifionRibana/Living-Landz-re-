# Ymir-side future work

Items that surfaced while integrating the native `.ymir` map path (LL-A/B/C) but
are best solved **in the Ymir generator** (producing richer layers) rather than
worked around in the game client/server. Each notes the current stopgap and what
Ymir should provide instead.

---

## 1. Biome transitions (distance-based blend factor)

**Context.** The client renders biomes from `biome.u8` (Whittaker ids → palettes).
The terrain biome texture is `RGBA8: R = primary id, G = secondary id, B = blend
factor`. On the Azgaar path the server computes `G`/`B` from distance to biome
boundaries (a triangulation), so the shader blends smoothly. On the Ymir path
`build_ymir_globals` writes `R = id`, `G = id`, `B = 0` — no blend data.

**Current stopgap (client shader).** `sample_vegetation_with_biome_blend` in
`assets/shaders/terrain_signed_sdf_painterly.wgsl` now always runs a jittered
multi-sample vegetation blur on the Ymir path (commit `8d28819`), averaging full
painterly samples so the texture stays crisp while base colours blend. It removes
the hard cell edges but is only a uniform spatial blur:
- the biome field is genuinely **high-frequency (per cell)**, so a wide blur muds
  colours and a narrow one still looks blocky — there is no good uniform radius;
- it costs ~9 painterly evaluations per land fragment.

**What Ymir should provide.** A proper transition needs per-cell blend data, i.e.
the same signal the Azgaar triangulation produces, computed at the source:
- a **secondary biome id** and a **blend factor** per cell (distance to the nearest
  differing-biome boundary, normalized), OR
- a smoother/antialiased biome field (e.g. sub-cell biome coverage / a low-res
  "biome weight" layer) the client can sample and interpolate.

With that, the server would fill `G`/`B` on the Ymir path exactly like Azgaar, the
client shader would take its cheap B-driven path, and transitions would be
distance-based (correct) instead of a blur (approximate).

---

## 2. Inland below-sea cells — coastline flood-fill / lakes

**Context.** Sea level is calibrated to the vector coastline (norm ≈ 0.574; commit
`bc9633d`), so land/sea is consistent across mesh, ocean SDF and biome. But cells
that are **below sea level yet enclosed by land** (inland depressions) are classed
as Ocean and rendered blue, even though the ocean shader (which follows the
coastline) doesn't reach them — the residual inland "blue spots".

**What's needed.** Distinguish ocean-connected water from enclosed water:
- a **flood-fill from the map border**: below-sea cells reachable from the edge =
  Ocean; enclosed below-sea = Lake (then the lake shader / terrain lake-bank
  handles them). This could run server-side, but is cleaner as a Ymir layer;
- ideally Ymir ships the already-present-but-empty **`lake_mask`** layer
  (`present: false` in current maps), which removes the guesswork entirely and
  also unlocks the Lake/Wetland biome rules that are currently no-ops
  (`crates/server/src/world/components/ymir_biome.rs`).

---

_Referenced commits: `bc9633d` (sea-level calibration), `64b281a` (ocean),
`8d28819` (biome blur). Integration branch: `229-ll-c-...`._
