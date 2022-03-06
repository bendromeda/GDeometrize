// Vertex shader

struct VertexInput {
    [[location(0)]] position: vec2<i32>;
    [[location(1)]] tex_coords: vec2<f32>;
};

struct VertexOutput {
    [[builtin(position)]] clip_position: vec4<f32>;
    [[location(0)]] tex_coords: vec2<f32>;
};

struct Size {
    width: u32;
    height: u32;
};
[[group(1), binding(0)]] // 1.
var<uniform> size: Size;

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
    return out;
}

// Fragment shader

struct Tint {
    tint: vec4<f32>;
};

[[group(0), binding(0)]]
var t_diffuse: texture_2d<f32>;
[[group(0), binding(1)]]
var s_diffuse: sampler;
[[group(2), binding(0)]] // 1.
var<uniform> tint: Tint;

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    let color = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    return color * tint.tint;
}
 
 