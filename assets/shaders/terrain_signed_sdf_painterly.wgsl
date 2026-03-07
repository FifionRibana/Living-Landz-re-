#import bevy_sprite::mesh2d_vertex_output::VertexOutput

@group(2) @binding(0) var sdf_texture: texture_2d<f32>;
@group(2) @binding(1) var sdf_sampler: sampler;
@group(2) @binding(2) var<uniform> sand_color: vec4<f32>;
@group(2) @binding(3) var<uniform> grass_color: vec4<f32>;
@group(2) @binding(4) var<uniform> params: vec4<f32>;
@group(2) @binding(5) var road_sdf_texture: texture_2d<f32>;
@group(2) @binding(6) var road_sdf_sampler: sampler;
@group(2) @binding(7) var<uniform> road_params: vec4<f32>;
@group(2) @binding(8) var<uniform> road_color_light: vec4<f32>;
@group(2) @binding(9) var<uniform> road_color_dark: vec4<f32>;
@group(2) @binding(10) var<uniform> road_color_tracks: vec4<f32>;
@group(2) @binding(11) var<uniform> chunk_info: vec4<f32>; // x,y = world offset; z,w = chunk width,height

// ============================================================================
// CONSTANTES PALETTE PAINTERLY
// Couleurs désaturées et terreuses pour un rendu organique
// ============================================================================

// --- Végétation (4 teintes mélangées par bruit) ---
const FOREST_DARK:   vec3<f32> = vec3<f32>(0.16, 0.24, 0.11);  // Sous-bois dense
const FOREST_MID:    vec3<f32> = vec3<f32>(0.24, 0.33, 0.16);  // Forêt tempérée
const MEADOW:        vec3<f32> = vec3<f32>(0.34, 0.40, 0.22);  // Prairie
const DRY_GRASS:     vec3<f32> = vec3<f32>(0.42, 0.39, 0.24);  // Herbe sèche / foin

// --- Sable multi-zones ---
const SAND_WET:      vec3<f32> = vec3<f32>(0.45, 0.40, 0.30);  // Sable mouillé (près de l'eau)
const SAND_DRY:      vec3<f32> = vec3<f32>(0.68, 0.62, 0.45);  // Sable sec
const SAND_GRASS_MIX: vec3<f32> = vec3<f32>(0.48, 0.46, 0.30); // Zone de transition sable/herbe

// --- Roche (pour futures falaises) ---
const ROCK_LIGHT:    vec3<f32> = vec3<f32>(0.52, 0.48, 0.42);
const ROCK_DARK:     vec3<f32> = vec3<f32>(0.35, 0.32, 0.28);

// ============================================================================
// FONCTIONS DE BRUIT
// ============================================================================

fn hash21(p: vec2<f32>) -> f32 {
    var p3 = fract(vec3<f32>(p.x, p.y, p.x) * 0.1031);
    p3 += dot(p3, p3.yzx + 33.33);
    return fract((p3.x + p3.y) * p3.z);
}

fn noise2d(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f);

    return mix(
        mix(hash21(i), hash21(i + vec2<f32>(1.0, 0.0)), u.x),
        mix(hash21(i + vec2<f32>(0.0, 1.0)), hash21(i + vec2<f32>(1.0, 1.0)), u.x),
        u.y
    );
}

fn fbm(p: vec2<f32>, octaves: i32) -> f32 {
    var value = 0.0;
    var amplitude = 0.5;
    var frequency = 1.0;
    var pos = p;

    for (var i = 0; i < octaves; i++) {
        value += amplitude * noise2d(pos * frequency);
        frequency *= 2.0;
        amplitude *= 0.5;
    }
    return value;
}

// Bruit avec rotation entre octaves pour réduire les artefacts d'alignement grille
fn fbm_rotated(p: vec2<f32>, octaves: i32) -> f32 {
    var value = 0.0;
    var amplitude = 0.5;
    var pos = p;
    // Matrice de rotation ~37° entre chaque octave
    let rot = mat2x2<f32>(0.8, 0.6, -0.6, 0.8);

    for (var i = 0; i < octaves; i++) {
        value += amplitude * noise2d(pos);
        pos = rot * pos * 2.0;
        amplitude *= 0.5;
    }
    return value;
}

// ============================================================================
// VEGETATION PAINTERLY
// Mélange organique de 4 teintes piloté par bruit multi-échelle
// ============================================================================

