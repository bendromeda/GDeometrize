use image::DynamicImage;
use std::fs;

use crate::{shape::*, State, TintBuffer};

pub const OPACITY: f32 = 0.8;

const ITERATIONS: usize = 3000;
// const SHAPES_ADJUSTED: usize = 10;
// const ADJUSTMENTS: usize = 100;

pub const TOTAL_SHAPES: usize = 2048;

const CUTOFF: usize = 32;
const PASSED_ON: usize = 600;
pub const ADJUSTMENTS: usize = 24;

pub const OBJ_IDS: &[u16] = &[
    18, 19, 20, 21, 41, 48, 49, 106, 107, 110, 113, 114, 115, 123, 124, 125, 126, 127, 128, 129,
    130, 131, 134, 151, 152, 153, 157, 158, 159, 190, 211, 225, 226, 227, 228, 229, 230, 231, 232,
    233, 234, 235, 237, 238, 239, 240, 241, 242, 251, 259, 266, 273, 277, 278, 279, 280, 281, 282,
    283, 284, 285, 406, 407, 408, 409, 410, 411, 412, 413, 414, 419, 420, 448, 449, 450, 451, 452,
    498, 499, 500, 501, 503, 504, 505, 578, 579, 580, 581, 582, 583, 584, 587, 588, 589, 590, 591,
    592, 593, 596, 597, 598, 599, 600, 601, 602, 605, 606, 607, 608, 609, 610, 611, 614, 615, 616,
    617, 618, 619, 620, 693, 695, 697, 699, 701, 867, 868, 869, 870, 871, 872, 873, 874, 877, 878,
    880, 881, 882, 883, 884, 885, 890, 891, 893, 894, 895, 606, 907, 908, 909, 910, 936, 937, 938,
    939, 940, 941, 942, 1011, 1012, 1013, 1014, 1015, 1016, 1043, 1044, 1047, 104, 1062, 1063,
    1064, 1065, 1066, 1067, 1068, 1069, 1099, 110, 1101, 1102, 1103, 1104, 1105, 1106, 1107, 1109,
    1112, 1113, 1114, 1115, 1116, 1117, 1118, 1120, 1122, 1123, 1124, 1125, 1126, 1127, 1132, 1133,
    1134, 1135, 1136, 1137, 1138, 1139, 1191, 1192, 1193, 1196, 1197, 1198, 1228, 1229, 1230, 1231,
    1232, 1233, 1234, 1235, 1236, 1237, 1238, 1239, 1240, 1269, 1270, 1291, 1293, 1348, 1349, 1350,
    1351, 1352, 1353, 1354, 1355, 1356, 1357, 1358, 1359, 1360, 1361, 1362, 1363, 1364, 1365, 1366,
    1367, 1368, 1369, 1370, 1371, 1372, 1373, 1374, 1375, 1376, 1377, 1378, 1379, 1380, 1381, 1382,
    1383, 1384, 1385, 1386, 1387, 1388, 1389, 1390, 1391, 1392, 1393, 1394, 1395, 1431, 1432, 1433,
    1434, 1435, 1436, 1437, 1438, 1439, 1440, 1441, 1442, 1443, 1444, 1445, 1446, 1447, 1448, 1449,
    1450, 1451, 1452, 1453, 1454, 1455, 1456, 1457, 1458, 1459, 1460, 1461, 1462, 1463, 1464, 1471,
    1472, 1473, 1496, 1507, 1510, 1511, 1512, 1513, 1514, 1515, 1529, 1530, 1531, 1532, 1533, 1534,
    1535, 1538, 1539, 1540, 1596, 1597, 1608, 1609, 1610, 1621, 1622, 1623, 1624, 1625, 1627, 1628,
    1629, 1630, 1631, 1632, 1633, 1634, 1635, 1636, 1738, 1753, 1754, 1757, 1764, 1765, 1766, 1767,
    1768, 1769, 1770, 1771, 1772, 1777, 1778, 1779, 1780, 1835, 1836, 1837, 1861, 1869, 1870, 1871,
    1875, 1876, 1877, 1888,
];

pub const TARGET: &str = "seal.png";

pub async fn process(state: &State, bg_color: [f32; 3]) {
    let mut level_string = format!(";1,899,2,-29,3,975,36,1,7,255,8,0,9,0,10,0,35,{OPACITY},23,1;1,899,2,-29,3,1005,36,1,7,{},8,{},9,{},10,0,35,1,23,1000;", to_srgb(bg_color[0]) * 255.0, to_srgb(bg_color[1]) * 255.0, to_srgb(bg_color[2]) * 255.0);
    let mut shapes = Vec::new();

    for iteration in 0..ITERATIONS {
        pollster::block_on(crate::render(
            state,
            &format!("./frames/anim{:04}.png", iteration),
        ));
        while shapes.len() < TOTAL_SHAPES {
            let shape = Shape::new_random(state.target_size.width, state.target_size.height);
            shapes.push(shape);
        }

        //dbg!(&shapes);
        let mut diff = test_diff(state, &shapes)
            .into_iter()
            .enumerate()
            .collect::<Vec<_>>();

        for j in 0..ADJUSTMENTS {
            diff.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
            let mut new_shapes = vec![shapes[diff[0].0]];
            for (i, _) in diff[..CUTOFF].iter() {
                for _ in 0..(TOTAL_SHAPES / CUTOFF - 2) {
                    let mut shape = shapes[*i];
                    shape.adjust_random(j);
                    new_shapes.push(shape);
                }
            }
            while new_shapes.len() < TOTAL_SHAPES {
                let mut shape = shapes[0];
                shape.adjust_random(j);
                new_shapes.push(shape);
            }
            //assert_eq!(new_shapes.len(), TOTAL_SHAPES);
            shapes = new_shapes;
            diff = test_diff(state, &shapes)
                .into_iter()
                .enumerate()
                .collect::<Vec<_>>();
        }

        if diff[0].1 >= 0 {
            continue;
        }
        println!("frame {} - improvement: {}", iteration, -diff[0].1);
        shapes[diff[0].0].paste(state, diff[0].0);
        let tint = pollster::block_on(get_tint(state, diff[0].0));

        //dbg!(shapes[diff[0].0]);
        // dbg!(tint.map(|x| (x * 255.0) as u8));

        // if iteration > 200 && diff[0].1 < -1000 {
        //     break;
        // }
        //dbg!(shapes[diff[0].0]);

        level_string += &shapes[diff[0].0].to_obj_string(
            to_srgb(tint[0]),
            to_srgb(tint[1]),
            to_srgb(tint[2]),
            iteration,
        );

        shapes = shapes[1..(PASSED_ON + 1)].to_vec();
    }

    crate::render(state, &format!("./frames/anim{:04}.png", ITERATIONS)).await;

    // let shape = Shape {
    //     img_index: 47,
    //     x: 100,
    //     y: 100,
    //     scale: 3.0,
    //     rot: 0.0,
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

pub fn test_diff(state: &State, shapes: &[Shape]) -> Vec<i32> {
    let mut encoder = state
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

    Shape::test_diff(shapes, state, &mut encoder);

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
