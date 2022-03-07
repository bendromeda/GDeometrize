use image::{DynamicImage, RgbaImage};
use std::fs;
use wgpu::Origin3d;

use crate::{image_diff, shape::*, State};

pub const OPACITY: f32 = 0.5;

const SHAPES_PER_OBJ: usize = 4;
const ITERATIONS: usize = 400;
// const SHAPES_ADJUSTED: usize = 10;
// const ADJUSTMENTS: usize = 100;

pub const TOTAL_SHAPES: usize = SHAPES_PER_OBJ * OBJ_IDS.len();

pub const OBJ_IDS: &[u16] = &[
    211, 259, 266, 273, 280, 693, 695, 697, 699, 701, 725, 1011, 1012, 1013, 1102, 1106, 1111,
    1112, 1113, 1114, 1115, 1116, 1117, 1118, 1348, 1351, 1352, 1353, 1354, 1355, 1442, 1443, 1461,
    1462, 1463, 1464, 1596, 1597, 1608, 1609, 1610, 1753, 1754, 1757, 1764, 1765, 1766, 1767, 1768,
    1769, 1837, 1835, 1869, 1870, 1871, 1874, 1875, 1886, 1887, 1888,
];

pub const TARGET: &str = "planet.jpeg";

pub fn process(state: &State, target: &DynamicImage, spritesheet: &RgbaImage) {
    let mut level_string = "1,899,2,-29,3,975,36,1,7,255,8,0,9,0,10,0,35,1,23,1;1,899,2,-29,3,1005,36,1,7,0,8,0,9,0,10,0,35,1,23,1000;".to_string();

    let mut current_diff = std::u32::MAX;
    for _ in 0..ITERATIONS {
        let mut shapes = Vec::new();

        for img_index in 0..OBJ_IDS.len() {
            for _ in 0..SHAPES_PER_OBJ {
                let shape = Shape::new_random(target.width(), target.height(), img_index);
                shapes.push(shape);
            }
        }
        //dbg!(&shapes);
        let diff = test_diff(state, &shapes, target, spritesheet);
        //dbg!(&diff);
        // get index of min diff
        let mut min_diff_index = 0;
        for i in 0..shapes.len() {
            if diff[i] < diff[min_diff_index] {
                min_diff_index = i;
            }
        }
        if diff[min_diff_index] < current_diff {
            current_diff = diff[min_diff_index];
        } else {
            continue;
        }
        dbg!(diff[min_diff_index]);
        let mut encoder = state
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
        encoder.copy_texture_to_texture(
            wgpu::ImageCopyTextureBase {
                texture: &state.temp_texture.texture,
                mip_level: 0,
                origin: Origin3d {
                    x: 0,
                    y: 0,
                    z: min_diff_index as u32,
                },
                aspect: wgpu::TextureAspect::All,
            },
            state.output_texture.texture.as_image_copy(),
            wgpu::Extent3d {
                width: state.size_uniform.width as u32,
                height: state.size_uniform.height as u32,
                depth_or_array_layers: 1,
            },
        );

        state.queue.submit(std::iter::once(encoder.finish()));
    }

    // for _ in 0..ITERATIONS {
    //     let mut min_shape = Shape::new_random(target.width(), target.height(), 0);
    //     let mut min_diff = std::f32::MAX;

    //     for img_index in 0..OBJ_IDS.len() {
    //         for _ in 0..SHAPES_PER_OBJ {
    //             let shape = Shape::new_random(target.width(), target.height(), img_index);
    //             let diff = test_diff(state, shape, target, spritesheet);

    //             if diff < min_diff {
    //                 min_diff = diff;
    //                 min_shape = shape;
    //             }
    //         }
    //     }

    //     if min_diff < current_diff {
    //         current_diff = min_diff;
    //     } else {
    //         continue;
    //     }

    //     dbg!(min_diff);

    //     let mut encoder = state
    //         .device
    //         .create_command_encoder(&wgpu::CommandEncoderDescriptor {
    //             label: Some("Render Encoder"),
    //         });

    //     level_string += &min_shape
    //         .paste(
    //             state,
    //             target,
    //             spritesheet,
    //             &mut encoder,
    //             &state.output_texture.view,
    //         )
    //         .unwrap();

    //     state.queue.submit(std::iter::once(encoder.finish()));
    // }

    // let shape = Shape {
    //     img_index: 0,
    //     x: 100,
    //     y: 20,
    //     scale: 0.0,
    //     rot: 0.0,
    // };
    // dbg!(test_diff(state, shape, target, spritesheet));

    // let shape2 = Shape {
    //     img_index: 0,
    //     x: 64,
    //     y: 32,
    //     scale: 10.0,
    //     rot: 0.0,
    // };

    // dbg!(test_diff(state, shape2, target, spritesheet));
    /*
    169.58418
    167.28096

    169.58418
    167.28096
    */

    // dbg!(test_diff(state, shape2, target, spritesheet));

    // {
    //     let mut encoder = state
    //         .device
    //         .create_command_encoder(&wgpu::CommandEncoderDescriptor {
    //             label: Some("Render Encoder"),
    //         });

    //     level_string += &shape2
    //         .paste(
    //             state,
    //             target,
    //             spritesheet,
    //             &mut encoder,
    //             &state.output_texture.view,
    //         )
    //         .unwrap();

    //     state.queue.submit(std::iter::once(encoder.finish()));
    // }

    // dbg!(test_diff(state, shape2, target, spritesheet));

    fs::write("./uhhh.txt", level_string).expect("Unable to write file");
}

