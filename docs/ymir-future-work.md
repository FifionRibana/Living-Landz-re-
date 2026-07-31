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

## 2. Inland water — DONE (LL-D), except per-lake water level

**Resolved.** Ymir now exports `water_class.u8` (0 land / 1 ocean edge-connected /
2 inland enclosed), `lake_mask.u32` (per-cell lake id), `flow_accumulation.f32` and
`lakes.json` (per-lake `level_m`, `shallow`, `lake_type`). The consumer (LL-D) reads
them (`ymir_map.rs`) and derives an **effective inland-water class** (water_class
folded with lake_mask) that drives: the `effective_binary` (only class 1 is ocean →
no more blue spots), the lake SDF source (class 2 → existing lake shader), the
per-cell biome (`resolve_biome`: class 2 → Lake, or Wetland when the lake is
shallow — the previously-no-op rules), and the per-chunk lake detection + Lakebank
shore-type. Azgaar path untouched.

**Ymir-side follow-up:** the hydro-export must also **update `manifest.json`** — set
`present: true` for the hydro layers and add a `water_class` raster layer entry;
otherwise the consumer stays inert (it honours the present flags).

**Remaining (Phase 2, cross-cutting):** the lake pipeline renders a **single flat
water layer** — no per-lake elevation. `lakes.json` `level_m` (and `lake_type`
Endorheic for salt/closed-basin visuals) are loaded but unused. True per-lake
surface elevation / elevation-accurate shorelines needs a change across the shared
`LakeData` type, server generation, the client cache, and both the lake and terrain
shaders (a per-cell lake-level texture). Deferred until the top-down flat render is
insufficient. Also unconsumed: `rivers.json` (river rendering / navigability).

---

_Referenced commits: `bc9633d` (sea-level calibration), `64b281a` (ocean),
`8d28819` (biome blur). Integration branch: `229-ll-c-...`._
