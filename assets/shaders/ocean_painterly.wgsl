#import bevy_sprite::mesh2d_vertex_output::VertexOutput

struct OceanParams {
    time: f32,
    world_width: f32,
    world_height: f32,
    max_depth: f32,
    wave_speed: f32,
    wave_amplitude: f32,
    foam_width: f32,
    _padding: f32,
}

@group(2) @binding(0) var heightmap: texture_2d<f32>;
@group(2) @binding(1) var heightmap_sampler: sampler;
@group(2) @binding(2) var sdf_texture: texture_2d<f32>;
@group(2) @binding(3) var sdf_sampler: sampler;
@group(2) @binding(4) var<uniform> shallow_color: vec4<f32>;
@group(2) @binding(5) var<uniform> deep_color: vec4<f32>;
@group(2) @binding(6) var<uniform> foam_color: vec4<f32>;
@group(2) @binding(7) var<uniform> params: OceanParams;

const TAU: f32 = 6.28318530718;
const ABYSS_COLOR: vec3<f32> = vec3<f32>(0.015, 0.04, 0.08);

// ============================================================================
// BRUIT
// ============================================================================

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
    let sdf_raw = textureSample(sdf_texture, sdf_sampler, uv).r;
    let sdf_signed = (sdf_raw - 0.5) * 2.0;

    if sdf_signed > 0.02 {
        discard;
    }
    let sdf_depth_raw = saturate(-sdf_signed);

    // === Heightmap ===
    let height = textureSample(heightmap, heightmap_sampler, uv).r;
    let bathymetry = 1.0 - height;

    // === Combine SDF + heightmap ===
    let world_pos = uv * vec2<f32>(params.world_width, params.world_height);
    let ref_pos = world_pos * (9600.0 / params.world_width);
    
    let sdf_depth = pow(sdf_depth_raw, 0.8);

    let shore_extent = fbm(ref_pos * 0.002, 3);
    let shallow_end = 0.25 + shore_extent * 0.6; // varie entre 0.25 et 0.55
    let sdf_weight = 1.0 - smoothstep(0.08, shallow_end, sdf_depth);
    let depth = sdf_depth * sdf_weight + bathymetry * (1.0 - sdf_weight);

    // === Gradient 3 points ===
    // Variation de teinte côtière — certaines zones plus vertes, d'autres plus bleues
    let shore_noise = fbm(ref_pos * 0.02, 4);
    let shallow_varied = shallow_color.rgb + vec3<f32>(
        (shore_noise - 0.5) * 0.04,
        (shore_noise - 0.5) * 0.02,
        -(shore_noise - 0.5) * 0.03
    );
    let shallow_mix = mix(shallow_varied, deep_color.rgb, saturate(depth * 2.0));
    let deep_mix = mix(deep_color.rgb, ABYSS_COLOR, saturate((depth - 0.5) * 2.0));
    var ocean_color = mix(shallow_mix, deep_mix, step(0.5, depth));

    let time = params.time;

    // === Courants océaniques ===
    let current_time = time * 0.03;
    let c1 = fbm(ref_pos * 0.00012 + vec2<f32>(current_time * 0.4, -current_time * 0.25), 5);
    let c2 = fbm(ref_pos * 0.00025 + vec2<f32>(-current_time * 0.2, current_time * 0.35) + vec2<f32>(42.0, 17.0), 4);
    let c3 = fbm(ref_pos * 0.0006 + vec2<f32>(current_time * 0.15, current_time * 0.1) + vec2<f32>(91.0, 53.0), 3);
    let current_val = c1 * 0.5 + c2 * 0.3 + c3 * 0.2;
    let current_strength = smoothstep(0.05, 0.25, sdf_depth);
    let cshift = (current_val - 0.5);
    ocean_color += vec3<f32>(cshift * 0.02, cshift * 0.008, -cshift * 0.015) * current_strength;
    ocean_color *= 1.0 + cshift * 0.04 * current_strength;

    // === Vent ===
    let wind_time = time * 0.02;
    let roughness = fbm(ref_pos * 0.00012 + vec2<f32>(wind_time * 0.1, wind_time * 0.06), 3);
    let rough_effect = (roughness - 0.5) * 0.04 * smoothstep(0.1, 0.35, sdf_depth);
    ocean_color *= 1.0 + rough_effect;

    // === Turbidité côtière ===
    let turbid_color = vec3<f32>(0.10, 0.14, 0.11);
    if sdf_depth < 0.40 {
        let turb_base = 1.0 - smoothstep(0.0, 0.35, sdf_depth);
        let turb_noise = fbm(ref_pos * 0.0008 + vec2<f32>(wind_time * 0.05, 0.0), 3);
        let turbidity = turb_base * (0.3 + turb_noise * 0.7) * 0.18;
        ocean_color = mix(ocean_color, turbid_color, turbidity);
    }

    // === Vagues de surface ===
    let noise_uv = ref_pos * 0.0052;
    let time_looped = time * params.wave_speed;
    let wave1 = fbm(noise_uv * 0.5 + vec2<f32>(time_looped * 0.008, time_looped * 0.006), 4);
    let wave2 = fbm(noise_uv * 0.7 + vec2<f32>(-time_looped * 0.009, time_looped * 0.007), 4);
    let waves = (wave1 + wave2) * 0.5;
    ocean_color += vec3<f32>(waves * 0.4, waves * 0.6, waves) * 0.015;

    // === Ripples ===
    let ripple_time = time * 1.2;
    let ripple1 = noise(ref_pos * 0.45);
    let ripple2 = noise(ref_pos * 0.55 + vec2<f32>(43.0, 17.0));
    let ripple3 = noise(ref_pos * 1.0 + vec2<f32>(91.0, 53.0));
    let flicker1 = sin(ripple_time * 1.1 + ripple1 * TAU);
    let flicker2 = sin(ripple_time * 1.5 + ripple2 * TAU);
    let flicker3 = sin(ripple_time * 2.1 + ripple3 * TAU);
    let ripples = (flicker1 + flicker2) * 0.25 + flicker3 * 0.2;
    ocean_color += vec3<f32>(ripples * 0.012 * 0.6, ripples * 0.012 * 0.8, ripples * 0.012);

    // === Caustiques ===
    let caustic_zone = 1.0 - smoothstep(0.0, 0.35, sdf_depth);
    if caustic_zone > 0.01 {
        let caustic_time = time * 0.4;
        let c_uv = ref_pos * 0.3;
        let c1_raw = fbm(c_uv + vec2<f32>(caustic_time * 0.12, -caustic_time * 0.08), 4);
        let c1c = pow(c1_raw, 2.0);
        let c2_raw = fbm(c_uv * 1.3 + vec2<f32>(-caustic_time * 0.10, caustic_time * 0.13), 4);
        let c2c = pow(c2_raw, 2.0);
        let caustics = (c1c + c2c) * 0.5;
        let caustic_intensity = caustic_zone * 0.24;
        let caustic_tint = mix(
            vec3<f32>(0.3, 0.5, 0.7),
            vec3<f32>(0.6, 0.55, 0.35),
            1.0 - smoothstep(0.0, 0.15, sdf_depth)
        );
        ocean_color += caustic_tint * caustics * caustic_intensity;
    }

    // === Brume ===
    let haze_color = vec3<f32>(0.12, 0.15, 0.19);
    let haze_depth = sdf_depth * 0.5 + bathymetry * 0.5;
    let haze_amount = smoothstep(0.02, 0.65, haze_depth) * 0.20;
    ocean_color = mix(ocean_color, haze_color, haze_amount);

