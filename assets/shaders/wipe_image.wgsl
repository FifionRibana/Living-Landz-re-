#import bevy_ui::ui_vertex_output::UiVertexOutput

struct WipeUniforms {
    size: vec2<f32>,
    screen_size: vec2<f32>,
    edge_fade: f32,
    visibility: f32,
}

@group(1) @binding(0) var<uniform> material: WipeUniforms;
@group(1) @binding(1) var image_texture: texture_2d<f32>;
@group(1) @binding(2) var image_sampler: sampler;

@fragment
fn fragment(in: UiVertexOutput) -> @location(0) vec4<f32> {
    // 1. Sample de l'image
    let tex_color = textureSample(image_texture, image_sampler, in.uv);
    
    // 2. Calcul du Masque de Fenêtre (Wipe)
    // Même logique que pour le verre dépoli
    let pixel_x_in_container = (in.uv.x - 0.5) * material.size.x + material.edge_fade;
    
    // Limite du carousel (ex: 45% de la largeur de l'écran)
    let container_limit = material.screen_size.x * 0.45;
    let feather = 50.0;
    
    let window_mask = smoothstep(container_limit, container_limit - feather, abs(pixel_x_in_container));
    
    // 3. Alpha final
    // On combine l'alpha de l'image d'origine * la visibilité globale * le masque de wipe
    let final_alpha = tex_color.a * material.visibility * window_mask;
    
    return vec4<f32>(tex_color.rgb, final_alpha);
}