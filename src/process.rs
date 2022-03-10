use image::DynamicImage;
use std::fs;

use crate::{shape::*, State, TintBuffer};

pub const OPACITY: f32 = 0.8;

const ITERATIONS: usize = 5000;
// const SHAPES_ADJUSTED: usize = 10;
// const ADJUSTMENTS: usize = 100;

pub const TOTAL_SHAPES: usize = 512;

const CUTOFF: usize = 8;

pub const OBJ_IDS: &[u16] = &[
    18, 19, 20, 211, 48, 49, 113, 114, 115, 129, 130, 211, 229, 230, 233, 242, 251, 259, 266, 273,
    279, 280, 281, 282, 419, 420, 503, 504, 505, 693, 695, 697, 699, 701, 907, 939, 1011, 1012,
    1013, 1117, 1118, 1192, 1193, 1196, 1291, 1293, 1348, 1349, 1350, 1351, 1352, 1353, 1354, 1355,
    1387, 1388, 1389, 1390, 1461, 1462, 1463, 1464, 1510, 1511, 1512, 1597, 1738, 1753, 1754, 1757,
    1764, 1765, 1766, 1767, 1768, 1769, 1770, 1771, 1772, 1777, 1778, 1779, 1780, 1835, 1836, 1837,
    1861, 1869, 1870, 1871, 1875, 1876, 1877, 1888,
];

pub const TARGET: &str = "galaxy.jpg";

pub fn process(state: &State, target: &DynamicImage, bg_color: [f32; 3]) {
    let mut level_string = format!(";1,899,2,-29,3,975,36,1,7,255,8,0,9,0,10,0,35,{OPACITY},23,1;1,899,2,-29,3,1005,36,1,7,{},8,{},9,{},10,0,35,1,23,1000;", to_srgb(bg_color[0]) * 255.0, to_srgb(bg_color[1]) * 255.0, to_srgb(bg_color[2]) * 255.0);

    //let mut current_diff = std::i32::MAX;
    for iteration in 0..ITERATIONS {
        let mut shapes = Vec::new();

        for _ in 0..TOTAL_SHAPES {
            let shape = Shape::new_random(target.width(), target.height());
            shapes.push(shape);
        }

        //dbg!(&shapes);
        let mut diff = test_diff(state, &shapes, target)
            .into_iter()
            .enumerate()
            .collect::<Vec<_>>();

        for _ in 0..6 {
            diff.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
            let mut new_shapes = vec![shapes[diff[0].0]];
            for (i, _) in diff[..CUTOFF].iter() {
                for _ in 0..(TOTAL_SHAPES / CUTOFF - 2) {
                    let mut shape = shapes[*i];
                    shape.adjust_random();
                    new_shapes.push(shape);
                }
            }
            while new_shapes.len() < TOTAL_SHAPES {
                let mut shape = shapes[0];
                shape.adjust_random();
                new_shapes.push(shape);
            }
            //assert_eq!(new_shapes.len(), TOTAL_SHAPES);
            shapes = new_shapes;
            diff = test_diff(state, &shapes, target)
                .into_iter()
                .enumerate()
                .collect::<Vec<_>>();
        }

        if diff[0].1 >= 0 {
            continue;
        }

        println!("improvement: {}", -diff[0].1);

        shapes[diff[0].0].paste(state, target, diff[0].0);
        let tint = pollster::block_on(get_tint(state, diff[0].0));

        // dbg!(shapes[diff[0].0]);
        // dbg!(tint.map(|x| (x * 255.0) as u8));

        // if iteration > 200 && diff[0].1 < -1000 {
        //     break;
        // }

        level_string += &shapes[diff[0].0].to_obj_string(
            to_srgb(tint[0]),
            to_srgb(tint[1]),
            to_srgb(tint[2]),
            iteration,
        );
    }

    // let shape = Shape {
    //     img_index: 64,
    //     x: 59,
    //     y: 39,
    //     scale: 0.6976323,
    //     rot: 0.9563002,
    // };
    // dbg!(OBJ_IDS[64]);

    // let diff = test_diff(state, &[shape], target);
    // println!("improvement: {}", -diff[0]);

    // shape.paste(state, target, 0);

    // pollster::block_on(async {
    //     let buffer_slice = state.tint_buffer.slice(..);

    //     let mapping = buffer_slice.map_async(wgpu::MapMode::Read);
    //     state.device.poll(wgpu::Maintain::Wait);
    //     mapping.await.unwrap();

    //     let data: crate::TintBuffer =
    //         *bytemuck::from_bytes(&buffer_slice.get_mapped_range().to_vec());
    //     state.tint_buffer.unmap();
    //     dbg!(&data);
    // });

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

    fs::write("./levelstring.txt", level_string).expect("Unable to write file");
}

pub fn test_diff(state: &State, shapes: &[Shape], target: &DynamicImage) -> Vec<i32> {
    let mut encoder = state
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

    Shape::test_diff(shapes, state, target, &mut encoder);

    state.queue.submit(std::iter::once(encoder.finish()));

    pollster::block_on(get_image_diff(state)).to_vec()
}

pub async fn get_image_diff(state: &State) -> [i32; TOTAL_SHAPES] {
    let buffer_slice = state.tint_buffer.slice(..);

    let mapping = buffer_slice.map_async(wgpu::MapMode::Read);
    state.device.poll(wgpu::Maintain::Wait);
    mapping.await.unwrap();

    let data: TintBuffer = *bytemuck::from_bytes(&buffer_slice.get_mapped_range().to_vec());
    state.tint_buffer.unmap();
    data.diff
}

pub async fn get_tint(state: &State, index: usize) -> [f32; 3] {
    let buffer_slice = state.tint_buffer.slice(..);

    let mapping = buffer_slice.map_async(wgpu::MapMode::Read);
    state.device.poll(wgpu::Maintain::Wait);
    mapping.await.unwrap();

    let data: TintBuffer = *bytemuck::from_bytes(&buffer_slice.get_mapped_range().to_vec());
    state.tint_buffer.unmap();
    data.tint[index].map(|x| x as f32 / data.counts[index] as f32)
}
