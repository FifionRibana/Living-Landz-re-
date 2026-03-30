// assets/shaders/mist.wgsl
// Issue #115 — Dynamic mist animated texture
// Issue #116 — Voronoi exploration boundary + threatening atmosphere

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
// NOISE TOOLKIT
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
    let world_pos = in.uv * vec2<f32>(mist_params.world_width, mist_params.world_height);

    // --- 1. FRONTIÈRE VIVANTE ET TENTACULAIRE ---
    let tentacle_scale = 0.012; 
    let t_edge = time * 0.08;
    let edge_coords = warp(world_pos * tentacle_scale, t_edge * 0.5);
    
    let dx = fbm(edge_coords + vec2<f32>(t_edge, 0.0), 3) - 0.5;
    let dy = fbm(edge_coords + vec2<f32>(0.0, t_edge), 3) - 0.5;

    let uv_scale = vec2<f32>(1.0 / mist_params.world_width, 1.0 / mist_params.world_height);
    
    let distortion = vec2<f32>(dx, dy) * 180.0 * uv_scale; 
    let distorted_uv = in.uv + distortion;

    let explored_raw = textureSample(mist_texture, mist_sampler, distorted_uv).r;

    let bp = world_pos * 0.01;
    let breath = sin(time * 0.1 + bp.x) * 0.5 + cos(time * 0.1 + bp.y) * 0.5;
    
    let breath_mask = smoothstep(0.0, 0.1, explored_raw);
    let explored_animated = explored_raw + (breath * 0.06 * breath_mask);

    let edge_variance = fbm(world_pos * 0.02 + vec2<f32>(time * 0.05, 0.0), 3);
    
    let softness = mix(0.4, 1.4, smoothstep(0.3, 0.7, edge_variance)); 
    let explored = clamp(smoothstep(0.0, softness, max(0.0, explored_animated)), 0.0, 1.0);
    
    let mist_alpha = pow(1.0 - explored, 1.8);

    if mist_alpha < 0.01 {
        discard;
    }

    let in_transition = explored; 

    // --- FRONTIÈRE AVEC L'OCÉAN TENTACULAIRE ---
    let coast_distortion = vec2<f32>(dx, dy) * 150.0 * uv_scale; 
    let coast_uv = in.uv + coast_distortion;
    
    let sdf_raw = textureSample(sdf_texture, sdf_sampler, coast_uv).r;
    let sdf_signed = (sdf_raw - 0.5) * 2.0;
    
    let coast_animated = sdf_signed + (breath * 0.03);

    let coast_mask = smoothstep(-0.02, 0.15, coast_animated);
    if coast_mask < 0.01 {
        discard;
    }

    // --- 2. TRAÎNÉES D'AQUARELLE ET DÉRIVE ---
    let ref_pos = world_pos * (9600.0 / mist_params.world_width);
    let cam = vec2<f32>(mist_params.camera_x, mist_params.camera_y) * (9600.0 / mist_params.world_width);
    
    let t_fluid = time * 0.0025;
    let scale = 0.018; 

    let base_coords = (ref_pos + cam * 0.07) * scale + vec2<f32>(time * 0.01, 0.0);
    let warped_pos = warp(base_coords, t_fluid);
    let base_noise = fbm(warped_pos, 5);
    let shaped = smoothstep(0.1, 0.8, base_noise);

    let dark_coords = (ref_pos + cam * 0.03) * (scale * 1.3) + vec2<f32>(time * 0.03, -time * 0.02);
    let warped_dark = warp(dark_coords, t_fluid * 1.4);
    let dark_noise = fbm(warped_dark, 4);
    let dark_patch = 1.0 - smoothstep(0.15, 0.65, dark_noise); 

    let shadow_coords = (ref_pos + cam * 0.12) * (scale * 2.0) + vec2<f32>(-time * 0.025, time * 0.03);
    let warped_shadow = warp(shadow_coords, t_fluid * 1.8);
    let shadow_noise = fbm(warped_shadow, 4);
    let shadow_patch = 1.0 - smoothstep(0.2, 0.7, shadow_noise);

    // --- 3. COULEURS NATURELLES ET HOSTILES ---
    let fog_light  = vec3<f32>(0.83, 0.84, 0.90);   
    let fog_mid    = vec3<f32>(0.63, 0.64, 0.76);   
    let fog_dark   = vec3<f32>(0.20, 0.22, 0.28);   
    
    var fog_color = mix(fog_mid, fog_light, shaped);

    let patch_intensity = clamp(dark_patch * 0.75 + shadow_patch * 0.45, 0.0, 1.0);
    fog_color = mix(fog_color, fog_dark, patch_intensity * 0.85); 

    let bruised_violet = vec3<f32>(0.35, 0.15, 0.55); 
    let rust_red       = vec3<f32>(0.65, 0.30, 0.25); 

    let v_coords = warped_pos * 1.5 + vec2<f32>(dx, dy) * 0.15 + vec2<f32>(time * 0.015, time * 0.01);
    let r_coords = warped_dark * 1.5 - vec2<f32>(dx, dy) * 0.15 + vec2<f32>(-time * 0.02, time * 0.01);

    let violet_mask = smoothstep(0.45, 0.75, fbm(v_coords, 3));
    let red_mask    = smoothstep(0.65, 0.85, fbm(r_coords, 3));

    fog_color = mix(fog_color, bruised_violet, violet_mask * 0.30);
    fog_color = mix(fog_color, rust_red, red_mask * 0.60); 

    // --- 4. PULSATION MALSAINE ---
    let pulse_noise = fbm(warped_pos * 1.2 + vec2<f32>(time * 0.004, 0.0), 3);
    let pulse_time = time * 1.2;
    let raw_pulse = sin(pulse_time + pulse_noise * 5.0);
    let glow_pulse = pow(max(0.0, raw_pulse), 3.0); 
    
    let blood_amber = vec3<f32>(0.45, 0.35, 0.32);
    let glow_strength = (0.05 + patch_intensity * 0.2) * glow_pulse;
    fog_color = mix(fog_color, blood_amber, glow_strength);

    // Volutes de bordures
    let wt = time * 0.015;
    let w1 = fbm((ref_pos + cam * 0.12) * 0.015 + vec2<f32>(wt * 0.3, wt * 0.2), 5);
    let w2 = fbm((ref_pos + cam * 0.12) * 0.020 + vec2<f32>(-wt * 0.25, wt * 0.3) + vec2<f32>(53.0, 29.0), 4);
    let wisp = w1 * 0.6 + w2 * 0.4;
    let wisp_mask = smoothstep(0.2, 0.7, wisp);

    let wisp_effect = in_transition * (0.3 + wisp_mask * 0.7);
    let final_alpha = mist_alpha * (1.0 - wisp_effect * 0.5);

    let alpha = final_alpha * coast_mask;

    if alpha < 0.01 {
        discard;
    }

    fog_color = mix(fog_color, fog_light, in_transition * 0.30);

    return vec4<f32>(fog_color, alpha);
}