fn painterly_vegetation(world_pos: vec2<f32>, base_green: vec3<f32>) -> vec3<f32> {
    // ---- Couche 1 : grandes taches (prairies vs forêts) ----
    // Fréquence très basse = grands patchs de ~120-200 pixels
    let large_noise = fbm_rotated(world_pos * 0.005, 4);
    
    // ---- Couche 2 : variation moyenne (bosquets, clairières) ----
    // Fréquence moyenne = patchs de ~40-60 pixels
    let medium_noise = fbm_rotated(world_pos * 0.018 + vec2<f32>(43.7, 91.2), 4);
    
    // ---- Couche 3 : micro-détails (touffes d'herbe) ----
    // Haute fréquence = variation pixel à pixel presque
    let detail_noise = fbm(world_pos * 0.06 + vec2<f32>(17.3, 53.1), 3);
    
    // ---- Couche 4 : "coups de pinceau" directionnels ----
    // Bruit étiré dans une direction pour simuler des stries de peinture
    let brush_pos = vec2<f32>(world_pos.x * 0.03, world_pos.y * 0.012);
    let brush_stroke = fbm(brush_pos + vec2<f32>(77.0, 33.0), 3);
    
    // Mélange des 4 teintes de vert
    // Grande échelle : forêt sombre vs prairie claire
    var color = mix(MEADOW, FOREST_MID, smoothstep(0.35, 0.65, large_noise));
    
    // Moyenne échelle : patches de forêt dense ou d'herbe sèche
    color = mix(color, FOREST_DARK, smoothstep(0.58, 0.78, medium_noise) * 0.6);
    color = mix(color, DRY_GRASS, smoothstep(0.60, 0.80, 1.0 - medium_noise) * 0.35);
    
    // Micro-détail : variation de luminosité (±12%)
    color *= 0.88 + detail_noise * 0.24;
    
    // Coups de pinceau : légère variation de teinte chaude/froide
    let warm_shift = (brush_stroke - 0.5) * 0.08;
    color += vec3<f32>(warm_shift, warm_shift * 0.3, -warm_shift * 0.5);
    
    // Moduler subtilement avec la couleur de base passée en uniform
    // (permet de garder un contrôle côté Rust)
    color = mix(color, base_green, 0.15);
    
    return color;
}

// ============================================================================
// TRANSITION PLAGE MULTI-ZONES
// Sable mouillé → sable sec → transition herbue → végétation
// Avec bords irréguliers painterly
// ============================================================================

fn painterly_beach_transition(
    sdf_signed: f32,
    world_pos: vec2<f32>,
    beach_start: f32,
    beach_end: f32,
    vegetation_color: vec3<f32>,
    base_sand: vec3<f32>,
) -> vec3<f32> {
    
    // Bruit pour casser les transitions en lignes irrégulières
    let edge_noise = fbm_rotated(world_pos * 0.025, 4);
    let edge_offset = (edge_noise - 0.5) * 0.18; // ±0.09 de décalage SDF
    
    // Bruit secondaire pour les taches de végétation dans le sable
    let patch_noise = fbm(world_pos * 0.04 + vec2<f32>(200.0, 100.0), 3);
    
    // SDF perturbée par le bruit (bords organiques)
    let sdf_noisy = sdf_signed + edge_offset;
    
    // ---- Zone 1 : Sable mouillé (très proche de l'eau) ----
    // De beach_start jusqu'à ~40% du chemin
    let wet_end = beach_start + (beach_end - beach_start) * 0.3;
    let wet_t = smoothstep(beach_start - 0.02, wet_end, sdf_noisy);
    
    // ---- Zone 2 : Sable sec ----
    // De 40% à 70%
    let dry_end = beach_start + (beach_end - beach_start) * 0.7;
    let dry_t = smoothstep(wet_end, dry_end, sdf_noisy);
    
    // ---- Zone 3 : Transition sable-herbe (moucheté) ----
    // De 70% à 100% + un peu au-delà
    let grass_start = dry_end;
    let grass_end = beach_end + 0.08;
    let grass_t = smoothstep(grass_start, grass_end, sdf_noisy);
    
    // Couleur sable mouillé = sable de base assombri
    let wet_sand = base_sand * 0.65 + vec3<f32>(0.02, 0.03, 0.05);
    
    // Construction progressive de la couleur
    var color = mix(wet_sand, base_sand, wet_t);
    color = mix(color, SAND_GRASS_MIX, dry_t * 0.4);
    
    // Zone de transition : taches de végétation dans le sable
    // Le bruit crée des "touffes" irrégulières
    let tuft_mask = smoothstep(0.45, 0.65, patch_noise) * grass_t;
    color = mix(color, vegetation_color * 0.85, tuft_mask);
    
    // Transition finale vers la végétation pleine
    color = mix(color, vegetation_color, grass_t * (1.0 - tuft_mask) * 0.9 + tuft_mask);
    
    // Micro-variation de teinte sur le sable (grains, coquillages, algues)
    let sand_detail = fbm(world_pos * 0.1, 2);
    let in_sand = 1.0 - grass_t;
    color += vec3<f32>(
        (sand_detail - 0.5) * 0.04 * in_sand,
        (sand_detail - 0.5) * 0.02 * in_sand,
        (sand_detail - 0.5) * 0.01 * in_sand
    );
    
    return color;
}

