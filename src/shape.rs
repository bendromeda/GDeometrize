use crate::Vertex;

use crate::OPACITY;

use crate::TintBuffer;

use crate::process::OBJ_IDS;
use crate::process::TOTAL_SHAPES;
use crate::State;

use image::DynamicImage;
use image::RgbaImage;
use rand::Rng;
use texture_packer::TexturePacker;

#[derive(Debug, Clone, Copy)]
pub struct Shape {
    pub(crate) img_index: usize,
    pub(crate) x: i32,
    pub(crate) y: i32,
    pub(crate) scale: f32,
    pub(crate) rot: f32,
    //pub(crate) tint: Option<[f32; 4]>,
}
// these are the obj ids were using

// and then it grabs all the images and packs them into a texture at runtime
// no i got the thing you sent in chat a few weeks ago
pub(crate) fn pack_textures<'a>() -> TexturePacker<'a, RgbaImage, u16> {
    let mut packer = TexturePacker::new_skyline(Default::default());

    for id in OBJ_IDS {
        let texture = image::open(&format!("objects/{}/main.png", id))
            .unwrap()
            .into_rgba8();
        packer.pack_own(*id, texture).unwrap();
    }

    packer
}

use wgpu::util::DeviceExt;

impl Shape {
    fn get_verts(&self, state: &State) -> ([[f32; 2]; 4], [[f32; 2]; 4]) {
        let frame = state.packer.get(&OBJ_IDS[self.img_index]).unwrap();
        // get texture coords
        let tex_coords = {
            let mut top_left = (frame.frame.x, frame.frame.y);
            let mut top_right = (frame.frame.x + frame.frame.w, frame.frame.y);
            let mut bottom_left = (frame.frame.x, frame.frame.y + frame.frame.h);
            let mut bottom_right = (frame.frame.x + frame.frame.w, frame.frame.y + frame.frame.h);
            if frame.rotated {
                // rotate -90 degrees
                let tmp = top_right;
                top_right = top_left;
                top_left = bottom_left;
                bottom_left = bottom_right;
                bottom_right = tmp;
            }
            let w = state.sheet_size[0] as f32;
            let h = state.sheet_size[1] as f32;
            [
                [top_left.0 as f32 / w as f32, top_left.1 as f32 / h as f32],
                [
                    bottom_left.0 as f32 / w as f32,
                    bottom_left.1 as f32 / h as f32,
                ],
                [
                    bottom_right.0 as f32 / w as f32,
                    bottom_right.1 as f32 / h as f32,
                ],
                [top_right.0 as f32 / w as f32, top_right.1 as f32 / h as f32],
            ]
        };

        // get positions
        let w = frame.frame.w as f32 / 2.0;
        let h = frame.frame.h as f32 / 2.0;
        let positions = [[-w, -h], [-w, h], [w, h], [w, -h]];

        // scale
        let scale = self.scale;
        let positions = positions.map(|p| [p[0] as f32 * scale, p[1] as f32 * scale]);

        // rotate
        let rot = self.rot;
        let positions = positions.map(|p| {
            let x = p[0] * rot.cos() - p[1] * rot.sin();
            let y = p[0] * rot.sin() + p[1] * rot.cos();
            [x, y]
        });

        // translate
        let positions = positions.map(|p| {
            let x = p[0] + self.x as f32;
            let y = p[1] + self.y as f32;
            [x, y]
        });

        (positions, tex_coords)
    }

