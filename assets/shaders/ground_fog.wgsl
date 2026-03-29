// assets/shaders/ground_fog.wgsl
// Low-lying fog visible between trees at the explored/unexplored boundary.
// Rendered BELOW trees, ABOVE terrain.

#import bevy_sprite::mesh2d_vertex_output::VertexOutput
#import bevy_render::globals::Globals

@group(0) @binding(1) var<uniform> globals: Globals;

@group(2) @binding(0) var mist_texture: texture_2d<f32>;
@group(2) @binding(1) var mist_sampler: sampler;
@group(2) @binding(2) var<uniform> params: GroundFogParams;

struct GroundFogParams {
    world_width: f32,
    world_height: f32,
    camera_x: f32,
    camera_y: f32,
}

// ============================================================================
// GRADIENT NOISE
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
    let explored_raw = textureSample(mist_texture, mist_sampler, in.uv).r;

    // Only visible in the transition zone — near the boundary.
    // explored_raw ~0.5 at chunk boundary (bilinear), 0 = deep fog, 1 = explored.
    // Show ground fog where explored_raw is between ~0.3 and 0.98.
    // Peak visibility around the boundary (~0.5–0.7), fades both ways.
    let zone_fade_in  = smoothstep(0.3, 0.55, explored_raw); // ramp up from fog side
    let zone_fade_out = 1.0 - smoothstep(0.85, 0.98, explored_raw); // ramp down into explored
    let zone_mask = zone_fade_in * zone_fade_out;

    if zone_mask < 0.01 {
        discard;
    }

    let time = globals.time;
    let world_pos = in.uv * vec2<f32>(params.world_width, params.world_height);
    let ref_pos = world_pos * (9600.0 / params.world_width);

    // Camera parallax — ground fog moves noticeably with camera
    let cam = vec2<f32>(params.camera_x, params.camera_y) * (9600.0 / params.world_width);
    let parallax = cam * 0.8;

    // ── Animated fog — faster drift than main mist ──
    let drift = time * 0.04; // Visible movement speed
    let p = ref_pos + parallax;

    // Two drifting layers for organic movement
    let fog1 = fbm(p * 0.015 + vec2<f32>(drift * 0.6, drift * 0.3), 4);
    let fog2 = fbm(p * 0.025 + vec2<f32>(-drift * 0.4, drift * 0.5) + vec2<f32>(31.0, 47.0), 3);
    let fog = fog1 * 0.6 + fog2 * 0.4;

    // Shape into wispy patches — not uniform
    let wisps = smoothstep(0.35, 0.65, fog);

    // ── Color — light, slightly cool white ──
    let fog_color = vec3<f32>(0.78, 0.80, 0.82);

    // ── Alpha — low, wispy, zone-masked ──
    let max_alpha = 0.18;
    let alpha = wisps * zone_mask * max_alpha;

    if alpha < 0.01 {
        discard;
    }

    return vec4<f32>(fog_color, alpha);
}
