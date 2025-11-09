#import bevy_pbr::{
    mesh_view_bindings::globals,
}
#import bevy_sprite::mesh2d_vertex_output::VertexOutput

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> material_color: vec4<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var<uniform> material_time: f32;

fn hsv_to_rgb(h: f32, s: f32, v: f32) -> vec3<f32> {
    let c = v * s;
    let hp = h * 6.0;
    let x = c * (1.0 - abs((hp % 2.0) - 1.0));
    let p = v - c;
    
    var rgb = vec3<f32>(0.0);
    if (hp < 1.0) { rgb = vec3<f32>(c, x, 0.0); }
    else if (hp < 2.0) { rgb = vec3<f32>(x, c, 0.0); }
    else if (hp < 3.0) { rgb = vec3<f32>(0.0, c, x); }
    else if (hp < 4.0) { rgb = vec3<f32>(0.0, x, c); }
    else if (hp < 5.0) { rgb = vec3<f32>(x, 0.0, c); }
    else { rgb = vec3<f32>(c, 0.0, x); }
    
    return rgb + p;
}

@fragment
fn fragment(
    mesh: VertexOutput
) -> @location(0) vec4<f32> {
    var color = material_color;

    let hue = fract(material_time / 2.0); // 0 à 1 en 2 secondes
    // let hsv = vec3<f32>(hue, 0.8, 1.0); // Saturation élevée, luminosité max
    let rgb = hsv_to_rgb(hue, 0.8, 1.0); // Saturation élevée, luminosité max
    
    return vec4<f32>(rgb, 0.6);
}