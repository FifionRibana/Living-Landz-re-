// assets/shaders/frosted_glass.wgsl

#import bevy_ui::ui_vertex_output::UiVertexOutput

struct FrostedGlassUniforms {
    color_top: vec4<f32>,
    color_bottom: vec4<f32>,
    opacity_top: f32,
    opacity_bottom: f32,
    edge_fade: f32,
    blur_strength: f32,
    border_radius: f32,
    size: vec2<f32>,
    screen_size: vec2<f32>,
    use_background_image: u32,
    _padding: vec2<f32>,
}

@group(1) @binding(0) var<uniform> material: FrostedGlassUniforms;
@group(1) @binding(1) var scene_texture: texture_2d<f32>;
@group(1) @binding(2) var scene_sampler: sampler;
@group(1) @binding(3) var background_image: texture_2d<f32>;
@group(1) @binding(4) var background_sampler: sampler;

fn sd_rounded_box(p: vec2<f32>, half_size: vec2<f32>, radius: f32) -> f32 {
    let r = min(radius, min(half_size.x, half_size.y));
    let q = abs(p) - half_size + vec2<f32>(r);
    return min(max(q.x, q.y), 0.0) + length(max(q, vec2<f32>(0.0))) - r;
}

fn sample_blur(tex: texture_2d<f32>, samp: sampler, uv: vec2<f32>, strength: f32) -> vec3<f32> {
    let tex_size = vec2<f32>(textureDimensions(tex));
    let texel = 1.0 / tex_size;
    let radius = strength * 5.0;
    
    let o1 = radius * 0.4 * texel;
    let o2 = radius * 0.8 * texel;
    let o3 = radius * 1.2 * texel;
    
    var color = vec3<f32>(0.0);
    
    // Centre
    color += textureSampleLevel(tex, samp, uv, 0.0).rgb * 4.0;
    
    // Anneau 1
    color += textureSampleLevel(tex, samp, uv + vec2<f32>(o1.x, 0.0), 0.0).rgb * 2.0;
    color += textureSampleLevel(tex, samp, uv + vec2<f32>(-o1.x, 0.0), 0.0).rgb * 2.0;
    color += textureSampleLevel(tex, samp, uv + vec2<f32>(0.0, o1.y), 0.0).rgb * 2.0;
    color += textureSampleLevel(tex, samp, uv + vec2<f32>(0.0, -o1.y), 0.0).rgb * 2.0;
    color += textureSampleLevel(tex, samp, uv + vec2<f32>(o1.x, o1.y) * 0.707, 0.0).rgb * 1.5;
    color += textureSampleLevel(tex, samp, uv + vec2<f32>(-o1.x, o1.y) * 0.707, 0.0).rgb * 1.5;
    color += textureSampleLevel(tex, samp, uv + vec2<f32>(o1.x, -o1.y) * 0.707, 0.0).rgb * 1.5;
    color += textureSampleLevel(tex, samp, uv + vec2<f32>(-o1.x, -o1.y) * 0.707, 0.0).rgb * 1.5;
    
    // Anneau 2
    color += textureSampleLevel(tex, samp, uv + vec2<f32>(o2.x, 0.0), 0.0).rgb * 1.0;
    color += textureSampleLevel(tex, samp, uv + vec2<f32>(-o2.x, 0.0), 0.0).rgb * 1.0;
    color += textureSampleLevel(tex, samp, uv + vec2<f32>(0.0, o2.y), 0.0).rgb * 1.0;
    color += textureSampleLevel(tex, samp, uv + vec2<f32>(0.0, -o2.y), 0.0).rgb * 1.0;
    color += textureSampleLevel(tex, samp, uv + vec2<f32>(o2.x, o2.y) * 0.707, 0.0).rgb * 0.75;
    color += textureSampleLevel(tex, samp, uv + vec2<f32>(-o2.x, o2.y) * 0.707, 0.0).rgb * 0.75;
    color += textureSampleLevel(tex, samp, uv + vec2<f32>(o2.x, -o2.y) * 0.707, 0.0).rgb * 0.75;
    color += textureSampleLevel(tex, samp, uv + vec2<f32>(-o2.x, -o2.y) * 0.707, 0.0).rgb * 0.75;
    
    // Anneau 3
    color += textureSampleLevel(tex, samp, uv + vec2<f32>(o3.x, 0.0), 0.0).rgb * 0.5;
    color += textureSampleLevel(tex, samp, uv + vec2<f32>(-o3.x, 0.0), 0.0).rgb * 0.5;
    color += textureSampleLevel(tex, samp, uv + vec2<f32>(0.0, o3.y), 0.0).rgb * 0.5;
    color += textureSampleLevel(tex, samp, uv + vec2<f32>(0.0, -o3.y), 0.0).rgb * 0.5;
    color += textureSampleLevel(tex, samp, uv + vec2<f32>(o3.x, o3.y) * 0.707, 0.0).rgb * 0.3;
    color += textureSampleLevel(tex, samp, uv + vec2<f32>(-o3.x, o3.y) * 0.707, 0.0).rgb * 0.3;
    color += textureSampleLevel(tex, samp, uv + vec2<f32>(o3.x, -o3.y) * 0.707, 0.0).rgb * 0.3;
    color += textureSampleLevel(tex, samp, uv + vec2<f32>(-o3.x, -o3.y) * 0.707, 0.0).rgb * 0.3;
    
    return color / 28.2;
}