    pub(crate) fn test_diff(
        shapes: &[Shape],
        state: &State,
        target: &DynamicImage,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        // println!("{:?}", avg_color);
        // if average[0].is_nan() {
        //     panic!(
        //         "Uh oh!!!! Average nan!!!!! {:?} {} {}",
        //         avg_color, x_samples, y_samples
        //     )
        // }

        state.queue.write_buffer(
            &state.tint_buffer,
            0,
            // here is the tint
            bytemuck::cast_slice(&[TintBuffer {
                tint: [[0, 0, 0]; TOTAL_SHAPES],
                counts: [0; TOTAL_SHAPES],
                opacity: OPACITY,
                diff: [0; TOTAL_SHAPES],
            }]),
        );
        let mut verteces = Vec::<Vertex>::new();

        for (i, shape) in shapes.iter().enumerate() {
            let (positions, tex_coords) = shape.get_verts(state);

            //let c = shape.get_avg_color(state, positions, tex_coords, target, spritesheet);
            //dbg!(c);

            let v = positions
                .iter()
                .zip(tex_coords.iter())
                .map(|(p, t)| Vertex {
                    position: [p[0] as i32, p[1] as i32],
                    tex_coords: [t[0], t[1]],
                    tint_index: i as i32,
                    target_coords: [p[0] / target.width() as f32, p[1] / target.height() as f32],
                })
                .collect::<Vec<_>>();

            verteces.extend([v[3], v[0], v[1], v[3], v[1], v[2]]);
        }

        let vertex_buffer = state
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&verteces),
                usage: wgpu::BufferUsages::VERTEX,
            });

        let mut render_pass = |label, pipeline| {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some(label),
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view: &state.dummy_texture_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });
            pass.set_pipeline(pipeline);
            pass.set_bind_group(0, &state.sheet_bind_group, &[]);
            pass.set_bind_group(1, &state.target_bind_group, &[]);
            pass.set_bind_group(2, &state.tint_bind_group, &[]);
            pass.set_bind_group(3, &state.output_texture_bind_group, &[]);

            pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            pass.draw(0..verteces.len() as u32, 0..1);
        };

        render_pass("avg color pass", &state.avg_color_pipeline);
        render_pass("diff pass", &state.diff_pipeline);
    }

    pub(crate) fn paste(
        &self,
        state: &State,
        target: &DynamicImage,
        //encoder: &mut wgpu::CommandEncoder,
        tint_index: usize,
    ) {
        let mut encoder = state
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
        let (positions, tex_coords) = self.get_verts(state);

        let v = positions
            .iter()
            .zip(tex_coords.iter())
            .map(|(p, t)| Vertex {
                position: [p[0] as i32, p[1] as i32],
                tex_coords: [t[0], t[1]],
                tint_index: tint_index as i32,
                target_coords: [p[0] / target.width() as f32, p[1] / target.height() as f32],
            })
            .collect::<Vec<_>>();

        let vertex_buffer = state
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&[v[3], v[0], v[1], v[3], v[1], v[2]]),
                usage: wgpu::BufferUsages::VERTEX,
            });

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("render pass"),
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view: &state.output_texture.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });
            pass.set_pipeline(&state.render_pipeline);
            pass.set_bind_group(0, &state.sheet_bind_group, &[]);
            pass.set_bind_group(1, &state.target_bind_group, &[]);
            pass.set_bind_group(2, &state.tint_bind_group, &[]);
            //pass.set_bind_group(3, &state.output_texture_bind_group, &[]);

            pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            pass.draw(0..6, 0..1);
        }

        state.queue.submit(std::iter::once(encoder.finish()));
    }

    pub(crate) fn new_random(width: u32, height: u32) -> Shape {
        let mut rng = rand::thread_rng();
        let x = rng.gen_range(0..width) as i32;
        let y = rng.gen_range(0..height) as i32;
        let scale = rng.gen_range(0.1..(std::cmp::max(width, height) as f32 / 40.0));
        let rot = rng.gen_range(0.0..(2.0 * std::f32::consts::PI));

        Shape {
            img_index: rng.gen_range(0..OBJ_IDS.len()) as usize,
            x,
            y,
            scale,
            rot,
        }
    }

    pub(crate) fn adjust_random(&mut self) {
        self.x += rand::thread_rng().gen_range(-3i32..=3);
        self.y += rand::thread_rng().gen_range(-3i32..=3);
        self.scale *= rand::thread_rng().gen_range(0.9..1.1);
        self.rot += rand::thread_rng().gen_range(-0.1..0.1);
    }

    pub(crate) fn to_obj_string(self, r: f32, g: f32, b: f32, layer: usize) -> String {
        let (h, s, v) = rgb_to_hsv(r, g, b);
        let hsv_string = format!("{}a{}a{}a0a0", h, s, v);
        let scale = 1.0;
        format!(
            "1,{},2,{},3,{},6,{},32,{},41,1,43,{hsv_string},21,1,22,2,25,{layer},24,-1;",
            OBJ_IDS[self.img_index],
            (self.x as f32) * 0.5 * scale,
            -(self.y as f32) * 0.5 * scale,
            self.rot * 180.0 / std::f32::consts::PI,
            self.scale * scale,
        )
    }
}

// pub(crate) fn rotate_point(x: f32, y: f32, angle: f32) -> (f32, f32) {
//     let cos = angle.cos();
//     let sin = angle.sin();
//     (x * cos - y * sin, x * sin + y * cos)
// }

pub(crate) fn max_float(a: f32, b: f32) -> f32 {
    if a > b {
        a
    } else {
        b
    }
}
pub(crate) fn min_float(a: f32, b: f32) -> f32 {
    if a < b {
        a
    } else {
        b
    }
}

pub(crate) fn to_srgb(v: f32) -> f32 {
    let sv = if v <= 0.0031308 {
        12.92 * v
    } else {
        1.055 * v.powf(1.0 / 2.4) - 0.055
    };
    if sv > 1.0 {
        1.0
    } else if sv < 0.0 {
        0.0
    } else {
        sv
    }
}

pub(crate) fn rgb_to_hsv(r: f32, g: f32, b: f32) -> (i32, f32, f32) {
    let max = max_float(r, max_float(g, b));
    let min = min_float(r, min_float(g, b));
    let v = max;

    let d = max - min;
    let s = if max == 0.0 { 0.0 } else { d / max };

    let h = if max == min {
        0.0 // achromatic
    } else {
        (if max == r {
            (g - b) / d + (if g < b { 6.0 } else { 0.0 })
        } else if max == g {
            (b - r) / d + 2.0
        } else {
            (r - g) / d + 4.0
        }) / 6.0
    };
    if h.is_nan() || s.is_nan() || v.is_nan() {
        panic!("NaN did done with R: {}, G: {}, B: {}", r, g, b)
    }
    ((h * 360.0) as i32, s, v)
}
