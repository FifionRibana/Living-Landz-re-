// assets/shaders/ground_fog.wgsl
// Low-lying fog visible between trees at the explored/unexplored boundary.
// Rendered BELOW trees, ABOVE terrain.
// Harmonized with mist.wgsl — shares warp function and tentacle distortion.

#import bevy_sprite::mesh2d_vertex_output::VertexOutput
#import bevy_render::globals::Globals

@group(0) @binding(1) var<uniform> globals: Globals;

@group(2) @binding(0) var mist_texture: texture_2d<f32>;
@group(2) @binding(1) var mist_sampler: sampler;
@group(2) @binding(2) var<uniform> params: GroundFogParams;
@group(2) @binding(3) var sdf_texture: texture_2d<f32>;
@group(2) @binding(4) var sdf_sampler: sampler;

struct GroundFogParams {
    world_width: f32,
    world_height: f32,
    camera_x: f32,
    camera_y: f32,
}

// ============================================================================
// NOISE TOOLKIT (same as mist.wgsl)
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

fn warp(p: vec2<f32>, t: f32) -> vec2<f32> {
    let q = vec2<f32>(
        fbm(p + vec2<f32>(t * 0.8, -t * 0.6), 3),
        fbm(p + vec2<f32>(5.2, 1.3) + vec2<f32>(-t * 0.6, t * 0.8), 3)
    );
    return p + q * 3.5;
}

// ============================================================================
// FRAGMENT
// ============================================================================

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let time = globals.time;
    let world_pos = in.uv * vec2<f32>(params.world_width, params.world_height);

    // --- 1. BOUNDARY — same tentacle distortion as mist ---
    let tentacle_scale = 0.012;
    let t_edge = time * 0.08;
    let edge_coords = warp(world_pos * tentacle_scale, t_edge * 0.5);

    let dx = fbm(edge_coords + vec2<f32>(t_edge, 0.0), 3) - 0.5;
    let dy = fbm(edge_coords + vec2<f32>(0.0, t_edge), 3) - 0.5;

    let uv_scale = vec2<f32>(1.0 / params.world_width, 1.0 / params.world_height);
    let distortion = vec2<f32>(dx, dy) * 180.0 * uv_scale;
    let distorted_uv = in.uv + distortion;

    let explored_raw = textureSample(mist_texture, mist_sampler, distorted_uv).r;

    // Breathing — same breath_mask as mist
    let bp = world_pos * 0.01;
    let breath = sin(time * 0.1 + bp.x) * 0.5 + cos(time * 0.1 + bp.y) * 0.5;
    let breath_mask = smoothstep(0.0, 0.1, explored_raw);

    // Negative bias: ground fog creeps further INTO explored zone
    let explored_animated = explored_raw + (breath * 0.06 * breath_mask) - 0.08;

    // Variable softness — coherent with mist boundary shape
    let edge_variance = fbm(world_pos * 0.02 + vec2<f32>(time * 0.05, 0.0), 3);
    let softness = mix(0.4, 1.4, smoothstep(0.3, 0.7, edge_variance));

    // Zone mask: ground fog is a BAND around the boundary
    let fade_val = clamp(smoothstep(0.0, softness, max(0.0, explored_animated)), 0.0, 1.0);
    let zone_fade_in  = smoothstep(0.0, 0.3, fade_val);
    let zone_fade_out = 1.0 - smoothstep(0.6, 1.0, fade_val);
    let zone_mask = zone_fade_in * zone_fade_out;

    let edge_band = zone_fade_in * (1.0 - smoothstep(0.25, 0.65, fade_val));

    if zone_mask < 0.01 {
        discard;
    }

    // --- COAST MASKING — same distortion amplitude, wide transition ---
    let coast_distortion = vec2<f32>(dx, dy) * 180.0 * uv_scale;
    let coast_uv = in.uv + coast_distortion;

    let sdf_raw = textureSample(sdf_texture, sdf_sampler, coast_uv).r;
    let sdf_signed = (sdf_raw - 0.5) * 2.0;
    let coast_animated = sdf_signed + (breath * 0.05);
    let coast_mask = smoothstep(-0.08, 0.25, coast_animated);

    if coast_mask < 0.01 {
        discard;
    }

    // --- 2. GROUND FOG TEXTURE — domain-warped tendrils ---
    let ref_pos = world_pos * (9600.0 / params.world_width);
    let cam = vec2<f32>(params.camera_x, params.camera_y) * (9600.0 / params.world_width);

    let t_fluid = time * 0.003;
    let fog_p = ref_pos + cam * 0.1;

    let warped_fog = warp(fog_p * 0.018, t_fluid);
    let fog1 = fbm(warped_fog, 4);
    let fog2 = fbm(warped_fog * 1.5 + vec2<f32>(31.0, 47.0) + vec2<f32>(time * 0.008, -time * 0.005), 3);
    let fog = fog1 * 0.6 + fog2 * 0.4;

    let wisps = smoothstep(0.30, 0.70, fog);

    // --- 3. COLOR — cold, bruised, violet contamination ---
    let gf_light  = vec3<f32>(0.72, 0.73, 0.78);
    let gf_dark   = vec3<f32>(0.45, 0.42, 0.50);
    let gf_violet = vec3<f32>(0.50, 0.35, 0.58);

    var fog_color = mix(gf_light, gf_dark, edge_band * 0.5);

    let violet_noise = fbm(warped_fog * 1.3 + vec2<f32>(time * 0.01, 0.0), 3);
    let violet_mask = smoothstep(0.4, 0.7, violet_noise) * edge_band;
    fog_color = mix(fog_color, gf_violet, violet_mask * 0.25);

    let hue_n = fbm(fog_p * 0.004 + vec2<f32>(time * 0.002, -time * 0.0015), 3);
    fog_color += vec3<f32>(
        (hue_n - 0.5) * 0.03,
        (hue_n - 0.5) * -0.015,
        (hue_n - 0.5) * -0.02
    );

    // --- 4. ALPHA — low, wispy, zone-masked, coast-masked ---
    let max_alpha = 0.22;
    let alpha = wisps * zone_mask * coast_mask * max_alpha;

    if alpha < 0.01 {
        discard;
    }

    return vec4<f32>(fog_color, alpha);
}
