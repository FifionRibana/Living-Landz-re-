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
    
    // Appliquer le mapping sur chaque composante
    let uv_mapped = vec2<f32>(
        mix(uv_start, uv_end, in.uv.x),
        mix(uv_start, uv_end, in.uv.y)
    );
    
    let sdf_raw = textureSample(sdf_texture, sdf_sampler, uv_mapped).r;
    
    // Debug : afficher rouge/vert
    if sdf_raw > 0.5 {
        return vec4<f32>(0.0, sdf_raw, 0.0, 1.0);
    } else {
        return vec4<f32>(1.0 - sdf_raw, 0.0, 0.0, 1.0);
    }
}