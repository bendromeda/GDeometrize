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

use rayon::prelude::*;

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

use image::GenericImageView;
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

    fn get_avg_color(
        &self,
        state: &State,
        positions: [[f32; 2]; 4],
        tex_coords: [[f32; 2]; 4],
        target: &DynamicImage,
        spritesheet: &RgbaImage,
    ) -> [f32; 3] {
        let frame = state.packer.get(&OBJ_IDS[self.img_index]).unwrap();
        let obj_width = frame.frame.w;
        let obj_height = frame.frame.h;

        // let mut average = [0.0, 0.0, 0.0];
        // let mut count = 0.0;
        let scale = self.scale;

        let x_samples = if scale < 1.0 {
            (obj_width as f32 * scale) as u32
        } else {
            obj_width
        };

        let y_samples = if scale < 1.0 {
            (obj_height as f32 * scale) as u32
        } else {
            obj_height
        };

        pub(crate) const CHUNK_SIZE: usize = 256;

        let avg_color = (0..(x_samples * y_samples))
            .into_par_iter()
            .chunks(CHUNK_SIZE)
            .map(|chunk| {
                let mut sum = ([0.0, 0.0, 0.0], 0.0);
                for i in chunk {
                    let x = i % x_samples;
                    let y = i / x_samples;

                    let x01 = x as f32 / x_samples as f32;
                    let y01 = y as f32 / y_samples as f32;

                    let target_pos = [
                        positions[0][0]
                            + (positions[1][0] - positions[0][0]) * x01
                            + (positions[3][0] - positions[0][0]) * y01,
                        positions[0][1]
                            + (positions[1][1] - positions[0][1]) * x01
                            + (positions[3][1] - positions[0][1]) * y01,
                    ];

                    let texture_pos = [
                        tex_coords[0][0]
                            + (tex_coords[1][0] - tex_coords[0][0]) * x01
                            + (tex_coords[3][0] - tex_coords[0][0]) * y01,
                        tex_coords[0][1]
                            + (tex_coords[1][1] - tex_coords[0][1]) * x01
                            + (tex_coords[3][1] - tex_coords[0][1]) * y01,
                    ];
                    // continue if out of bounds
                    if target_pos[0] < 0.0
                        || (target_pos[0] as u32) >= target.width()
                        || target_pos[1] < 0.0
                        || (target_pos[1] as u32) >= target.height()
                    {
                        continue;
                    }

                    let target_pixel = target.get_pixel(target_pos[0] as u32, target_pos[1] as u32);
                    let texture_pixel = *spritesheet.get_pixel(
                        (texture_pos[0] * state.sheet_size[0] as f32) as u32,
                        (texture_pos[1] * state.sheet_size[1] as f32) as u32,
                    );

                    //dbg!(target_pixel, texture_pixel);
                    // understandable
                    // this part just gets the average color of the target image in the area of the shape
                    // its not on the gpu because idk how and also its not that much since tthe images are small
                    // so i do it with cpu threads
                    let alpha = texture_pixel.0[3] as f32 / 255.0;
                    if alpha == 0.0 {
                        continue;
                    }

                    sum.1 += alpha;
                    // texture_pixel * x = target_pixel
                    // x = target_pixel / texture_pixel
                    if texture_pixel.0[0] > 0 {
                        sum.0[0] += alpha
                            * ((target_pixel.0[0] as f32 / 255.0)
                                / (texture_pixel.0[0] as f32 / 255.0));
                    }
                    if texture_pixel.0[1] > 0 {
                        sum.0[1] += alpha
                            * ((target_pixel.0[1] as f32 / 255.0)
                                / (texture_pixel.0[1] as f32 / 255.0));
                    }
                    if texture_pixel.0[2] > 0 {
                        sum.0[2] += alpha
                            * ((target_pixel.0[2] as f32 / 255.0)
                                / (texture_pixel.0[2] as f32 / 255.0));
                    }

                    // sum.0[0] += alpha * (target_pixel.0[0] as f32 / 255.0);

                    // sum.0[1] += alpha * (target_pixel.0[1] as f32 / 255.0);

                    // sum.0[2] += alpha * (target_pixel.0[2] as f32 / 255.0);
                }
                //dbg!(sum);
                sum
            })
            .reduce(
                || ([0.0, 0.0, 0.0], 0.0),
                |(sum, sc), (next, c)| {
                    (
                        [sum[0] + next[0], sum[1] + next[1], sum[2] + next[2]],
                        sc + c,
                    )
                },
            );

        //dbg!(avg_color);

        if avg_color.1 > 0.0 {
            avg_color.0.map(|a| a / avg_color.1)
        } else {
            [0.0, 0.0, 0.0]
        }
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

    pub(crate) fn to_obj_string(&self, r: f32, g: f32, b: f32, layer: usize) -> String {
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

pub(crate) fn rotate_point(x: f32, y: f32, angle: f32) -> (f32, f32) {
    let cos = angle.cos();
    let sin = angle.sin();
    (x * cos - y * sin, x * sin + y * cos)
}

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
    let (mut h, mut s, v) = (max, max, max);

    let d = max - min;
    s = if max == 0.0 { 0.0 } else { d / max };

    if max == min {
        h = 0.0; // achromatic
    } else {
        h = if max == r {
            (g - b) / d + (if g < b { 6.0 } else { 0.0 })
        } else if max == g {
            (b - r) / d + 2.0
        } else {
            (r - g) / d + 4.0
        };

        h /= 6.0;
    }
    if h.is_nan() || s.is_nan() || v.is_nan() {
        panic!("NaN did done with R: {}, G: {}, B: {}", r, g, b)
    }
    ((h * 360.0) as i32, s, v)
}