pub fn test_diff(
    state: &State,
    shapes: &[Shape],
    target: &DynamicImage,
    spritesheet: &RgbaImage,
) -> Vec<u32> {
    let mut encoder = state
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

    let input_shapes = shapes
        .iter()
        .enumerate()
        .map(|(i, shape)| {
            encoder.copy_texture_to_texture(
                state.output_texture.texture.as_image_copy(),
                wgpu::ImageCopyTextureBase {
                    texture: &state.temp_texture.texture,
                    mip_level: 0,
                    origin: Origin3d {
                        x: 0,
                        y: 0,
                        z: i as u32,
                    },
                    aspect: wgpu::TextureAspect::All,
                },
                wgpu::Extent3d {
                    width: state.size_uniform.width as u32,
                    height: state.size_uniform.height as u32,
                    depth_or_array_layers: 1,
                },
            );

            let view = state
                .temp_texture
                .texture
                .create_view(&wgpu::TextureViewDescriptor {
                    dimension: Some(wgpu::TextureViewDimension::D2),
                    base_array_layer: i as u32,
                    array_layer_count: Some(std::num::NonZeroU32::new(1).unwrap()),
                    ..Default::default()
                });
            (*shape, view)
        })
        .collect::<Vec<_>>();

    Shape::paste(&input_shapes, state, target, spritesheet, &mut encoder);

    let view3d = state
        .temp_texture
        .texture
        .create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            ..Default::default()
        });

    image_diff::calc_image_diff(state, &mut encoder, &view3d);

    // encoder.copy_texture_to_texture(
    //     wgpu::ImageCopyTexture {
    //         aspect: wgpu::TextureAspect::All,
    //         texture: &state.temp_texture.texture,
    //         mip_level: 0,
    //         origin: wgpu::Origin3d::ZERO,
    //     },
    //     wgpu::ImageCopyTexture {
    //         aspect: wgpu::TextureAspect::All,
    //         texture: &state.output_texture.texture,
    //         mip_level: 0,
    //         origin: wgpu::Origin3d::ZERO,
    //     },
    //     wgpu::Extent3d {
    //         width: state.size_uniform.width as u32,
    //         height: state.size_uniform.height as u32,
    //         depth_or_array_layers: 1,
    //     },
    // );

    state.queue.submit(std::iter::once(encoder.finish()));

    pollster::block_on(image_diff::get_image_diff(state)).to_vec()
}
