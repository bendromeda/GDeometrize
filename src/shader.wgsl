// Vertex shader

struct VertexInput {
    [[location(0)]] position: vec2<i32>;
    [[location(1)]] tex_coords: vec2<f32>;
    // [[location(2)]] tint: vec4<f32>;
    [[location(2)]] tint_index: i32;
};

struct VertexOutput {
    [[builtin(position)]] clip_position: vec4<f32>;
    [[location(0)]] tex_coords: vec2<f32>;
    // [[location(1)]] tint: vec4<f32>;
    [[location(1)]] tint_index: i32;
};

struct Size {
    width: u32;
    height: u32;
};
struct Tint {
    tint: array<atomic<vec4<f32>>>;
};

[[group(1), binding(0)]] // 1.
var<uniform> size: Size;

[[group(2), binding(0)]]
var<storage, read_write> tint: Tint;

[[stage(vertex)]]
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    out.clip_position = vec4<f32>(
        (f32(model.position.x) / f32(size.width)) * 2.0 - 1.0, 
        -((f32(model.position.y) / f32(size.height)) * 2.0 - 1.0),
        0.0, 1.0
    );
    out.tint = model.tint;
    return out;
}

[[group(0), binding(0)]]
var t_target: texture_2d<f32>;
[[group(0), binding(1)]]
var s_target: sampler;
[[group(1), binding(0)]]
var t_diffuse: texture_2d<f32>;
[[group(1), binding(1)]]
var s_diffuse: sampler;
// [[group(2), binding(0)]] // 1.
// var<uniform> tint: Tint;

[[stage(fragment)]]
fn fs_find_avg_color(in: VertexOutput) {
    let color = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    // ...
}

[[group(0), binding(0)]]
var t_diffuse: texture_2d<f32>;
[[group(0), binding(1)]]
var s_diffuse: sampler;
// [[group(2), binding(0)]] // 1.
// var<uniform> tint: Tint;

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    let color = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    return color * in.tint;
}




struct Diff {
    diff: array<atomic<u32>>;
};

// fn color_diff(p1: vec3<f32>, p2: vec3<f32>) -> f32 {
//     let dr = (p1.r - p2.r) * 255.0;
//     let dg = (p1.g - p2.g) * 255.0;
//     let db = (p1.b - p2.b) * 255.0;
//     let rdash = (p1.r * 255.0 - p2.r * 255.0) * 0.5;
//     return ((2.0 + rdash / 256.0) * dr * dr + 4.0 * dg * dg + (2.0 + (255.0 - rdash) / 256.0) * db * db);
// }

[[group(0), binding(0)]]
var t_target: texture_2d<f32>;
[[group(0), binding(1)]]
var s_target: sampler;

[[group(1), binding(0)]]
var t_current: texture_2d_array<f32>;
[[group(1), binding(1)]]
var s_current: sampler;

[[group(2), binding(0)]]
var<uniform> size2: Size;

[[group(3), binding(0)]]
var<storage, read_write> total: Diff;

[[stage(compute), workgroup_size(1)]]
fn cmp_main([[builtin(global_invocation_id)]] global_id: vec3<u32>) {
    //if (global_id.x == 0u && global_id.y == 0u) {
    let coord = vec2<i32>(
        i32(global_id.x) - i32(size2.width / 2u), 
        i32(global_id.y) - i32(size2.height / 2u),
    );

    let uv = vec2<f32>(
        f32(coord.x) / f32(size2.width), 
        f32(coord.y) / f32(size2.height),
    );

    let z = i32(global_id.z);

    let target_color: vec4<f32> = textureSampleLevel(t_target, s_target, uv, 0.0);
    let current_color: vec4<f32> = textureSampleLevel(t_current, s_current, uv, z, 0.0);

    // let target_color: vec4<f32> = textureLoad(t_target, coord, 0);
    // let current_color: vec4<f32> = textureLoad(t_current, coord, 0);
    let sub: vec3<f32> = current_color.rgb - target_color.rgb;
    let diff: f32 = sub.r * sub.r + sub.g * sub.g + sub.b * sub.b;

    atomicAdd(&total.diff[z], u32(1000.0 * diff));
    //atomicAdd(&total.diff, 69u);
    //}
}
 

