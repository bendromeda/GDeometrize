// Vertex shader

struct VertexInput {
    [[location(0)]] position: vec2<i32>;
    [[location(1)]] tex_coords: vec2<f32>;
    // [[location(2)]] tint: vec4<f32>;
    [[location(2)]] tint_index: i32;
    [[location(3)]] target_coords: vec2<f32>;
};

struct VertexOutput {
    [[builtin(position)]] clip_position: vec4<f32>;
    [[location(0)]] tex_coords: vec2<f32>;
    // [[location(1)]] tint: vec4<f32>;
    [[location(1)]] tint_index: i32;
    [[location(2)]] target_coords: vec2<f32>;
};

let total_shapes = 800;

let factor = 1000.0;
struct Tint {
    tint: array<array<atomic<u32>, 3>, total_shapes>;
    counts: array<atomic<u32>, total_shapes>;
    opacity: f32;

    diff: array<atomic<i32>, total_shapes>;
};


[[group(1), binding(0)]]
var t_target: texture_2d<f32>;
[[group(1), binding(1)]]
var s_target: sampler;


[[group(2), binding(0)]]
var<storage, read_write> tint: Tint;

[[group(3), binding(0)]]
var t_current: texture_2d<f32>;
[[group(3), binding(1)]]
var s_current: sampler;


[[stage(vertex)]]
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    let size = textureDimensions(t_target);
    out.clip_position = vec4<f32>(
        (f32(model.position.x) / f32(size.x)) * 2.0 - 1.0, 
        -((f32(model.position.y) / f32(size.y)) * 2.0 - 1.0),
        0.0, 1.0
    );
    out.tint_index = model.tint_index;
    out.target_coords = model.target_coords;
    return out;
}

[[group(0), binding(0)]]
var t_diffuse: texture_2d<f32>;
[[group(0), binding(1)]]
var s_diffuse: sampler;

[[stage(fragment)]]
fn fs_find_avg_color(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    let tex = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    let target = textureSample(t_target, s_target, in.target_coords);

    let a = tex.a;
    atomicAdd(&tint.counts[in.tint_index], u32(a * factor));
    atomicAdd(&tint.tint[in.tint_index][0], u32(a * (target.r / tex.r) * factor));
    atomicAdd(&tint.tint[in.tint_index][1], u32(a * (target.g / tex.g) * factor));
    atomicAdd(&tint.tint[in.tint_index][2], u32(a * (target.b / tex.b) * factor));
    return vec4<f32>(0.0);
}

fn color_diff(p1: vec3<f32>, p2: vec3<f32>) -> f32 {
    let sub: vec3<f32> = p1 - p2;
    return sqrt(sub.r * sub.r + sub.g * sub.g + sub.b * sub.b);
}

[[stage(fragment)]]
fn fs_find_diff(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    let c = f32(tint.counts[in.tint_index]) / factor;
    
    var t = vec4<f32>(
        (f32(tint.tint[in.tint_index][0]) / factor) / c,
        (f32(tint.tint[in.tint_index][1]) / factor) / c,
        (f32(tint.tint[in.tint_index][2]) / factor) / c,
        tint.opacity,
    );

    let color = textureSample(t_diffuse, s_diffuse, in.tex_coords) * t;

    let target = textureSample(t_target, s_target, in.target_coords).rgb;
    let current = textureSample(t_current, s_current, in.target_coords).rgb;

    let next = t.rgb + current * (1.0 - t.a);

    let diff = color_diff(target, next) - color_diff(target, current);
    
    atomicAdd(&tint.diff[in.tint_index], i32(64.0 * diff));

    return vec4<f32>(0.0);
}

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    let color = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    
    let c = f32(tint.counts[in.tint_index]);
    
    var tint = vec4<f32>(
        f32(tint.tint[in.tint_index][0]) / c,
        f32(tint.tint[in.tint_index][1]) / c,
        f32(tint.tint[in.tint_index][2]) / c,
        tint.opacity,
    );

    return color * tint;
}


