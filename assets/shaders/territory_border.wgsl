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

/// Distance minimale d'un point à un segment de droite
fn distance_to_segment(p: vec2<f32>, a: vec2<f32>, b: vec2<f32>) -> f32 {
    let pa = p - a;
    let ba = b - a;
    let h = clamp(dot(pa, ba) / dot(ba, ba), 0.0, 1.0);
    return length(pa - ba * h);
}

/// Test de ray-crossing : rayon horizontal vers +X depuis le point p.
/// Retourne 1 si le segment croise le rayon, 0 sinon.
fn ray_crosses_segment(p: vec2<f32>, a: vec2<f32>, b: vec2<f32>) -> i32 {
    // Le segment doit traverser la ligne y=p.y
    if (a.y <= p.y && b.y <= p.y) || (a.y > p.y && b.y > p.y) {
        return 0;
    }
    // Calculer l'intersection x du segment avec y=p.y
    let t = (p.y - a.y) / (b.y - a.y);
    let x_intersect = a.x + t * (b.x - a.x);
    // Le croisement doit être à droite du point
    if x_intersect > p.x {
        return 1;
    }
    return 0;
}

/// Teste si un point est à l'intérieur du contour (winding number)
fn is_inside(p: vec2<f32>) -> bool {
    var crossings = 0;
    let n = settings.num_segments;
    for (var i = 0u; i < n; i = i + 1u) {
        let seg = segments[i];
        crossings = crossings + ray_crosses_segment(p, seg.start, seg.end);
    }
    // Impair = intérieur
    return (crossings % 2) == 1;
}

/// Trouve la distance minimale au contour
fn min_distance_to_contour(p: vec2<f32>) -> f32 {
    var min_dist = 1e10;
    let n = settings.num_segments;
    for (var i = 0u; i < n; i = i + 1u) {
        let seg = segments[i];
        let dist = distance_to_segment(p, seg.start, seg.end);
        min_dist = min(min_dist, dist);
    }
    return min_dist;
}

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let world_pos = mesh.world_position.xy;
    
    let dist = min_distance_to_contour(world_pos);
    let inside = is_inside(world_pos);
    
    let max_render_dist = settings.border_width + settings.fade_distance;
    
    // Extérieur et trop loin → discard
    if !inside && dist > settings.border_width * 0.5 {
        discard;
    }
    
    // Zone de bordure (centrée sur le contour)
    let half_border = settings.border_width * 0.5;
    if dist < half_border {
        return vec4<f32>(settings.border_color.rgb, settings.border_color.a);
    }
    
    // Zone de fondu intérieur
    if inside {
        if dist > max_render_dist {
            discard;
        }
        let interior_dist = dist - half_border;
        let fade_factor = clamp(interior_dist / settings.fade_distance, 0.0, 1.0);
        let alpha = (1.0 - fade_factor) * settings.fill_color.a;
        
        if alpha < 0.01 {
            discard;
        }
        
        return vec4<f32>(settings.fill_color.rgb, alpha);
    }
    
    discard;
}