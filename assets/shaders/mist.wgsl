#import bevy_sprite::mesh2d_vertex_output::VertexOutput

@group(2) @binding(0) var mist_texture: texture_2d<f32>;
@group(2) @binding(1) var mist_sampler: sampler;

@group(2) @binding(2) var<uniform> mist_params: MistParams;

struct MistParams {
    world_width: f32,
    world_height: f32,
    _padding1: f32,
    _padding2: f32,
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // Unexplored = dark, explored = fully transparent
    let explored_raw = textureSample(mist_texture, mist_sampler, in.uv).r;
    
    // Shift transition entirely into explored zone:
    // explored_raw < 0.5 → fully opaque (unexplored side)
    // explored_raw 0.5 → 1.0 → transition to transparent (explored side)
    let explored = smoothstep(0.7, 0.98, explored_raw);
    let mist_alpha = 1.0 - explored;

    if mist_alpha < 0.01 {
        discard;
    }

    let mist_color = vec3<f32>(0.08, 0.08, 0.12);
    return vec4<f32>(mist_color, mist_alpha);
}