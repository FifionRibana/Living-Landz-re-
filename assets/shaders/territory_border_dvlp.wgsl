#import bevy_sprite::mesh2d_vertex_output::VertexOutput

struct TerritorySettings {
    num_points: u32,
    border_width: f32,      // Épaisseur du contour noir (ex: 2.0 ou 3.0)
    fade_distance: f32,     // Distance du fondu intérieur
    _padding: f32,
    border_color: vec4<f32>,  // Couleur du contour (noir)
    fill_color: vec4<f32>,    // Couleur du fondu intérieur
}

struct ContourPoint {
    position: vec2<f32>,
    _padding: vec2<f32>,
}

@group(2) @binding(0)
var<uniform> settings: TerritorySettings;

@group(2) @binding(1)
var<storage, read> contour_points: array<ContourPoint>;

fn distance_to_segment(p: vec2<f32>, a: vec2<f32>, b: vec2<f32>) -> f32 {
    let pa = p - a;
    let ba = b - a;
    let h = clamp(dot(pa, ba) / dot(ba, ba), 0.0, 1.0);
    return length(pa - ba * h);
}

fn signed_distance_to_polygon(p: vec2<f32>) -> f32 {
    var min_dist = 1e10;
    var winding = 0;
    
    let n = settings.num_points;
    
    for (var i = 0u; i < n; i = i + 1u) {
        let j = (i + 1u) % n;
        
        let a = contour_points[i].position;
        let b = contour_points[j].position;
        
        let d = distance_to_segment(p, a, b);
        min_dist = min(min_dist, d);
        
        if (a.y <= p.y) {
            if (b.y > p.y) {
                let cross = (b.x - a.x) * (p.y - a.y) - (b.y - a.y) * (p.x - a.x);
                if (cross > 0.0) {
                    winding = winding + 1;
                }
            }
        } else {
            if (b.y <= p.y) {
                let cross = (b.x - a.x) * (p.y - a.y) - (b.y - a.y) * (p.x - a.x);
                if (cross < 0.0) {
                    winding = winding - 1;
                }
            }
        }
    }
    
    let sign = select(1.0, -1.0, winding != 0);
    return sign * min_dist;
}

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let world_pos = mesh.world_position.xy;
    let sdf = signed_distance_to_polygon(world_pos);
    
    // sdf > 0 : extérieur
    // sdf < 0 : intérieur
    // sdf = 0 : exactement sur la frontière
    
    // Extérieur → transparent
    if (sdf > 0.0) {
        discard;
    }
    
    // Distance vers l'intérieur (valeur positive)
    let interior_dist = -sdf;
    
    // Zone du contour noir (de 0 à border_width pixels vers l'intérieur)
    if (interior_dist < settings.border_width) {
        return settings.border_color;
    }
    
    // Zone du fondu (de border_width à border_width + fade_distance)
    let fade_start = settings.border_width;
    let fade_end = settings.border_width + settings.fade_distance;
    
    // Calculer le facteur de fondu : 0 au début du fondu, 1 à la fin
    let t = (interior_dist - fade_start) / settings.fade_distance;
    let fade_factor = clamp(t, 0.0, 1.0);
    
    // Alpha : opaque (1.0) près du contour → transparent (0.0) vers l'intérieur
    let alpha = 1.0 - fade_factor;
    
    // Si complètement transparent, discard
    if (alpha < 0.01) {
        discard;
    }
    
    return vec4<f32>(settings.fill_color.rgb, settings.fill_color.a * alpha);
}