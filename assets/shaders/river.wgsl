// LL-E stage B: flowing-river water shader.
//
// Samples a client-rasterized river coverage texture (R = channel coverage, 1 in
// the core → 0 at the banks; width by Strahler order). Matches the painterly lake
// palette/detail (shallow→deep gradient, colour variation, caustics) and adds
// river-specific motion: ripples scroll ALONG the channel (flow axis = the
// perpendicular of the coverage gradient). The bank edge is broken up with fbm so
// it reads organic instead of showing the rasterized cells; alpha fades to 0 at
// the banks so the river blends onto the terrain.

#import bevy_sprite::mesh2d_vertex_output::VertexOutput
#import bevy_render::globals::Globals

@group(0) @binding(1) var<uniform> globals: Globals;

struct RiverParams {
    world_width: f32,
    world_height: f32,
    flow_speed: f32,
    _padding: f32,
}

@group(2) @binding(0) var coverage_tex: texture_2d<f32>;
@group(2) @binding(1) var coverage_sampler: sampler;
@group(2) @binding(2) var<uniform> params: RiverParams;
@group(2) @binding(3) var<uniform> shallow_color: vec4<f32>;
@group(2) @binding(4) var<uniform> deep_color: vec4<f32>;
@group(2) @binding(5) var<uniform> foam_color: vec4<f32>;

const TAU: f32 = 6.28318530718;

fn hash(p: vec2<f32>) -> f32 {
    return fract(sin(dot(p, vec2<f32>(127.1, 311.7))) * 43758.5453123);
}

fn noise(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let a = hash(i);
    let b = hash(i + vec2<f32>(1.0, 0.0));
    let c = hash(i + vec2<f32>(0.0, 1.0));
    let d = hash(i + vec2<f32>(1.0, 1.0));
    let u = f * f * (3.0 - 2.0 * f);
    return mix(mix(a, b, u.x), mix(c, d, u.x), u.y);
}

fn fbm(p: vec2<f32>, octaves: i32) -> f32 {
    var value = 0.0;
    var amp = 0.5;
    var pos = p;
    let rot = mat2x2<f32>(0.8, 0.6, -0.6, 0.8);
    for (var i = 0; i < octaves; i = i + 1) {
        value = value + amp * noise(pos);
        pos = rot * pos * 2.0;
        amp = amp * 0.5;
    }
    return value;
}

