#import bevy_sprite::mesh2d_vertex_output::VertexOutput

struct SdfParams {
    beach_start: f32,
    beach_end: f32,
    opacity_start: f32,
    opacity_end: f32,
}

@group(2) @binding(0) var sdf_texture: texture_2d<f32>;
@group(2) @binding(1) var sdf_sampler: sampler;
@group(2) @binding(2) var<uniform> sand_color: vec4<f32>;
@group(2) @binding(3) var<uniform> grass_color: vec4<f32>;
@group(2) @binding(4) var<uniform> sdf_params: SdfParams;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // La texture overlap d'un demi-texel de chaque côté
    // On doit mapper UV [0, 1] → [0.5/64, 63.5/64] pour centrer sur la zone du chunk
    let tex_res = 64.0;
    let half_texel = 0.9 / tex_res;
    
    // Centrer: décaler de half_texel et réduire l'échelle
    let uv_centered = in.uv * (1.0 - 2.0 * half_texel) + half_texel;
    
    // Correction Y (flip vertical)
    let uv_corrected = vec2<f32>(uv_centered.x, uv_centered.y);
    
    let sdf_raw = textureSample(sdf_texture, sdf_sampler, uv_corrected).r;
    
    // Debug : afficher rouge/vert
    if sdf_raw > 0.5 {
        return vec4<f32>(0.0, sdf_raw, 0.0, 1.0);
    } else {
        return vec4<f32>(1.0 - sdf_raw, 0.0, 0.0, 1.0);
    }
}