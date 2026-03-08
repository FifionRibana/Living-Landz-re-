#import bevy_sprite::mesh2d_vertex_output::VertexOutput

@group(2) @binding(0) var border_sdf_texture: texture_2d<f32>;
@group(2) @binding(1) var border_sdf_sampler: sampler;

struct BorderParams {
    line_width: f32,
    edge_softness: f32,
    glow_intensity: f32,
    color: vec4<f32>,
}

@group(2) @binding(2) var<uniform> params: BorderParams;
@group(2) @binding(3) var<uniform> time: f32;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // Flip UV Y coordinate to match SDF generation coordinate system
    let flipped_uv = vec2<f32>(in.uv.x, 1.0 - in.uv.y);

    // Sample SDF - value is 0-1 where 0 = on border, 1 = far from border
    let sdf_value = textureSample(border_sdf_texture, border_sdf_sampler, flipped_uv).r;

    // Convert back to pixel distance (0-50px range)
    let distance = sdf_value * 50.0;

    // Solid border: 2-3 pixels fully opaque
    let solid_border_width = 2.5;
    let solid_alpha = 1.0 - smoothstep(0.0, 1.0, distance - solid_border_width);

    // Fade toward interior: 15-30 pixels gradient to transparent
    let fade_start = solid_border_width;
    let fade_end = 25.0; // 15-30 pixel fade range
    let fade_alpha = 1.0 - smoothstep(fade_start, fade_end, distance);

    // Combine: solid border OR faded region (whichever is stronger)
    let total_alpha = max(solid_alpha, fade_alpha);

    // Discard fully transparent pixels
    if (total_alpha < 0.01) {
        discard;
    }

    // Final color with organization color and calculated alpha
    return vec4<f32>(params.color.rgb, total_alpha * params.color.a);
}
