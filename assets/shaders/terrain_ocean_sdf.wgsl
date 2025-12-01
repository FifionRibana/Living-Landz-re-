#import bevy_sprite::mesh2d_vertex_output::VertexOutput

struct SdfParams {
    beach_start: f32,
    beach_end: f32,
    opacity_start: f32,
    opacity_end: f32,
}

struct WaveParams {
    time: f32,
    wave_speed: f32,
    wave_amplitude: f32,
    foam_width: f32,
}

@group(2) @binding(0) var sdf_texture: texture_2d<f32>;
@group(2) @binding(1) var sdf_sampler: sampler;
@group(2) @binding(2) var<uniform> sand_color: vec4<f32>;
@group(2) @binding(3) var<uniform> grass_color: vec4<f32>;
@group(2) @binding(4) var<uniform> sdf_params: SdfParams;
@group(2) @binding(5) var<uniform> wave_params: WaveParams;

const PI: f32 = 3.14159265359;
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

fn fbm(p: vec2<f32>) -> f32 {
    var value = 0.0;
    var amplitude = 0.5;
    var pos = p;
    
    for (var i = 0; i < 4; i++) {
        value += amplitude * noise(pos);
        pos *= 2.0;
        amplitude *= 0.5;
    }
    
    return value;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let tex_res = 64.0;
    let overlap = 0.5;
    
    let total_span = tex_res - 1.0 + 2.0 * overlap;
    let uv_start = overlap / total_span;
    let uv_end = (tex_res - 1.0 + overlap) / total_span;
    
    let uv_mapped = vec2<f32>(
        mix(uv_start, uv_end, in.uv.x),
        mix(uv_start, uv_end, in.uv.y)
    );
    
    let sdf_raw = textureSample(sdf_texture, sdf_sampler, uv_mapped).r;
    let sdf_signed = (sdf_raw - 0.5) * 2.0;
    
    let time_looped = (wave_params.time * wave_params.wave_speed) % (TAU * 16.0);
    let noise_pos = in.uv * 8.0;
    
    // === COULEURS DE BASE ===
    let shallow_water_color = vec3<f32>(0.15, 0.35, 0.45);  // Eau peu profonde
    let deep_water_color = vec3<f32>(0.05, 0.12, 0.18);     // Eau profonde
    let foam_color = vec3<f32>(0.9, 0.95, 1.0);
    
    // === TERRAIN (sable → herbe) ===
    let terrain_t = smoothstep(sdf_params.beach_start, sdf_params.beach_end, sdf_signed);
    let terrain_color = mix(sand_color.rgb, grass_color.rgb, terrain_t);
    
    // === OCÉAN ===
    // Profondeur : plus on est négatif, plus c'est profond
    let depth = smoothstep(0.0, -1.0, sdf_signed);
    var ocean_color = mix(shallow_water_color, deep_water_color, depth);
    
    // Variation subtile de l'eau avec le bruit
    let water_noise = fbm(noise_pos * 3.0 + vec2<f32>(time_looped * 0.02, time_looped * 0.015)) * 0.08;
    ocean_color += vec3<f32>(water_noise * 0.5, water_noise * 0.7, water_noise);
    
    // Reflets/caustiques subtils
    let caustic_noise = fbm(noise_pos * 6.0 + vec2<f32>(time_looped * 0.05, -time_looped * 0.03));
    let caustics = pow(caustic_noise, 2.0) * 0.15 * (1.0 - depth * 0.7);
    ocean_color += vec3<f32>(caustics * 0.3, caustics * 0.5, caustics);
    
    // === ÉCUME ===
    let phase_offset_1 = hash(floor(noise_pos * 2.0)) * TAU;
    let phase_offset_2 = hash(floor(noise_pos * 2.0) + vec2<f32>(100.0, 0.0)) * TAU;
    let phase_offset_3 = hash(floor(noise_pos * 3.0) + vec2<f32>(200.0, 0.0)) * TAU;
    
    let amp_variation = 0.6 + fbm(noise_pos * 1.5) * 0.8;
    let width_variation = 0.7 + fbm(noise_pos * 3.0 + vec2<f32>(50.0, 50.0)) * 0.6;
    let speed_variation = 0.8 + hash(floor(noise_pos * 2.0) + vec2<f32>(0.0, 100.0)) * 0.4;
    
    // Vague principale
    let wave_1 = sin(time_looped * speed_variation + phase_offset_1);
    let wave_offset = wave_1 * wave_params.wave_amplitude * amp_variation;
    let wave_center = -0.05 + wave_offset;
    let foam_half_width = wave_params.foam_width * 0.5 * width_variation;
    let dist_to_wave = abs(sdf_signed - wave_center);
    let foam_intensity = 1.0 - smoothstep(0.0, foam_half_width, dist_to_wave);
    
    // Vague secondaire
    let wave_2 = sin(time_looped * 1.3 * speed_variation + phase_offset_2);
    let wave_offset_2 = wave_2 * wave_params.wave_amplitude * 0.5 * amp_variation;
    let wave_center_2 = -0.18 + wave_offset_2;
    let foam_half_width_2 = wave_params.foam_width * 0.35 * width_variation;
    let dist_to_wave_2 = abs(sdf_signed - wave_center_2);
    let foam_intensity_2 = (1.0 - smoothstep(0.0, foam_half_width_2, dist_to_wave_2)) * 0.5;
    
    // Vague tertiaire
    let wave_3 = sin(time_looped * 2.0 + phase_offset_3);
    let wave_offset_3 = wave_3 * wave_params.wave_amplitude * 0.25;
    let wave_center_3 = -0.08 + wave_offset_3;
    let foam_half_width_3 = wave_params.foam_width * 0.25;
    let dist_to_wave_3 = abs(sdf_signed - wave_center_3);
    let foam_intensity_3 = (1.0 - smoothstep(0.0, foam_half_width_3, dist_to_wave_3)) * 0.3;
    
    // Zone d'écume (près de la côte)
    let foam_zone = smoothstep(-0.4, -0.05, sdf_signed) * (1.0 - smoothstep(-0.05, 0.15, sdf_signed));
    let total_foam = min(foam_intensity + foam_intensity_2 + foam_intensity_3, 1.0) * foam_zone;
    
    // Appliquer l'écume à l'océan
    ocean_color = mix(ocean_color, foam_color, total_foam);
    
    // === MÉLANGE FINAL ===
    // Transition douce entre océan et terrain
    let land_blend = smoothstep(-0.05, 0.1, sdf_signed);
    var final_color = mix(ocean_color, terrain_color, land_blend);
    
    // Opacité : l'eau profonde devient transparente pour voir le fond global si besoin
    // Ici on garde opaque mais on pourrait ajuster
    let final_opacity = 1.0;
    
    return vec4<f32>(final_color, final_opacity);
}