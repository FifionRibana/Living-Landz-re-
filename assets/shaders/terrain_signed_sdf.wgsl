#import bevy_sprite::mesh2d_vertex_output::VertexOutput

@group(2) @binding(0) var sdf_texture: texture_2d<f32>;
@group(2) @binding(1) var sdf_sampler: sampler;
@group(2) @binding(2) var<uniform> sand_color: vec4<f32>;
@group(2) @binding(3) var<uniform> grass_color: vec4<f32>;
@group(2) @binding(4) var<uniform> params: vec4<f32>;
@group(2) @binding(5) var road_sdf_texture: texture_2d<f32>;
@group(2) @binding(6) var road_sdf_sampler: sampler;
@group(2) @binding(7) var<uniform> road_params: vec4<f32>; // has_roads, edge_softness, noise_frequency, noise_amplitude
@group(2) @binding(8) var<uniform> road_color_light: vec4<f32>;
@group(2) @binding(9) var<uniform> road_color_dark: vec4<f32>;
@group(2) @binding(10) var<uniform> road_color_tracks: vec4<f32>;

// ============================================================================
// ROAD RENDER FUNCTIONS (inlined from road_render.wgsl)
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

// --- Fonction principale de rendu des routes ---

/// Applique le rendu des routes sur la couleur terrain
///
/// Paramètres :
/// - uv : coordonnées UV du fragment (0-1)
/// - world_pos : position dans l'espace monde
/// - terrain_color : couleur du terrain sous la route
/// - road_sdf_tex : texture SDF des routes (RG16)
/// - road_sampler : sampler pour la texture
/// - color_light : couleur terre claire (centre de la route)
/// - color_dark : couleur terre sombre (bords de la route)
/// - color_tracks : couleur des ornières
/// - edge_softness : largeur de la transition douce vers l'herbe
/// - noise_frequency : fréquence du bruit pour les bords organiques
/// - noise_amplitude : amplitude du bruit
///
/// Retourne la couleur finale (mélange terrain/route)
fn render_road(
    uv: vec2<f32>,
    world_pos: vec2<f32>,
    terrain_color: vec3<f32>,
    road_sdf_tex: texture_2d<f32>,
    road_sampler: sampler,
    color_light: vec3<f32>,
    color_dark: vec3<f32>,
    color_tracks: vec3<f32>,
    edge_softness: f32,
    noise_frequency: f32,
    noise_amplitude: f32,
) -> vec3<f32> {

    // Échantillonner le SDF
    let road_data = textureSample(road_sdf_tex, road_sampler, uv);

    // Décoder R16 et G16
    // R: Distance normalisée (0 = loin négatif, 0.5 = sur route, 1 = loin positif)
    // G: Métadonnées encodées
    let dist_normalized = road_data.r;
    let metadata = road_data.g;

    // Convertir la distance normalisée en distance signée
    // 0.5 = distance 0, <0.5 = négatif (dedans), >0.5 = positif (dehors)
    let max_dist = 50.0;
    let dist = (dist_normalized - 0.5) * 2.0 * max_dist;

    // Décoder les métadonnées
    // Bits 0-7: Importance * 64
    // Bit 8: Flag tracks
    // Bit 9: Flag intersection
    let metadata_u16 = u32(metadata * 65535.0);
    let importance = f32(metadata_u16 & 0xFF) / 64.0;
    let has_tracks = (metadata_u16 & 0x100) != 0;
    let in_intersection = (metadata_u16 & 0x200) != 0;

    // =======================================
    // Bruit pour bords organiques (CHEMIN DE TERRE)
    // =======================================

    // Bruit multi-échelle pour des bords TRÈS irréguliers
    let coarse_noise = fbm(world_pos * noise_frequency * 0.3, 3);  // Grandes variations
    let medium_noise = fbm(world_pos * noise_frequency * 1.0, 4);  // Variations moyennes
    let fine_noise = fbm(world_pos * noise_frequency * 3.0, 2);    // Détails fins

    // Combiner les bruits pour un effet très naturel
    let edge_noise = (
        (coarse_noise - 0.5) * noise_amplitude * 3.0 +
        (medium_noise - 0.5) * noise_amplitude * 1.5 +
        (fine_noise - 0.5) * noise_amplitude * 0.5
    );

    // Distance perturbée par le bruit
    let noisy_dist = dist + edge_noise;

    // =======================================
    // Masque de la route (bords TRÈS estompés)
    // =======================================

    // Transition très douce et irrégulière pour un chemin de terre
    let road_mask = 1.0 - smoothstep(-edge_softness * 1.5, edge_softness * 1.2, noisy_dist);

    // Sortie anticipée si hors route
    if (road_mask < 0.001) {
        return terrain_color;
    }

    // =======================================
    // Couleur de la route
    // =======================================

    // Gradient centre (clair) → bords (sombre)
    let center_factor = smoothstep(0.0, -4.0, noisy_dist);
    var road_color = mix(color_dark, color_light, center_factor);

    // Variation de teinte naturelle
    let color_variation = fbm(world_pos * 0.05, 2);
    road_color += vec3<f32>(
        (color_variation - 0.5) * 0.04,
        (color_variation - 0.5) * 0.025,
        (color_variation - 0.5) * 0.015
    );

    // =======================================
    // Ornières (DEUX SILLONS pour routes importantes)
    // =======================================

    if (has_tracks && !in_intersection) {
        // Calculer la distance perpendiculaire à la route
        // On utilise le gradient du SDF pour obtenir la direction perpendiculaire
        let dx = 0.5;
        let dist_x = (textureSample(road_sdf_tex, road_sampler, uv + vec2<f32>(dx / 1024.0, 0.0)).r - 0.5) * 2.0 * max_dist;
        let dist_y = (textureSample(road_sdf_tex, road_sampler, uv + vec2<f32>(0.0, dx / 1024.0)).r - 0.5) * 2.0 * max_dist;

        let gradient = normalize(vec2<f32>(dist_x - dist, dist_y - dist));
        let perpendicular = vec2<f32>(-gradient.y, gradient.x);

        // Calculer la distance perpendiculaire depuis le centre de la route
        let perp_dist = abs(dot(world_pos - floor(world_pos / 10.0) * 10.0, perpendicular));

        // Deux sillons parallèles espacés de ~3 unités
        let track_spacing = 3.0;
        let track_width = 0.8;

        // Distance au sillon gauche et droit
        let left_track_dist = abs(perp_dist - track_spacing);
        let right_track_dist = abs(perp_dist + track_spacing);

        // Intensité des sillons (0 = pas de sillon, 1 = sillon profond)
        let left_track = 1.0 - smoothstep(0.0, track_width, left_track_dist);
        let right_track = 1.0 - smoothstep(0.0, track_width, right_track_dist);
        let track_intensity = max(left_track, right_track);

        // Bruit pour variation le long des sillons
        let track_noise = fbm(world_pos * noise_frequency * 0.8 + vec2<f32>(42.0, 17.0), 3);
        let track_variation = (track_noise - 0.5) * 0.2;

        // Assombrir les sillons
        road_color = mix(road_color, color_tracks, track_intensity * (0.6 + track_variation));
    }

    // =======================================
    // Ajustements intersection
    // =======================================

    if (in_intersection) {
        // Placettes légèrement plus claires et uniformes (terre bien tassée)
        road_color = mix(road_color, color_light, 0.2);

        // Moins de variation dans les intersections
        let uniform_factor = 0.7;
        road_color = mix(road_color, color_light * 0.9, uniform_factor * center_factor);
    }

    // =======================================
    // Mélange final
    // =======================================

    return mix(terrain_color, road_color, road_mask);
}

