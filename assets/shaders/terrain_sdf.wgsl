// assets/shaders/terrain_sdf.wgsl

#import bevy_sprite::mesh2d_vertex_output::VertexOutput

@group(2) @binding(0) var sdf_texture: texture_2d<f32>;
@group(2) @binding(1) var sdf_sampler: sampler;
@group(2) @binding(2) var<uniform> sand_color: vec4<f32>;
@group(2) @binding(3) var<uniform> grass_color: vec4<f32>;
@group(2) @binding(4) var<uniform> params: vec4<f32>;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let beach_start = params.x;
    let beach_end = params.y;
    let has_coast = params.z;
    
    if has_coast < 0.5 {
        return grass_color;
    }
    
    let sdf_value = textureSample(sdf_texture, sdf_sampler, in.uv).r;
    let t = smoothstep(beach_start, beach_end, sdf_value);
    let final_color = mix(sand_color.rgb, grass_color.rgb, t);
    
    return vec4<f32>(final_color, 1.0);
}