@fragment
fn fragment(in: UiVertexOutput) -> @location(0) vec4<f32> {
    let screen_uv = in.position.xy / material.screen_size;
    
    // Blur
    var blurred_color: vec3<f32>;
    if material.use_background_image == 1u {
        blurred_color = sample_blur(background_image, background_sampler, screen_uv, material.blur_strength);
    } else {
        blurred_color = sample_blur(scene_texture, scene_sampler, screen_uv, material.blur_strength);
    }
    
    // Désaturation
    let luma = dot(blurred_color, vec3<f32>(0.299, 0.587, 0.114));
    blurred_color = mix(blurred_color, vec3<f32>(luma), 0.12);
    
    // Gradient
    let t = in.uv.y;
    let gradient_color = mix(material.color_top.rgb, material.color_bottom.rgb, t);
    // let tint_opacity = mix(-0.15, 0.7, t);
    let raw_opacity = mix(material.opacity_top, material.opacity_bottom, t);
    let tint_opacity = -0.2 + raw_opacity * 0.8; // 0 → -0.15, 1 → 0.7
    let final_color = mix(blurred_color, gradient_color, clamp(tint_opacity, 0.0, 1.0));
    
    // Highlight en haut
    let top_highlight = smoothstep(0.1, 0.0, t) * 0.05;
    let output_color = final_color + vec3<f32>(top_highlight);

    // Border radius
    var border_alpha = 1.0;
    if material.border_radius > 0.0 && material.size.x > 0.0 {
        let half_size = material.size * 0.5;
        let local_pos = (in.uv - 0.5) * material.size;
        let dist = sd_rounded_box(local_pos, half_size, material.border_radius);
        border_alpha = 1.0 - smoothstep(-1.5, 0.5, dist);
    }
    
    // Edge fade (carousel)
    var edge_alpha = 1.0;
    if material.edge_fade != 0.0 {
        let fade_amount = abs(material.edge_fade);
        if material.edge_fade < 0.0 {
            edge_alpha = smoothstep(0.0, fade_amount, in.uv.x);
        } else {
            edge_alpha = smoothstep(0.0, fade_amount, 1.0 - in.uv.x);
        }
    }
    
    // let output_alpha = border_alpha * edge_alpha;

    // --- Calcul du Fade Horizontal (Carousel) ---
    var horizontal_fade = 1.0;
    
    if (material.edge_fade > 0.0) {
        // Cas : Carte à droite (edge_fade positif)
        // On transitionne de 1.0 (opaque à gauche de la carte) vers 0.0 (transparent à droite)
        // L'intensité (edge_fade) détermine à quel point le bord est "grignoté"
        horizontal_fade = mix(1.0, 1.0 - in.uv.x, material.edge_fade);
    } else if (material.edge_fade < 0.0) {
        // Cas : Carte à gauche (edge_fade négatif)
        // On transitionne de 0.0 (transparent à gauche) vers 1.0 (opaque à droite)
        let intensity = abs(material.edge_fade);
        horizontal_fade = mix(1.0, in.uv.x, intensity);
    }

    // --- Calcul de l'Alpha Final ---
    // On combine : 
    // 1. Le gradient vertical (opacity_top/bottom)
    // 2. Le border radius (arrondis)
    // 3. Le fade horizontal (carousel)
    
    let vertical_alpha = mix(material.opacity_top, material.opacity_bottom, t);
    // let final_alpha = vertical_alpha * border_alpha * horizontal_fade * edge_alpha;
    let final_alpha = border_alpha * horizontal_fade * edge_alpha;

    return vec4<f32>(output_color, final_alpha);
    
    // return vec4<f32>(output_color, output_alpha);
}