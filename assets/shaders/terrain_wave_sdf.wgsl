// assets/shaders/terrain_sdf.wgsl

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
    let tex_res = 64.0;
    let overlap = 0.4;
    
    let total_span = tex_res - 1.0 + 2.0 * overlap;
    let uv_start = overlap / total_span;
    let uv_end = (tex_res - 1.0 + overlap) / total_span;
    
    let uv_mapped = vec2<f32>(
        mix(uv_start, uv_end, in.uv.x),
        mix(uv_start, uv_end, in.uv.y)
    );
    
    let sdf_raw = textureSample(sdf_texture, sdf_sampler, uv_mapped).r;
    let sdf_signed = (sdf_raw - 0.5) * 2.0;
    
    // Ne rendre que la terre
    if sdf_signed < -0.05 {
        discard;
    }
    
    // Couleur terrain (sable → herbe)
    let color_t = smoothstep(sdf_params.beach_start, sdf_params.beach_end, sdf_signed);
    let color = mix(sand_color.rgb, grass_color.rgb, color_t);
    
    // Opacité avec transition douce vers l'eau
    let opacity = smoothstep(-0.05, 0.1, sdf_signed);
    
    return vec4<f32>(color, opacity);
}