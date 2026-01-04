#import bevy_sprite::mesh2d_vertex_output::VertexOutput

struct ChunkContourSettings {
    num_segments: u32,
    border_width: f32,
    fade_distance: f32,
    _padding: f32,
    border_color: vec4<f32>,
    fill_color: vec4<f32>,
}

struct ContourSegment {
    start: vec2<f32>,
    end: vec2<f32>,
    normal: vec2<f32>,
    _padding: vec2<f32>,
}

@group(2) @binding(0)
var<uniform> settings: ChunkContourSettings;

@group(2) @binding(1)
var<storage, read> segments: array<ContourSegment>;

/// Distance signée d'un point à un segment, avec signe basé sur la normale
/// Retourne: (distance au segment, distance signée perpendiculaire)
fn signed_distance_to_segment(p: vec2<f32>, seg: ContourSegment) -> vec2<f32> {
    let pa = p - seg.start;
    let ba = seg.end - seg.start;
    let h = clamp(dot(pa, ba) / dot(ba, ba), 0.0, 1.0);
    let closest = seg.start + ba * h;
    
    let dist = length(p - closest);
    
    // Distance signée : positive si du côté de la normale (intérieur), négative sinon
    let to_point = p - closest;
    let signed_dist = sign(dot(to_point, seg.normal)) * dist;
    
    return vec2<f32>(dist, signed_dist);
}

/// Trouve le segment le plus proche et retourne la distance signée
fn find_closest_segment(p: vec2<f32>) -> vec2<f32> {
    var min_dist = 1e10;
    var best_signed_dist = 0.0;
    
    let n = settings.num_segments;
    
    for (var i = 0u; i < n; i = i + 1u) {
        let seg = segments[i];
        let result = signed_distance_to_segment(p, seg);
        let dist = result.x;
        let signed_dist = result.y;
        
        if (dist < min_dist) {
            min_dist = dist;
            best_signed_dist = signed_dist;
        }
    }
    
    return vec2<f32>(min_dist, best_signed_dist);
}

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let world_pos = mesh.world_position.xy;
    
    let result = find_closest_segment(world_pos);
    let dist = result.x;           // Distance absolue au contour
    let signed_dist = result.y;    // Distance signée (positif = intérieur)
    
    // Si trop loin du contour, discard
    let max_render_dist = settings.border_width + settings.fade_distance;
    if (dist > max_render_dist) {
        discard;
    }
    
    // Si du côté extérieur (signed_dist < 0), ne pas rendre
    if (signed_dist < -settings.border_width * 0.5) {
        discard;
    }
    
    // Zone du contour noir (centré sur la ligne)
    let half_border = settings.border_width * 0.5;
    if (dist < half_border) {
        // Anti-aliasing
        let aa = smoothstep(0.0, 1.0, dist / half_border);
        return vec4<f32>(settings.border_color.rgb, settings.border_color.a);
    }
    
    // Zone du fondu (uniquement vers l'intérieur)
    if (signed_dist > 0.0) {
        let interior_dist = dist - half_border;
        let fade_factor = clamp(interior_dist / settings.fade_distance, 0.0, 1.0);
        let alpha = (1.0 - fade_factor) * settings.fill_color.a;
        
        if (alpha < 0.01) {
            discard;
        }
        
        return vec4<f32>(settings.fill_color.rgb, alpha);
    }
    
    discard;
}