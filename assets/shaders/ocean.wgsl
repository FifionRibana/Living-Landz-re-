// assets/shaders/ocean.wgsl

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
    
    for (var i = 0; i < 5; i++) {
        value += amplitude * noise(pos);
        pos *= 2.0;
        amplitude *= 0.5;
    }
    
    return value;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let time_looped = (params.time * params.wave_speed) % (TAU * 16.0);
    
    // UV pour les textures globales
    let uv = in.uv;

    // === SDF : distance à la côte ===
    let sdf_raw = textureSample(sdf_texture, sdf_sampler, uv).r;
    let sdf_signed = (sdf_raw - 0.5) * 2.0;
    // sdf_signed < 0 = eau, > 0 = terre

    // Si on est sur terre, ne pas rendre (le terrain s'en charge)
    if sdf_signed > 0.1 {
        discard;
    }

    // === PROFONDEUR COMBINÉE : SDF + HEIGHTMAP ===

    // 1. Distance à la côte (SDF) - Tendance générale
    // -sdf_signed car négatif dans l'eau, on veut une valeur positive pour la distance
    let distance_from_shore = -sdf_signed;
    // Le SDF est déjà normalisé entre 0 (côte) et ~1 (large)
    // On le clamp entre 0 et 1 pour être sûr
    let sdf_depth = saturate(distance_from_shore);

    // 2. Heightmap - Profondeur locale du fond marin (lissée)
    // Calculer la taille d'un texel pour l'échantillonnage voisin
    let texel_size = 1.0 / vec2<f32>(params.world_width, params.world_height);

    // Échantillonnage 3x3 pour lisser la heightmap
    var height_sum = 0.0;
    let kernel_size = 2.0; // Rayon du kernel

    for (var y = -kernel_size; y <= kernel_size; y += 1.0) {
        for (var x = -kernel_size; x <= kernel_size; x += 1.0) {
            let offset = vec2<f32>(x, y) * texel_size;
            height_sum += textureSample(heightmap, heightmap_sampler, uv + offset).r;
        }
    }

    // Moyenne des échantillons (kernel 5x5 = 25 samples)
    let height = height_sum / 25.0;

    // Inverser si nécessaire : 0 = profond, 1 = peu profond
    let local_depth = 1.0 - height;

    // 3. Combiner les deux
    // Le SDF donne la tendance générale (70%)
    // La heightmap module localement (30%)
    let combined_depth = sdf_depth * 0.7 + local_depth * 0.3;

    // Courbe pour rendre la transition plus douce
    let depth_curved = pow(combined_depth, 0.7);

    // === COULEUR DE L'EAU BASÉE SUR LA PROFONDEUR ===
    var ocean_color = mix(shallow_color.rgb, deep_color.rgb, depth_curved);
    
    // === VARIATIONS SUBTILES ===
    let noise_uv = uv * 50.0; // Échelle du bruit
    
    // Vagues de surface
    let wave1 = fbm(noise_uv * 0.5 + vec2<f32>(time_looped * 0.01, time_looped * 0.008));
    let wave2 = fbm(noise_uv * 0.7 + vec2<f32>(-time_looped * 0.012, time_looped * 0.01));
    let waves = (wave1 + wave2) * 0.5;
    
    // Modulation subtile de la couleur
    ocean_color += vec3<f32>(waves * 0.02, waves * 0.03, waves * 0.04);

    // Caustiques (plus visibles en eau peu profonde)
    let caustic_strength = (1.0 - combined_depth) * 0.15;
    let caustic1 = pow(fbm(noise_uv * 2.0 + vec2<f32>(time_looped * 0.02, -time_looped * 0.015)), 2.0);
    let caustic2 = pow(fbm(noise_uv * 2.5 + vec2<f32>(-time_looped * 0.018, time_looped * 0.02)), 2.0);
    let caustics = (caustic1 + caustic2) * caustic_strength;

    ocean_color += vec3<f32>(caustics * 0.4, caustics * 0.6, caustics * 0.8);
    
    // === ÉCUME ===
    // Zone d'écume (près de la côte, côté eau)
    let foam_zone = smoothstep(-0.4, -0.05, sdf_signed) * (1.0 - smoothstep(-0.05, 0.05, sdf_signed));
    
    if foam_zone > 0.01 {
        let foam_noise_pos = noise_uv * 0.3;
        
        // Variations statiques
        let phase_offset_1 = hash(floor(foam_noise_pos * 2.0)) * TAU;
        let phase_offset_2 = hash(floor(foam_noise_pos * 2.0) + vec2<f32>(100.0, 0.0)) * TAU;
        let phase_offset_3 = hash(floor(foam_noise_pos * 3.0) + vec2<f32>(200.0, 0.0)) * TAU;
        
        let amp_variation = 0.6 + fbm(foam_noise_pos * 1.5) * 0.8;
        let width_variation = 0.7 + fbm(foam_noise_pos * 3.0 + vec2<f32>(50.0, 50.0)) * 0.6;
        let speed_variation = 0.8 + hash(floor(foam_noise_pos * 2.0) + vec2<f32>(0.0, 100.0)) * 0.4;
        
        // Vague principale
        let wave_1 = sin(time_looped * speed_variation + phase_offset_1);
        let wave_offset = wave_1 * params.wave_amplitude * amp_variation;
        let wave_center = -0.05 + wave_offset;
        let foam_half_width = params.foam_width * 0.5 * width_variation;
        let dist_to_wave = abs(sdf_signed - wave_center);
        let foam_intensity = 1.0 - smoothstep(0.0, foam_half_width, dist_to_wave);
        
        // Vague secondaire
        let wave_2 = sin(time_looped * 1.3 * speed_variation + phase_offset_2);
        let wave_offset_2 = wave_2 * params.wave_amplitude * 0.5 * amp_variation;
        let wave_center_2 = -0.18 + wave_offset_2;
        let foam_half_width_2 = params.foam_width * 0.35 * width_variation;
        let dist_to_wave_2 = abs(sdf_signed - wave_center_2);
        let foam_intensity_2 = (1.0 - smoothstep(0.0, foam_half_width_2, dist_to_wave_2)) * 0.5;
        
        // Vague tertiaire
        let wave_3 = sin(time_looped * 2.0 + phase_offset_3);
        let wave_offset_3 = wave_3 * params.wave_amplitude * 0.25;
        let wave_center_3 = -0.08 + wave_offset_3;
        let foam_half_width_3 = params.foam_width * 0.25;
        let dist_to_wave_3 = abs(sdf_signed - wave_center_3);
        let foam_intensity_3 = (1.0 - smoothstep(0.0, foam_half_width_3, dist_to_wave_3)) * 0.3;
        
        let total_foam = min(foam_intensity + foam_intensity_2 + foam_intensity_3, 1.0) * foam_zone;
        
        ocean_color = mix(ocean_color, foam_color.rgb, total_foam);
    }
    
    // === OPACITÉ ===
    // Transition douce vers la côte pour blend avec le sable
    let edge_opacity = smoothstep(0.05, -0.1, sdf_signed);
    
    return vec4<f32>(ocean_color, edge_opacity);
}