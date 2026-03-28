#import bevy_sprite::mesh2d_vertex_output::VertexOutput
#import bevy_render::globals::Globals

@group(0) @binding(1) var<uniform> globals: Globals;

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> material_color: vec4<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var<uniform> material_time: f32;

@fragment
fn fragment(
    mesh: VertexOutput
) -> @location(0) vec4<f32> {
    var color = material_color;
    let pulse = 0.2 + 0.1 * sin(globals.time * 3.14 / 1.);
    color.a = pulse;
    return color;
}