// ============================================================================
// AMBIENT OCCLUSION APPROXIMATIF VIA SDF
// Les zones proches des côtes/bordures sont légèrement assombries
// ============================================================================

fn sdf_ambient_occlusion(sdf_signed: f32) -> f32 {
    // AO douce : les zones très proches de la frontière terre/eau sont plus sombres
    // Simule l'ombre dans le "creux" côtier
    let dist_to_edge = abs(sdf_signed);
    let ao = smoothstep(0.0, 0.12, dist_to_edge);
    // Retourne un facteur multiplicatif (0.82 au bord → 1.0 loin)
    return 0.82 + ao * 0.18;
}

// ============================================================================
// ROAD RENDER FUNCTIONS (inchangé)
// ============================================================================

fn render_road(
    uv: vec2<f32>,
    world_pos: vec2<f32>,
    terrain_color: vec3<f32>,
    road_sdf_tex: texture_2d<f32>,
    road_sampler_arg: sampler,
    color_light: vec3<f32>,
    color_dark: vec3<f32>,
    color_tracks: vec3<f32>,
    edge_softness: f32,
    noise_frequency: f32,
    noise_amplitude: f32,
) -> vec3<f32> {

    let road_data = textureSample(road_sdf_tex, road_sampler_arg, uv);
    let dist_normalized = road_data.r;
    let metadata = road_data.g;

    let max_dist = 50.0;
    let dist = (dist_normalized - 0.5) * 2.0 * max_dist;

    let metadata_u16 = u32(metadata * 65535.0);
    let importance = f32(metadata_u16 & 0xFF) / 64.0;
    let has_tracks = (metadata_u16 & 0x100) != 0;
    let in_intersection = (metadata_u16 & 0x200) != 0;

    // Bruit multi-échelle pour bords organiques
    let coarse_noise = fbm(world_pos * noise_frequency * 0.3, 3);
    let medium_noise = fbm(world_pos * noise_frequency * 1.0, 4);
    let fine_noise = fbm(world_pos * noise_frequency * 3.0, 2);

    let edge_noise = (
        (coarse_noise - 0.5) * noise_amplitude * 3.0 +
        (medium_noise - 0.5) * noise_amplitude * 1.5 +
        (fine_noise - 0.5) * noise_amplitude * 0.5
    );

    let noisy_dist = dist + edge_noise;

    let road_mask = 1.0 - smoothstep(-edge_softness * 1.5, edge_softness * 1.2, noisy_dist);

    if (road_mask < 0.001) {
        return terrain_color;
    }

    let center_factor = smoothstep(0.0, -4.0, noisy_dist);
    var road_color = mix(color_dark, color_light, center_factor);

    let color_variation = fbm(world_pos * 0.05, 2);
    road_color += vec3<f32>(
        (color_variation - 0.5) * 0.04,
        (color_variation - 0.5) * 0.025,
        (color_variation - 0.5) * 0.015
    );

    if (has_tracks && !in_intersection) {
        let dx = 0.5;
        let dist_x = (textureSample(road_sdf_tex, road_sampler_arg, uv + vec2<f32>(dx / 1024.0, 0.0)).r - 0.5) * 2.0 * max_dist;
        let dist_y = (textureSample(road_sdf_tex, road_sampler_arg, uv + vec2<f32>(0.0, dx / 1024.0)).r - 0.5) * 2.0 * max_dist;

        let gradient = normalize(vec2<f32>(dist_x - dist, dist_y - dist));
        let perpendicular = vec2<f32>(-gradient.y, gradient.x);
        let perp_dist = abs(dot(world_pos - floor(world_pos / 10.0) * 10.0, perpendicular));

        let track_spacing = 3.0;
        let track_width = 0.8;
        let left_track_dist = abs(perp_dist - track_spacing);
        let right_track_dist = abs(perp_dist + track_spacing);
        let left_track = 1.0 - smoothstep(0.0, track_width, left_track_dist);
        let right_track = 1.0 - smoothstep(0.0, track_width, right_track_dist);
        let track_intensity = max(left_track, right_track);

        let track_noise = fbm(world_pos * noise_frequency * 0.8 + vec2<f32>(42.0, 17.0), 3);
        let track_variation = (track_noise - 0.5) * 0.2;
        road_color = mix(road_color, color_tracks, track_intensity * (0.6 + track_variation));
    }

    if (in_intersection) {
        road_color = mix(road_color, color_light, 0.2);
        let uniform_factor = 0.7;
        road_color = mix(road_color, color_light * 0.9, uniform_factor * center_factor);
    }

    return mix(terrain_color, road_color, road_mask);
}

