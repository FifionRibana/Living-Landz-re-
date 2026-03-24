#import bevy_sprite::mesh2d_vertex_output::VertexOutput

@group(2) @binding(0) var base_texture: texture_2d<f32>;
@group(2) @binding(1) var base_sampler: sampler;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let color = textureSample(base_texture, base_sampler, in.uv);

    // Alpha test — discard instead of blend
    if color.a < 0.5 {
        discard;
    }

    return color;
}