// assets/shaders/mist.wgsl
// Issue #115 — Dynamic mist animated texture
// Adds painterly animated texture to the original mist logic.

#import bevy_sprite::mesh2d_vertex_output::VertexOutput
#import bevy_render::globals::Globals

@group(0) @binding(1) var<uniform> globals: Globals;

@group(2) @binding(0) var mist_texture: texture_2d<f32>;
@group(2) @binding(1) var mist_sampler: sampler;
@group(2) @binding(2) var<uniform> mist_params: MistParams;
@group(2) @binding(3) var sdf_texture: texture_2d<f32>;
@group(2) @binding(4) var sdf_sampler: sampler;

struct MistParams {
    world_width: f32,
    world_height: f32,
    camera_x: f32,
    camera_y: f32,
}

// ============================================================================
// GRADIENT NOISE — Perlin-style, no grid artifacts
// ============================================================================

fn hash_grad(p: vec2<f32>) -> vec2<f32> {
    let a = dot(p, vec2<f32>(127.1, 311.7));
    let b = dot(p, vec2<f32>(269.5, 183.3));
    let h = sin(vec2<f32>(a, b)) * 43758.5453;
    let angle = fract(h) * 6.28318530718;
    return vec2<f32>(cos(angle.x), sin(angle.y));
}

fn gnoise(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let g00 = hash_grad(i);
    let g10 = hash_grad(i + vec2<f32>(1.0, 0.0));
    let g01 = hash_grad(i + vec2<f32>(0.0, 1.0));
    let g11 = hash_grad(i + vec2<f32>(1.0, 1.0));
    let v00 = dot(g00, f);
    let v10 = dot(g10, f - vec2<f32>(1.0, 0.0));
    let v01 = dot(g01, f - vec2<f32>(0.0, 1.0));
    let v11 = dot(g11, f - vec2<f32>(1.0, 1.0));
    let u = f * f * f * (f * (f * 6.0 - 15.0) + 10.0);
    return mix(mix(v00, v10, u.x), mix(v01, v11, u.x), u.y) * 0.5 + 0.5;
}

fn fbm(p: vec2<f32>, octaves: i32) -> f32 {
    var value = 0.0;
    var amp = 0.5;
    var pos = p;
    let rot = mat2x2<f32>(0.8, 0.6, -0.6, 0.8);
    for (var i = 0; i < octaves; i++) {
        value += amp * gnoise(pos);
        pos = rot * pos * 2.0;
        amp *= 0.5;
    }
    return value;
}

// ============================================================================
// FRAGMENT
// ============================================================================

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // ── Original exploration logic, untouched ──
    let explored_raw = textureSample(mist_texture, mist_sampler, in.uv).r;
    let explored = smoothstep(0.7, 0.98, explored_raw);
    let mist_alpha = 1.0 - explored;

    if mist_alpha < 0.01 {
        discard;
    }

    // ── Coast masking ──
    let sdf_raw = textureSample(sdf_texture, sdf_sampler, in.uv).r;
    let sdf_signed = (sdf_raw - 0.5) * 2.0;
    let coast_mask = smoothstep(-0.02, 0.15, sdf_signed);
    if coast_mask < 0.01 {
        discard;
    }

    let time = globals.time;
    let world_pos = in.uv * vec2<f32>(mist_params.world_width, mist_params.world_height);
    let ref_pos = world_pos * (9600.0 / mist_params.world_width);

    // ── Parallax — each layer shifts at a different rate with camera ──
    // Higher layers (large shapes) shift less → appear far away.
    // Lower layers (fine detail) shift more → appear close/low.
    let cam = vec2<f32>(mist_params.camera_x, mist_params.camera_y) * (9600.0 / mist_params.world_width);
    let parallax_high = cam * 0.3;    // Large distant clouds — slow shift
    let parallax_mid  = cam * 0.7;    // Mid-level fog
    let parallax_low  = cam * 1.2;    // Close wisps — fast shift

    // ── Animated fog texture ──
    let t1 = time * 0.006;
    let n1 = fbm((ref_pos + parallax_high) * 0.004 + vec2<f32>(t1 * 0.3, -t1 * 0.2), 6);
    let n2 = fbm((ref_pos + parallax_mid) * 0.009 + vec2<f32>(-t1 * 0.4, t1 * 0.35) + vec2<f32>(42.0, 17.0), 5);
    let n3 = fbm((ref_pos + parallax_low) * 0.025 + vec2<f32>(time * 0.012, -time * 0.009), 4);
    let n4 = fbm((ref_pos + parallax_low) * 0.06 + vec2<f32>(-time * 0.015, time * 0.011), 3);
    let density = n1 * 0.4 + n2 * 0.3 + n3 * 0.2 + n4 * 0.1;
    let shaped = smoothstep(0.25, 0.75, density);

    // ── Color ──
    let fog_dark  = vec3<f32>(0.52, 0.54, 0.55);
    let fog_light = vec3<f32>(0.82, 0.83, 0.82);
    var fog_color = mix(fog_dark, fog_light, shaped);

    // Subtle hue drift (also parallaxed at mid rate)
    let hue_n = fbm((ref_pos + parallax_mid) * 0.003 + vec2<f32>(time * 0.002, -time * 0.0015), 3);
    fog_color += vec3<f32>(
        (hue_n - 0.5) * -0.02,
        (hue_n - 0.5) *  0.01,
        (hue_n - 0.5) *  0.025
    );

    // ── Wisp modulation in the transition zone ──
    let wt = time * 0.015;
    let w1 = fbm((ref_pos + parallax_low) * 0.012 + vec2<f32>(wt * 0.3, wt * 0.2), 5);
    let w2 = fbm((ref_pos + parallax_low) * 0.02 + vec2<f32>(-wt * 0.25, wt * 0.3) + vec2<f32>(53.0, 29.0), 4);
    let wisp = w1 * 0.6 + w2 * 0.4;
    let wisp_mask = smoothstep(0.3, 0.6, wisp);

    // Wisps only affect the transition zone (where mist_alpha < 1)
    let in_transition = 1.0 - mist_alpha; // 0 in core, 1 at fully explored
    let wisp_effect = in_transition * (0.5 + wisp_mask * 0.5);
    let final_alpha = mist_alpha * (1.0 - wisp_effect * 0.4);

    let alpha = final_alpha * coast_mask;

    if alpha < 0.01 {
        discard;
    }

    // Lighten fog at the edges
    fog_color = mix(fog_color, fog_light, in_transition * 0.3);

    return vec4<f32>(fog_color, alpha);
}