/// Version simplifiée pour tester si on est sur une route
fn is_on_road(
    uv: vec2<f32>,
    road_sdf_tex: texture_2d<f32>,
    road_sampler: sampler,
) -> bool {
    let road_data = textureSample(road_sdf_tex, road_sampler, uv);
    let dist_normalized = road_data.r;
    let dist = (dist_normalized - 0.5) * 2.0 * 50.0;
    return dist < 2.0; // 2 pixels de marge
}


// ============================================================================
// FRAGMENT SHADER
// ============================================================================

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let beach_start = params.x;
    let beach_end = params.y;
    let has_coast = params.z;
    
    // Récupérer l'opacité depuis vertex color
    #ifdef VERTEX_COLORS
        let vertex_alpha = in.color.a;
    #else
        let vertex_alpha = 1.0;
    #endif
    
    if has_coast < 0.5 {
        return vec4<f32>(grass_color.rgb, vertex_alpha);
    }
    
    let uv_corrected = vec2<f32>(in.uv.x, in.uv.y);
    let sdf_raw = textureSample(sdf_texture, sdf_sampler, uv_corrected).r;
    
    // Décoder SDF signée
    let sdf_signed = (sdf_raw - 0.5) * 2.0;
    
    let t = smoothstep(beach_start, beach_end, sdf_signed);
    var final_color = mix(sand_color.rgb, grass_color.rgb, t);

    // Appliquer le rendu des routes si présentes
    let has_roads = road_params.x;
    if (has_roads > 0.5) {
        let edge_softness = road_params.y;
        let noise_frequency = road_params.z;
        let noise_amplitude = road_params.w;

        // Calculer la position monde approximative (chunk_size * uv)
        let chunk_size = vec2<f32>(600.0, 503.0);
        let world_pos = uv_corrected * chunk_size;

        final_color = render_road(
            uv_corrected,
            world_pos,
            final_color,
            road_sdf_texture,
            road_sdf_sampler,
            road_color_light.rgb,
            road_color_dark.rgb,
            road_color_tracks.rgb,
            edge_softness,
            noise_frequency,
            noise_amplitude
        );
    }

    // Appliquer l'opacité du vertex
    return vec4<f32>(final_color, vertex_alpha);
}