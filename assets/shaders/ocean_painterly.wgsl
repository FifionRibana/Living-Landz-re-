// assets/shaders/ocean_painterly.wgsl
//
// Étape 2 : Gradient + écume + vagues
// - Gradient : pow(2.0) sur SDF, heightmap au large
// - Écume : 2 vagues lentes, liseré statique, bords organiques
// - Vagues de surface : subtiles

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

// ============================================================================
// FRAGMENT SHADER
// ============================================================================

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv;
    let time = params.time;

    // === SDF ===
    let sdf_raw = textureSample(sdf_texture, sdf_sampler, uv).r;
    let sdf_signed = (sdf_raw - 0.5) * 2.0;

    if sdf_signed > 0.1 {
        discard;
    }

    let sdf_depth_raw = saturate(-sdf_signed);
    // Étirer la zone côtière
    let sdf_depth = pow(sdf_depth_raw, 1.5);

    // === Heightmap ===
    let hm_dims = vec2<f32>(textureDimensions(heightmap));
    let hm_texel = 1.0 / hm_dims;

    var height_sum = 0.0;
    var weight_total = 0.0;
    for (var dy = -2.0; dy <= 2.0; dy += 1.0) {
        for (var dx = -2.0; dx <= 2.0; dx += 1.0) {
            let offset = vec2<f32>(dx, dy) * hm_texel * 1.5;
            let dist = abs(dx) + abs(dy);
            let w = select(select(select(1.0, 2.0, dist < 3.5), 4.0, dist < 2.5), 6.0, dist < 1.5);
            height_sum += textureSample(heightmap, heightmap_sampler, uv + offset).r * w;
            weight_total += w;
        }
    }
    let height = height_sum / weight_total;
    let bathymetry = 1.0 - height;

    // === Combiner SDF + heightmap ===
    let world_pos = uv * vec2<f32>(params.world_width, params.world_height);

    let sdf_weight = 1.0 - smoothstep(0.3, 0.9, sdf_depth);
    let depth = sdf_depth * sdf_weight + bathymetry * (1.0 - sdf_weight);

    // === Gradient de couleur 3 points ===
    var ocean_color: vec3<f32>;
    if depth < 0.5 {
        ocean_color = mix(shallow_color.rgb, deep_color.rgb, depth * 2.0);
    } else {
        ocean_color = mix(deep_color.rgb, ABYSS_COLOR, (depth - 0.5) * 2.0);
    }

    // =====================================================================
    // COURANTS OCÉANIQUES — variations lentes de teinte et luminosité
    // 2 couches de FBM qui dérivent dans des directions différentes
    // Pas de domain warping = pas d'artefacts géométriques
    // =====================================================================

    let current_time = time * 0.03; // Très lent

    // Couche 1 : très grande échelle, dérive sud-est
    let c1 = fbm(world_pos * 0.00012 + vec2<f32>(current_time * 0.4, -current_time * 0.25), 5);

    // Couche 2 : échelle moyenne, dérive nord-ouest (interférence)
    let c2 = fbm(world_pos * 0.00025 + vec2<f32>(-current_time * 0.2, current_time * 0.35) + vec2<f32>(42.0, 17.0), 4);

    // Couche 3 : petite échelle, lente, pour les détails
    let c3 = fbm(world_pos * 0.0006 + vec2<f32>(current_time * 0.15, current_time * 0.1) + vec2<f32>(91.0, 53.0), 3);

    let current_val = c1 * 0.5 + c2 * 0.3 + c3 * 0.2;

    // Actif dès qu'on n'est plus sur la frange côtière immédiate
    let current_strength = smoothstep(0.05, 0.25, sdf_depth);

    // Variation de teinte chaude/froide
    let cshift = (current_val - 0.5);
    ocean_color += vec3<f32>(cshift * 0.02, cshift * 0.008, -cshift * 0.015) * current_strength;

    // Variation de luminosité
    ocean_color *= 1.0 + cshift * 0.04 * current_strength;

    // =====================================================================
    // RUGOSITÉ DE SURFACE — patches de vent
    // Zones calmes (lisses, plus claires) vs zones ventées (clapot, plus sombres)
    // Très basse fréquence, évolue lentement
    // =====================================================================

    let wind_time = time * 0.02;
    let roughness = fbm(world_pos * 0.00012 + vec2<f32>(wind_time * 0.1, wind_time * 0.06), 3);

    // Modulation de luminosité douce
    let rough_effect = (roughness - 0.5) * 0.04 * smoothstep(0.1, 0.35, sdf_depth);
    ocean_color *= 1.0 + rough_effect;

    // =====================================================================
    // TURBIDITÉ CÔTIÈRE — voile vert-brun près des côtes
    // Simulant sédiments, algues, eaux troubles des estuaires
    // =====================================================================

    let turbid_color = vec3<f32>(0.10, 0.14, 0.11); // Vert-brun côtier
    if sdf_depth < 0.40 {
        let turb_base = 1.0 - smoothstep(0.0, 0.35, sdf_depth);
        // Panaches irréguliers le long de la côte
        let turb_noise = fbm(world_pos * 0.0008 + vec2<f32>(wind_time * 0.05, 0.0), 3);
        let turbidity = turb_base * (0.3 + turb_noise * 0.7) * 0.18;
        ocean_color = mix(ocean_color, turbid_color, turbidity);
    }

    // =====================================================================
    // VAGUES DE SURFACE
    // =====================================================================

    let noise_uv = uv * 50.0;
    let time_looped = time * params.wave_speed;

    let wave1 = fbm(noise_uv * 0.5 + vec2<f32>(time_looped * 0.008, time_looped * 0.006), 4);
    let wave2 = fbm(noise_uv * 0.7 + vec2<f32>(-time_looped * 0.009, time_looped * 0.007), 4);
    let waves = (wave1 + wave2) * 0.5;

    ocean_color += vec3<f32>(waves * 0.4, waves * 0.6, waves) * 0.015;

    // =====================================================================
    // MICRO-VAGUELETTES — clapot chaotique, scintillement sur place
    // Pas de direction : oscillation temporelle, pas de scroll spatial
    // =====================================================================

    let ripple_time = time * 1.2;
    
    // Couche 1 : bruit statique modulé en amplitude par le temps
    let ripple1 = noise(world_pos * 0.15);
    let ripple2 = noise(world_pos * 0.18 + vec2<f32>(43.0, 17.0));
    let ripple3 = noise(world_pos * 0.35 + vec2<f32>(91.0, 53.0));
    
    // Modulation temporelle : chaque couche pulse à une fréquence différente
    // sin() fait scintiller sans déplacer
    let flicker1 = sin(ripple_time * 1.1 + ripple1 * TAU);
    let flicker2 = sin(ripple_time * 1.5 + ripple2 * TAU);
    let flicker3 = sin(ripple_time * 2.1 + ripple3 * TAU);
    
    let ripples = (flicker1 + flicker2) * 0.25 + flicker3 * 0.2;
    let ripple_highlight = ripples * 0.012;
    
    ocean_color += vec3<f32>(ripple_highlight * 0.6, ripple_highlight * 0.8, ripple_highlight);

    // =====================================================================
    // CAUSTIQUES — motifs lumineux sur le fond marin en eaux peu profondes
    // 2 couches de bruit au carré qui se déplacent lentement en sens opposé
    // =====================================================================

    let caustic_zone = 1.0 - smoothstep(0.0, 0.35, sdf_depth);
    if caustic_zone > 0.01 {
        let caustic_time = time * 0.4;
        let c_uv = world_pos * 0.12;

        // Couche 1 : dérive lente vers le sud-est
        let c1_raw = fbm(c_uv + vec2<f32>(caustic_time * 0.12, -caustic_time * 0.08), 4);
        let c1 = pow(c1_raw, 2.0); // Le carré crée des motifs en réseau lumineux

        // Couche 2 : dérive vers le nord-ouest (interférence)
        let c2_raw = fbm(c_uv * 1.3 + vec2<f32>(-caustic_time * 0.10, caustic_time * 0.13), 4);
        let c2 = pow(c2_raw, 2.0);

        // Combiner : les deux couches créent un motif d'interférence
        let caustics = (c1 + c2) * 0.5;

        // Intensité : forte très près de la côte, s'estompe avec la profondeur
        let caustic_intensity = caustic_zone * 0.24;

        // Teinte : dorée en très peu profond (fond sableux), bleutée un peu plus loin
        let caustic_tint = mix(
            vec3<f32>(0.3, 0.5, 0.7),   // Bleuté
            vec3<f32>(0.6, 0.55, 0.35),  // Doré (sable visible)
            1.0 - smoothstep(0.0, 0.15, sdf_depth)
        );

        ocean_color += caustic_tint * caustics * caustic_intensity;
    }

    // =====================================================================
    // BRUME DE PROFONDEUR — voile bleu-gris qui désature les eaux profondes
    // Réduit le contraste au large, renforce la lisibilité côte vs pleine mer
    // Appliqué APRÈS courants/vagues/caustiques pour les atténuer naturellement
    // mais AVANT l'écume (qui reste nette par-dessus)
    // =====================================================================

    let haze_color = vec3<f32>(0.12, 0.15, 0.19); // Bleu-gris brumeux
    // Brume pilotée par la profondeur réelle (SDF + heightmap)
    // Eau peu profonde = claire, eau profonde = brume, même loin de la côte
    let haze_depth = sdf_depth * 0.5 + bathymetry * 0.5;
    let haze_amount = smoothstep(0.02, 0.65, haze_depth) * 0.20;
    ocean_color = mix(ocean_color, haze_color, haze_amount);

    // =====================================================================
    // ÉCUME — riche, vitesse réaliste pour vue à ~20m
    // =====================================================================

    let foam_time = time * params.wave_speed * 0.7; // Pas de modulo = pas de reset

    // Zone d'écume modulée par bruit côtier
    let foam_outer = -0.95; // Assez large pour les vagues directionnelles au large
    let foam_zone = smoothstep(foam_outer, -0.05, sdf_signed)
                   * (1.0 - smoothstep(-0.05, 0.05, sdf_signed));

    if foam_zone > 0.01 {
        // Variations spatiales basse fréquence
        let phase_1 = fbm(world_pos * 0.006, 3) * TAU;
        let phase_2 = fbm(world_pos * 0.007 + vec2<f32>(100.0, 0.0), 3) * TAU;
        let amp_var = 0.5 + fbm(world_pos * 0.005, 3) * 0.5;
        let width_var = 0.7 + fbm(world_pos * 0.008 + vec2<f32>(50.0, 50.0), 3) * 0.6;

        // Vague principale — vitesse FIXE, seule la phase varie spatialement
        let w1 = sin(foam_time * 0.8 + phase_1);
        let wc1 = -0.05 + w1 * params.wave_amplitude * 0.8 * amp_var;
        let fhw1 = params.foam_width * 0.5 * width_var;
        // Fade aux extrêmes : très doux, seulement tout au bout de course
        let fade1 = 1.0 - pow(abs(w1), 6.0);
        let fi1 = (1.0 - smoothstep(0.0, fhw1, abs(sdf_signed - wc1))) * fade1;

        // Vague secondaire
        let w2 = sin(foam_time * 0.55 + phase_2);
        let wc2 = -0.15 + w2 * params.wave_amplitude * 0.4 * amp_var;
        let fhw2 = params.foam_width * 0.35 * width_var;
        let fade2 = 1.0 - pow(abs(w2), 6.0);
        let fi2 = (1.0 - smoothstep(0.0, fhw2, abs(sdf_signed - wc2))) * 0.5 * fade2;

        // Vagues directionnelles — bandes isolées du large vers le rivage
        // 4 vagues avec distances de départ/arrivée différentes et variables par cycle
        var fi_directional = 0.0;
        
        for (var wi = 0; wi < 4; wi++) {
            let wave_idx = f32(wi);
            let w_phase = fbm(world_pos * (0.003 + wave_idx * 0.001) + vec2<f32>(wave_idx * 37.0, wave_idx * 53.0), 3);
            
            // Vitesse réduite, légèrement différente par vague
            let w_speed = 0.05 + wave_idx * 0.012;
            let raw_cycle = foam_time * w_speed + w_phase;
            let w_cycle = fract(raw_cycle);
            
            // Index de cycle — modulo pour garder les hashes stables dans le temps
            let cycle_id = floor(raw_cycle) % 256.0;
            let cycle_rand = fract(sin(cycle_id * 43.758 + wave_idx * 17.31) * 12345.6789);
            let cycle_rand2 = fract(sin(cycle_id * 71.137 + wave_idx * 23.57) * 54321.9876);
            
            // Occurrence aléatoire : seulement ~35% des cycles produisent une vague
            let cycle_occur = fract(sin(cycle_id * 97.13 + wave_idx * 41.07) * 78901.2345);
            let wave_active = select(0.0, 1.0, cycle_occur < 0.35);
            
            // Distance de départ : varie par vague et par cycle
            // Vague 0 part de très loin, vague 3 part de moins loin
            let start_base = -0.85 + wave_idx * 0.08; // -0.85, -0.77, -0.69, -0.61
            let start_dist = start_base - cycle_rand * 0.15; // variation ±0.15 par cycle
            
            // Distance d'arrivée : s'arrête bien avant l'écume côtière
            let end_base = -0.50 + wave_idx * 0.04; // -0.50, -0.46, -0.42, -0.38
            let end_dist = end_base + cycle_rand2 * 0.08;
            
            // Position le long du trajet
            let travel = end_dist - start_dist;
            let w_center = start_dist + w_cycle * travel;
            
            // Fade in au départ, fade out à l'arrivée
            let w_fade = smoothstep(0.0, 0.15, w_cycle)
                       * (1.0 - smoothstep(0.75, 1.0, w_cycle));
            
            let w_half = params.foam_width * (0.06 + wave_idx * 0.008) * width_var;
            let w_intensity = 1.0 - smoothstep(0.0, w_half, abs(sdf_signed - w_center));
            
            // Bandes isolées — forme change à chaque cycle
            let band_seed = vec2<f32>(cycle_id * 7.13 + wave_idx * 31.0, cycle_id * 11.7 + wave_idx * 17.0);
            let band_noise = fbm(world_pos * (0.006 + wave_idx * 0.002) + band_seed, 3);
            let band_mask = smoothstep(0.40, 0.58, band_noise);
            
            fi_directional += w_intensity * w_fade * band_mask * wave_active * 0.25;
        }
        
        fi_directional = min(fi_directional, 0.6);

        var total_foam = min(fi1 + fi2 + fi_directional, 1.0) * foam_zone;

        // Liseré statique de base (collé à la côte)
        let static_foam = smoothstep(-0.06, -0.02, sdf_signed)
                        * (1.0 - smoothstep(-0.02, -0.005, sdf_signed))
                        * 0.25;
        total_foam = max(total_foam, static_foam);

        // Variation de couleur dans l'écume
        let fcv = fbm(world_pos * 0.012 + vec2<f32>(foam_time * 0.003, 0.0), 3);
        var foam_tinted = foam_color.rgb + vec3<f32>(
            (fcv - 0.5) * 0.05,
            (fcv - 0.5) * 0.03,
            (fcv - 0.5) * 0.02
        );

        ocean_color = mix(ocean_color, foam_tinted, total_foam);
    }

    // === Opacité ===
    let edge_opacity = smoothstep(0.02, -0.25, sdf_signed);

    return vec4<f32>(ocean_color, edge_opacity);
}