fn is_on_road(
    uv: vec2<f32>,
    road_sdf_tex: texture_2d<f32>,
    road_sampler_arg: sampler,
) -> bool {
    let road_data = textureSample(road_sdf_tex, road_sampler_arg, uv);
    let dist_normalized = road_data.r;
    let dist = (dist_normalized - 0.5) * 2.0 * 50.0;
    return dist < 2.0;
}


// ============================================================================
// FRAGMENT SHADER — PAINTERLY
// ============================================================================

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let beach_start = params.x;
    let beach_end = params.y;
    let has_coast = params.z;
    
    #ifdef VERTEX_COLORS
        let vertex_alpha = in.color.a;
    #else
        let vertex_alpha = 1.0;
    #endif
    
    let uv_corrected = vec2<f32>(in.uv.x, in.uv.y);
    
    // Position monde pour le bruit (calculée tôt, utilisée partout)
    // Position monde GLOBALE (offset du chunk + position locale)
    // Ceci assure la continuité du bruit entre les chunks
    let chunk_offset = vec2<f32>(chunk_info.x, chunk_info.y);
    let chunk_size = vec2<f32>(chunk_info.z, chunk_info.w);
    let world_pos = chunk_offset + uv_corrected * chunk_size;
    
    // ---- Végétation painterly (toujours calculée) ----
    let vegetation = painterly_vegetation(world_pos, grass_color.rgb);
    
    // ---- Chunk sans côte : 100% végétation ----
    if has_coast < 0.5 {
        var color = vegetation;
        
        // Appliquer les routes même sur les chunks sans côte
        let has_roads = road_params.x;
        if (has_roads > 0.5) {
            color = render_road(
                uv_corrected, world_pos, color,
                road_sdf_texture, road_sdf_sampler,
                road_color_light.rgb, road_color_dark.rgb, road_color_tracks.rgb,
                road_params.y, road_params.z, road_params.w
            );
        }
        
        return vec4<f32>(color, vertex_alpha);
    }
    
    // ---- Chunk côtier : SDF + transition multi-zones ----
    let sdf_raw = textureSample(sdf_texture, sdf_sampler, uv_corrected).r;
    let sdf_signed = (sdf_raw - 0.5) * 2.0;
    
    // Transition plage painterly (sable mouillé → sec → touffes → végétation)
    var final_color = painterly_beach_transition(
        sdf_signed,
        world_pos,
        beach_start,
        beach_end,
        vegetation,
        sand_color.rgb * 0.92, // Légèrement désaturer le sable de base
    );
    
    // Ambient occlusion côtière
    let ao = sdf_ambient_occlusion(sdf_signed);
    final_color *= ao;

    // ---- Routes ----
    let has_roads = road_params.x;
    if (has_roads > 0.5) {
        final_color = render_road(
            uv_corrected, world_pos, final_color,
            road_sdf_texture, road_sdf_sampler,
            road_color_light.rgb, road_color_dark.rgb, road_color_tracks.rgb,
            road_params.y, road_params.z, road_params.w
        );
    }

    return vec4<f32>(final_color, vertex_alpha);
}
