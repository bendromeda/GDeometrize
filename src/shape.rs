use crate::Vertex;

use crate::OPACITY;

use crate::TintUniform;

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
}

pub const OBJ_IDS: &[u16] = &[
    211, 259, 266, 273, 280, 693, 695, 697, 699, 701, 725, 1011, 1012, 1013, 1102, 1106, 1111,
    1112, 1113, 1114, 1115, 1116, 1117, 1118, 1348, 1351, 1352, 1353, 1354, 1355, 1442, 1443, 1461,
    1462, 1463, 1464, 1596, 1597, 1608, 1609, 1610, 1753, 1754, 1757, 1764, 1765, 1766, 1767, 1768,
    1769, 1837, 1835, 1869, 1870, 1871, 1874, 1875, 1886, 1887, 1888,
];

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
    pub(crate) fn paste(
        &self,
        state: &State,
        target: &DynamicImage,
        spritesheet: &RgbaImage,
    ) -> Result<(), wgpu::SurfaceError> {
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
            [x as f32, y as f32]
        });

        let obj_width = frame.frame.w;
        let obj_height = frame.frame.h;

        // let mut average = [0.0, 0.0, 0.0];
        // let mut count = 0.0;

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

        // for x in 0..x_samples {
        //     for y in 0..y_samples {
        //         let x01 = x as f32 / x_samples as f32;
        //         let y01 = y as f32 / y_samples as f32;

        //         let target_pos = [
        //             positions[0][0]
        //                 + (positions[1][0] - positions[0][0]) * x01
        //                 + (positions[3][0] - positions[0][0]) * y01,
        //             positions[0][1]
        //                 + (positions[1][1] - positions[0][1]) * x01
        //                 + (positions[3][1] - positions[0][1]) * y01,
        //         ];

        //         let texture_pos = [
        //             tex_coords[0][0]
        //                 + (tex_coords[1][0] - tex_coords[0][0]) * x01
        //                 + (tex_coords[3][0] - tex_coords[0][0]) * y01,
        //             tex_coords[0][1]
        //                 + (tex_coords[1][1] - tex_coords[0][1]) * x01
        //                 + (tex_coords[3][1] - tex_coords[0][1]) * y01,
        //         ];
        //         // continue if out of bounds
        //         if target_pos[0] < 0.0
        //             || (target_pos[0] as u32) >= target.width()
        //             || target_pos[1] < 0.0
        //             || (target_pos[1] as u32) >= target.height()
        //         {
        //             continue;
        //         }

        //         let target_pixel = target.get_pixel(target_pos[0] as u32, target_pos[1] as u32);
        //         let texture_pixel = *spritesheet.get_pixel(
        //             (texture_pos[0] * state.sheet_size[0] as f32) as u32,
        //             (texture_pos[1] * state.sheet_size[1] as f32) as u32,
        //         );
        //         let alpha = texture_pixel.0[3] as f32 / 255.0;
        //         count += alpha;
        //         // texture_pixel * x = target_pixel
        //         // x = target_pixel / texture_pixel
        //         average[0] += alpha
        //             * ((target_pixel.0[0] as f32 / 255.0) / (texture_pixel.0[0] as f32 / 255.0));
        //         average[1] += alpha
        //             * ((target_pixel.0[1] as f32 / 255.0) / (texture_pixel.0[1] as f32 / 255.0));
        //         average[2] += alpha
        //             * ((target_pixel.0[2] as f32 / 255.0) / (texture_pixel.0[2] as f32 / 255.0));
        //     }
        // }

        pub(crate) const CHUNK_SIZE: usize = 256;

        pub(crate) fn lin(c: f32) -> f32 {
            if c > 0.04045 {
                ((c + 0.055) / 1.055).powf(2.4)
            } else {
                c / 12.92
            }
        }

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
                    let alpha = texture_pixel.0[3] as f32 / 255.0;
                    if alpha == 0.0 {
                        continue;
                    }
                    sum.1 += alpha;
                    // texture_pixel * x = target_pixel
                    // x = target_pixel / texture_pixel
                    if texture_pixel.0[0] > 0 {
                        sum.0[0] += alpha
                            * lin((target_pixel.0[0] as f32 / 255.0)
                                / (texture_pixel.0[0] as f32 / 255.0));
                    }
                    if texture_pixel.0[1] > 0 {
                        sum.0[1] += alpha
                            * lin((target_pixel.0[1] as f32 / 255.0)
                                / (texture_pixel.0[1] as f32 / 255.0));
                    }
                    if texture_pixel.0[2] > 0 {
                        sum.0[2] += alpha
                            * lin((target_pixel.0[2] as f32 / 255.0)
                                / (texture_pixel.0[2] as f32 / 255.0));
                    }

                    // sum.0[0] += alpha * (target_pixel.0[0] as f32 / 255.0);

                    // sum.0[1] += alpha * (target_pixel.0[1] as f32 / 255.0);

                    // sum.0[2] += alpha * (target_pixel.0[2] as f32 / 255.0);
                }
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

        let average = avg_color.0.map(|a| a / avg_color.1);

        state.queue.write_buffer(
            &state.tint_buffer,
            0,
            bytemuck::cast_slice(&[TintUniform {
                tint: [average[0], average[1], average[2], OPACITY],
            }]),
        );

        let verteces = positions
            .iter()
            .zip(tex_coords.iter())
            .map(|(p, t)| Vertex {
                position: [p[0] as i32, p[1] as i32],
                tex_coords: [t[0], t[1]],
            })
            .collect::<Vec<_>>();

        let quad = &[
            verteces[3],
            verteces[0],
            verteces[1],
            verteces[3],
            verteces[1],
            verteces[2],
        ];

        let vertex_buffer = state
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(quad),
                usage: wgpu::BufferUsages::VERTEX,
            });

        let mut encoder = state
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view: &state.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });
            render_pass.set_pipeline(&state.render_pipeline);

            render_pass.set_bind_group(0, &state.sheet_bind_group, &[]);
            render_pass.set_bind_group(1, &state.size_bind_group, &[]);
            render_pass.set_bind_group(2, &state.tint_bind_group, &[]);

            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.draw(0..quad.len() as u32, 0..1);
        }

        // submit will accept anything that implements IntoIter
        state.queue.submit(std::iter::once(encoder.finish()));

        Ok(())
    }

    pub(crate) fn new_random(width: u32, height: u32, img_index: usize) -> Shape {
        let mut rng = rand::thread_rng();
        let x = rng.gen_range(0..width) as i32;
        let y = rng.gen_range(0..height) as i32;
        let scale = rng.gen_range(0.2..3.0);
        let rot = rng.gen_range(0.0..(2.0 * std::f32::consts::PI));

        Shape {
            img_index,
            x,
            y,
            scale,
            rot,
        }
    }

    pub(crate) fn adjust_random(&mut self) {
        self.x = self.x as i32 + rand::thread_rng().gen_range(-3i32..=3);
        self.y = self.y as i32 + rand::thread_rng().gen_range(-3i32..=3);
        self.scale *= rand::thread_rng().gen_range(0.9..1.1);
        self.rot += rand::thread_rng().gen_range(-0.1..0.1);
    }
}

pub(crate) fn rotate_point(x: f32, y: f32, angle: f32) -> (f32, f32) {
    let cos = angle.cos();
    let sin = angle.sin();
    (x * cos - y * sin, x * sin + y * cos)
}
