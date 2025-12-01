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
    let overlap = 0.4;
    
    let total_span = tex_res - 1.0 + 2.0 * overlap;
    let uv_start = overlap / total_span;
    let uv_end = (tex_res - 1.0 + overlap) / total_span;
    
    let uv_mapped = vec2<f32>(
        mix(uv_start, uv_end, in.uv.x),
        mix(uv_start, uv_end, in.uv.y)
    );
    
    let sdf_raw = textureSample(sdf_texture, sdf_sampler, uv_mapped).r;
    let sdf_signed = (sdf_raw - 0.5) * 2.0;
    
    // === OPACITÉ ===
    let opacity = smoothstep(sdf_params.opacity_start, sdf_params.opacity_end, sdf_signed);
    
    // === COULEUR DE BASE ===
    let color_t = smoothstep(sdf_params.beach_start, sdf_params.beach_end, sdf_signed);
    var color = mix(sand_color.rgb, grass_color.rgb, color_t);
    
    // === ÉCUME AVEC VARIATION ===
    
    // Boucler le temps pour éviter la perte de précision
    // Période de ~100 secondes (suffisant pour ne pas voir de répétition)
    let time_looped = (wave_params.time * wave_params.wave_speed) % (TAU * 16.0);
    
    let noise_pos = in.uv * 8.0;
    
    // Variations statiques basées sur la position (ne dépendent pas du temps)
    let phase_offset_1 = hash(floor(noise_pos * 2.0)) * TAU;
    let phase_offset_2 = hash(floor(noise_pos * 2.0) + vec2<f32>(100.0, 0.0)) * TAU;
    let phase_offset_3 = hash(floor(noise_pos * 3.0) + vec2<f32>(200.0, 0.0)) * TAU;
    
    let amp_variation = 0.6 + fbm(noise_pos * 1.5) * 0.8;
    let width_variation = 0.7 + fbm(noise_pos * 3.0 + vec2<f32>(50.0, 50.0)) * 0.6;
    let speed_variation = 0.8 + hash(floor(noise_pos * 2.0) + vec2<f32>(0.0, 100.0)) * 0.4;
    
    // === VAGUE PRINCIPALE ===
    let wave_1 = sin(time_looped * speed_variation + phase_offset_1);
    let wave_offset = wave_1 * wave_params.wave_amplitude * amp_variation;
    let wave_center = -0.05 + wave_offset;
    
    let foam_half_width = wave_params.foam_width * 0.5 * width_variation;
    let dist_to_wave = abs(sdf_signed - wave_center);
    let foam_intensity = 1.0 - smoothstep(0.0, foam_half_width, dist_to_wave);
    
    // === VAGUE SECONDAIRE ===
    let wave_2 = sin(time_looped * 1.3 * speed_variation + phase_offset_2);
    let wave_offset_2 = wave_2 * wave_params.wave_amplitude * 0.5 * amp_variation;
    let wave_center_2 = -0.18 + wave_offset_2;
    let foam_half_width_2 = wave_params.foam_width * 0.35 * width_variation;
    
    let dist_to_wave_2 = abs(sdf_signed - wave_center_2);
    let foam_intensity_2 = (1.0 - smoothstep(0.0, foam_half_width_2, dist_to_wave_2)) * 0.5;
    
    // === VAGUE TERTIAIRE ===
    let wave_3 = sin(time_looped * 2.0 + phase_offset_3);
    let wave_offset_3 = wave_3 * wave_params.wave_amplitude * 0.25;
    let wave_center_3 = -0.08 + wave_offset_3;
    let foam_half_width_3 = wave_params.foam_width * 0.25;
    
    let dist_to_wave_3 = abs(sdf_signed - wave_center_3);
    let foam_intensity_3 = (1.0 - smoothstep(0.0, foam_half_width_3, dist_to_wave_3)) * 0.3;
    
    // === COMBINER ===
    let total_foam = min(foam_intensity + foam_intensity_2 + foam_intensity_3, 1.0);
    
    let foam_color = vec3<f32>(0.95, 0.98, 1.0);
    
    // Zone d'écume
    let foam_zone = smoothstep(-0.35, 0.0, sdf_signed) * (1.0 - smoothstep(0.0, 0.15, sdf_signed));
    let final_foam = total_foam * foam_zone;
    
    color = mix(color, foam_color, final_foam);
    
    if opacity < 0.01 {
        discard;
    }
    
    return vec4<f32>(color, opacity);
}