// 9-tap blur of the coverage (anti-pixellisation, same idea as lake/ocean).
fn coverage_blurred(uv: vec2<f32>) -> f32 {
    let o = 1.0 / vec2<f32>(textureDimensions(coverage_tex));
    var s = 0.0;
    s = s + textureSample(coverage_tex, coverage_sampler, uv + vec2<f32>(-o.x, -o.y)).r;
    s = s + textureSample(coverage_tex, coverage_sampler, uv + vec2<f32>(0.0, -o.y)).r;
    s = s + textureSample(coverage_tex, coverage_sampler, uv + vec2<f32>(o.x, -o.y)).r;
    s = s + textureSample(coverage_tex, coverage_sampler, uv + vec2<f32>(-o.x, 0.0)).r;
    s = s + textureSample(coverage_tex, coverage_sampler, uv).r;
    s = s + textureSample(coverage_tex, coverage_sampler, uv + vec2<f32>(o.x, 0.0)).r;
    s = s + textureSample(coverage_tex, coverage_sampler, uv + vec2<f32>(-o.x, o.y)).r;
    s = s + textureSample(coverage_tex, coverage_sampler, uv + vec2<f32>(0.0, o.y)).r;
    s = s + textureSample(coverage_tex, coverage_sampler, uv + vec2<f32>(o.x, o.y)).r;
    return s / 9.0;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv;
    let time = globals.time;
    let world_pos = uv * vec2<f32>(params.world_width, params.world_height);
    // Normalize noise frequencies to a 9600-unit reference (matches the lake).
    let ref_pos = world_pos * (9600.0 / params.world_width);

    // Break the straight-channel look with a low-frequency DOMAIN WARP. The
    // displacement is a pure function of ABSOLUTE world position (via ref_pos, which
    // is world_pos scaled) and is applied in WORLD units, then converted back to uv.
    // This keeps the meander identical regardless of how the world is chunked — the
    // noise never resets per chunk/quad — so there's no discontinuity at chunk
    // junctions. Amplitude ~1 cell so the water stays on the River/bank cells.
    let world_size = vec2<f32>(params.world_width, params.world_height);
    let warp_world = vec2<f32>(
        fbm(ref_pos * 0.010 + vec2<f32>(31.0, 17.0), 3) - 0.5,
        fbm(ref_pos * 0.010 + vec2<f32>(83.0, 61.0), 3) - 0.5,
    ) * 220.0; // ~±110 world units ≈ ±1 cell of meander
    let wuv = uv + warp_world / world_size;

    let cov = coverage_blurred(wuv);
    // Organic bank: warp the coverage edge with a finer fbm so it wobbles instead
    // of showing the rasterized cell grid, on top of the meandering channel.
    let shore_noise = (fbm(ref_pos * 0.06, 3) - 0.5) * 0.12;
    let cov_p = clamp(cov + shore_noise, 0.0, 1.0);
    if (cov_p < 0.04) {
        discard;
    }

    // Flow axis = perpendicular of the coverage gradient (runs along the channel).
    // Sample the gradient at the warped uv so flow tracks the meandering channel.
    let texel = 1.0 / vec2<f32>(textureDimensions(coverage_tex));
    let gx = textureSample(coverage_tex, coverage_sampler, wuv + vec2<f32>(texel.x, 0.0)).r
           - textureSample(coverage_tex, coverage_sampler, wuv - vec2<f32>(texel.x, 0.0)).r;
    let gy = textureSample(coverage_tex, coverage_sampler, wuv + vec2<f32>(0.0, texel.y)).r
           - textureSample(coverage_tex, coverage_sampler, wuv - vec2<f32>(0.0, texel.y)).r;
    let grad = vec2<f32>(gx, gy);
    var flow = vec2<f32>(1.0, 0.0);
    if (length(grad) > 0.0005) {
        flow = normalize(vec2<f32>(-grad.y, grad.x));
    }

    // Depth from coverage (channel core is deeper/darker), like the lake.
    let depth = pow(saturate(cov_p), 1.1);
    var color = mix(shallow_color.rgb, deep_color.rgb, depth);

    // Subtle large-scale colour variation (matches the lake).
    let variation = fbm(ref_pos * 0.0003, 4);
    color = color + vec3<f32>(
        (variation - 0.5) * 0.02,
        (variation - 0.5) * 0.01,
        -(variation - 0.5) * 0.015,
    );

    // River-specific: ripples scrolling ALONG the channel.
    let ft = time * params.flow_speed;
    let f1 = fbm(ref_pos * 0.30 + flow * ft * 1.1, 4);
    let f2 = fbm(ref_pos * 0.60 - flow * ft * 0.7, 3);
    let flow_detail = f1 * 0.6 + f2 * 0.4;
    color = color + vec3<f32>((flow_detail - 0.5) * 0.06) * smoothstep(0.1, 0.6, depth);

    // Caustics near the banks (matches the lake).
    let caustic_zone = 1.0 - smoothstep(0.0, 0.4, depth);
    if (caustic_zone > 0.01) {
        let ct = time * 0.3;
        let c_uv = ref_pos * 0.35;
        let c1 = pow(fbm(c_uv + vec2<f32>(ct * 0.08, -ct * 0.06), 4), 2.0);
        let c2 = pow(fbm(c_uv * 1.3 + vec2<f32>(-ct * 0.07, ct * 0.09), 4), 2.0);
        let caustics = (c1 + c2) * 0.5;
        let caustic_tint = mix(
            vec3<f32>(0.2, 0.4, 0.5),
            vec3<f32>(0.5, 0.45, 0.3),
            1.0 - smoothstep(0.0, 0.12, depth),
        );
        color = color + caustic_tint * caustics * caustic_zone * 0.15;
    }

    // Foam: organic band near the banks + broken on the flow crests.
    let bank = (1.0 - smoothstep(0.0, 0.55, cov_p)) * smoothstep(0.04, 0.28, cov_p);
    let crest = smoothstep(0.62, 0.82, flow_detail) * smoothstep(0.1, 0.5, depth);
    let foam = clamp(bank * 0.85 + crest * 0.25, 0.0, 1.0);
    color = mix(color, foam_color.rgb, foam * 0.5);

    // Alpha fades organically at the banks (cov_p already carries the fbm warp).
    // The window accommodates the coverage blur (which lowers thin-channel peaks)
    // so smoothed reaches stay solid without a wide faint fringe.
    let alpha = smoothstep(0.10, 0.30, cov_p);
    return vec4<f32>(color, alpha);
}
