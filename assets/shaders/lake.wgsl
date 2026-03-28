#import bevy_sprite::mesh2d_vertex_output::VertexOutput
#import bevy_render::globals::Globals

@group(0) @binding(1) var<uniform> globals: Globals;

struct LakeParams {
    world_width: f32,
    world_height: f32,
    _padding1: f32,
    _padding2: f32,
}

@group(2) @binding(0) var mask_texture: texture_2d<f32>;
@group(2) @binding(1) var mask_sampler: sampler;
@group(2) @binding(2) var sdf_texture: texture_2d<f32>;
@group(2) @binding(3) var sdf_sampler: sampler;
@group(2) @binding(4) var<uniform> shallow_color: vec4<f32>;
@group(2) @binding(5) var<uniform> deep_color: vec4<f32>;
@group(2) @binding(6) var<uniform> params: LakeParams;

const TAU: f32 = 6.28318530718;

fn hash(p: vec2<f32>) -> f32 {
    let h = dot(p, vec2<f32>(127.1, 311.7));
    return fract(sin(h) * 43758.5453123);
}

fn noise(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let a = hash(i);
    let b = hash(i + vec2<f32>(1.0, 0.0));
    let c = hash(i + vec2<f32>(0.0, 1.0));
    let d = hash(i + vec2<f32>(1.0, 1.0));
    let u = f * f * (3.0 - 2.0 * f);
    return mix(mix(a, b, u.x), mix(c, d, u.x), u.y);
}

fn fbm(p: vec2<f32>, octaves: i32) -> f32 {
    var value = 0.0;
    var amplitude = 0.5;
    var pos = p;
    let rot = mat2x2<f32>(0.8, 0.6, -0.6, 0.8);
    for (var i = 0; i < octaves; i++) {
        value += amplitude * noise(pos);
        pos = rot * pos * 2.0;
        amplitude *= 0.5;
    }
    return value;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv;
    let time = globals.time;

    // === Lake mask — for discard only ===
    let mask = textureSample(mask_texture, mask_sampler, uv).r;
    if mask < 0.5 {
        discard;
    }

    // === SDF — primary depth source ===
    let sdf_raw = textureSample(sdf_texture, sdf_sampler, uv).r;
    let sdf_signed = (sdf_raw - 0.5) * 2.0; // -1=deep lake, 0=shore, +1=land

    // Discard land pixels
    if sdf_signed > 0.1 {
        discard;
    }

    let world_pos = uv * vec2<f32>(params.world_width, params.world_height);
    let ref_pos = world_pos * (9600.0 / params.world_width);
    
    // Depth from SDF
    let depth_raw = saturate(-sdf_signed);
    let depth_noise = (fbm(ref_pos * 0.03, 4) - 0.5) * 0.1;
    let depth = pow(saturate(depth_raw + depth_noise), 1.2);


    // === Color gradient ===
    var color = mix(shallow_color.rgb, deep_color.rgb, depth);

    // === Subtle color variation ===
    let variation = fbm(ref_pos * 0.0003, 4);
    color += vec3<f32>(
        (variation - 0.5) * 0.02,
        (variation - 0.5) * 0.01,
        -(variation - 0.5) * 0.015
    );

    // === Micro-ripples — calm lake surface ===
    let ripple_time = time * 0.6;
    let r1 = noise(ref_pos * 0.5);
    let r2 = noise(ref_pos * 0.6 + vec2<f32>(43.0, 17.0));
    let r3 = noise(ref_pos * 1.1 + vec2<f32>(91.0, 53.0));

    let flicker1 = sin(ripple_time * 0.8 + r1 * TAU);
    let flicker2 = sin(ripple_time * 1.1 + r2 * TAU);
    let flicker3 = sin(ripple_time * 1.7 + r3 * TAU);

    let ripples = (flicker1 + flicker2) * 0.2 + flicker3 * 0.15;
    let ripple_strength = smoothstep(0.0, 0.2, depth) * 0.008;
    color += vec3<f32>(ripples * 0.5, ripples * 0.7, ripples) * ripple_strength;

    // === Caustiques near banks ===
    let caustic_zone = 1.0 - smoothstep(0.0, 0.3, depth);
    if caustic_zone > 0.01 {
        let ct = time * 0.3;
        let c_uv = ref_pos * 0.35;
        let c1 = pow(fbm(c_uv + vec2<f32>(ct * 0.08, -ct * 0.06), 4), 2.0);
        let c2 = pow(fbm(c_uv * 1.3 + vec2<f32>(-ct * 0.07, ct * 0.09), 4), 2.0);
        let caustics = (c1 + c2) * 0.5;

        let caustic_tint = mix(
            vec3<f32>(0.2, 0.4, 0.5),
            vec3<f32>(0.5, 0.45, 0.3),
            1.0 - smoothstep(0.0, 0.12, depth)
        );
        color += caustic_tint * caustics * caustic_zone * 0.15;
    }

    // === Gentle wind patches ===
    let wind_time = time * 0.015;
    let wind = fbm(ref_pos * 0.00015 + vec2<f32>(wind_time * 0.08, wind_time * 0.05), 3);
    color *= 1.0 + (wind - 0.5) * 0.03 * smoothstep(0.1, 0.3, depth);

    // === Opacity — wide smooth transition at shore ===
    // Use noise to break up the geometric edge
    let shore_noise = fbm(ref_pos * 0.04, 3) * 0.06;
    let opacity = smoothstep(0.15 + shore_noise, -0.45 + shore_noise, sdf_signed);

    return vec4<f32>(color, opacity);
}