// =====================================================================
    // ÉCUME — vagues suivant les iso-contours SDF vers la côte
    // =====================================================================

    let foam_time = time * params.wave_speed * 0.7;

    // Foam line width correction for world scale
    let sdf_dims = vec2<f32>(textureDimensions(sdf_texture));
    let fc = (sdf_dims.x / params.world_width) / (1024.0 / 9600.0);

    // Zone d'écume
    let foam_outer = -0.45;
    let foam_zone = smoothstep(foam_outer, -0.05, sdf_signed)
                   * (1.0 - smoothstep(-0.05, 0.05, sdf_signed));

    if foam_zone > 0.01 {
        let phase_1 = fbm(ref_pos * 0.006, 3) * TAU;
        let phase_2 = fbm(ref_pos * 0.007 + vec2<f32>(100.0, 0.0), 3) * TAU;
        let amp_var = 0.5 + fbm(ref_pos * 0.005, 3) * 0.5;
        let width_var = 0.7 + fbm(ref_pos * 0.008 + vec2<f32>(50.0, 50.0), 3) * 0.6;

        // Ajouter juste avant "// Vague principale" :
        // Bruit côtier pour casser les lignes droites des iso-contours SDF
        let coast_warp = (fbm(ref_pos * 0.025, 4) - 0.5) * 0.03 * fc;
        let sdf_warped = sdf_signed + coast_warp;

        // Vague principale — approche fine, reflux s'étalant
        let local_phase1 = fbm(ref_pos * 0.015, 3) * TAU;
        let local_speed1 = 0.8 + fbm(ref_pos * 0.004, 2) * 0.4;
        let cycle1 = fract(foam_time * 0.06 * local_speed1 + phase_1 / TAU + local_phase1 / TAU);
        let wave_pos1 = select(
            smoothstep(0.0, 0.6, cycle1),
            1.0 - (cycle1 - 0.6) * 2.5,
            cycle1 > 0.6
        );
        let wc1 = -0.06 * (1.0 - wave_pos1) * amp_var;
        let width_mult1 = select(1.0, 1.0 + (1.0 - wave_pos1) * 3.0, cycle1 > 0.6);
        let fhw1 = params.foam_width * fc * 0.15 * width_var * width_mult1;
        let fade_in1 = smoothstep(0.0, 0.3, cycle1);
        let fade_out1 = smoothstep(1.0, 0.7, cycle1);
        let retreat_mask1 = select(1.0, smoothstep(-0.03, -0.005, sdf_warped), cycle1 > 0.6);
        let fi1 = (1.0 - smoothstep(0.0, fhw1, abs(sdf_warped - wc1)))
                 * fade_in1 * fade_out1 * retreat_mask1 * 0.5;

        // Vague secondaire — déphasée et vitesse locale différente
        let local_phase2 = fbm(ref_pos * 0.012 + vec2<f32>(77.0, 33.0), 3) * TAU;
        let local_speed2 = 0.7 + fbm(ref_pos * 0.005 + vec2<f32>(20.0, 50.0), 2) * 0.6;
        let cycle2 = fract(foam_time * 0.045 * local_speed2 + phase_2 / TAU + local_phase2 / TAU);
        let wave_pos2 = select(
            smoothstep(0.0, 0.6, cycle2),
            1.0 - (cycle2 - 0.6) * 2.5,
            cycle2 > 0.6
        );
        let wc2 = -0.08 * (1.0 - wave_pos2) * amp_var;
        let width_mult2 = select(1.0, 1.0 + (1.0 - wave_pos2) * 3.0, cycle2 > 0.6);
        let fhw2 = params.foam_width * fc * 0.12 * width_var * width_mult2;
        let fade_in2 = smoothstep(0.0, 0.35, cycle2);
        let fade_out2 = smoothstep(1.0, 0.75, cycle2);
        let retreat_mask2 = select(1.0, smoothstep(-0.02, -0.003, sdf_warped), cycle2 > 0.6);
        let fi2 = (1.0 - smoothstep(0.0, fhw2, abs(sdf_warped - wc2)))
                 * fade_in2 * fade_out2 * retreat_mask2 * 0.3;

        // Vagues directionnelles au large
        var fi_directional = 0.0;
        for (var wi = 0; wi < 4; wi++) {
            let wave_idx = f32(wi);
            let w_phase = fbm(ref_pos * (0.003 + wave_idx * 0.001) + vec2<f32>(wave_idx * 37.0, wave_idx * 53.0), 3);
            let w_speed = 0.05 + wave_idx * 0.012;
            let raw_cycle = foam_time * w_speed + w_phase;
            let w_cycle = fract(raw_cycle);
            let cycle_id = floor(raw_cycle) % 256.0;
            let cycle_rand = fract(sin(cycle_id * 43.758 + wave_idx * 17.31) * 12345.6789);
            let cycle_rand2 = fract(sin(cycle_id * 71.137 + wave_idx * 23.57) * 54321.9876);
            let cycle_occur = fract(sin(cycle_id * 97.13 + wave_idx * 41.07) * 78901.2345);
            let wave_active = select(0.0, 1.0, cycle_occur < 0.35);
            let start_base = -0.35 + wave_idx * 0.03;
            let start_dist = start_base - cycle_rand * 0.06;
            let end_base = -0.18 + wave_idx * 0.02;
            let end_dist = end_base + cycle_rand2 * 0.04;
            let travel = end_dist - start_dist;
            let w_center = start_dist + w_cycle * travel;
            let w_fade = smoothstep(0.0, 0.15, w_cycle)
                       * (1.0 - smoothstep(0.75, 1.0, w_cycle));
            let w_half = params.foam_width * fc * (0.06 + wave_idx * 0.008) * width_var;
            let w_intensity = 1.0 - smoothstep(0.0, w_half, abs(sdf_signed - w_center));
            let band_seed = vec2<f32>(cycle_id * 7.13 + wave_idx * 31.0, cycle_id * 11.7 + wave_idx * 17.0);
            let band_noise = fbm(ref_pos * (0.025 + wave_idx * 0.008) + band_seed, 4);
            let band_mask = smoothstep(0.52, 0.62, band_noise);
            fi_directional += w_intensity * w_fade * band_mask * wave_active * 0.15;
        }
        fi_directional = min(fi_directional, 0.6);

        var total_foam = min(fi1 + fi2 + fi_directional, 1.0) * foam_zone;

        // Liseré statique côtier
        let static_foam = smoothstep(-0.06 * fc, -0.02 * fc, sdf_warped)
                        * (1.0 - smoothstep(-0.02 * fc, -0.005 * fc, sdf_warped))
                        * 0.15;
        total_foam = max(total_foam, static_foam);

        // Variation couleur écume
        let fcv = fbm(ref_pos * 0.012 + vec2<f32>(foam_time * 0.003, 0.0), 3);
        var foam_tinted = foam_color.rgb + vec2<f32>(0.0, 0.0).xxx + vec3<f32>(
            (fcv - 0.5) * 0.05,
            (fcv - 0.5) * 0.03,
            (fcv - 0.5) * 0.02
        );

        ocean_color = mix(ocean_color, foam_tinted, total_foam);
    }

    // === Opacité — synchronisée avec l'écume ===
    let foam_time_op = time * params.wave_speed * 0.7;
    let op_phase = fbm(ref_pos * 0.015, 3) * TAU;
    let op_speed = 0.8 + fbm(ref_pos * 0.004, 2) * 0.4;
    let op_cycle = fract(foam_time_op * 0.06 * op_speed + op_phase / TAU);
    // Quand la vague arrive (cycle < 0.6), l'eau avance sur la plage
    let wave_advance = select(
        smoothstep(0.0, 0.6, op_cycle) * 0.03,  // approche: pousse jusqu'à +0.03
        (1.0 - (op_cycle - 0.6) * 2.5) * 0.02,  // reflux: recule vite
        op_cycle > 0.6
    );
    let opacity = smoothstep(0.02 + wave_advance, -0.08, sdf_signed);

    return vec4<f32>(ocean_color, opacity);
}