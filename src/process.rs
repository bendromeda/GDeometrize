use image::{DynamicImage, RgbaImage};

use crate::{shape::*, State};

pub const OPACITY: f32 = 1.0;

pub fn process(state: &State, target: &DynamicImage, spritesheet: &RgbaImage) {
    // for i in 0..400 {
    //     let shape =
    //         shape::Shape::new_random(target.width(), target.height(), i % shape::OBJ_IDS.len());
    //     shape.paste(state, target, spritesheet).unwrap();
    // }

    let shape = Shape {
        img_index: 0,
        x: 200,
        y: 200,
        scale: 5.0,
        rot: 0.5,
    };
    shape.paste(state, target, spritesheet).unwrap();
}

// for x in 0..80 {
//     for y in 0..50 {
//         let angle = if x % 2 == 0 {
//             0.0
//         } else {
//             std::f32::consts::PI / 2.0
//         };
//         let shape = Shape {
//             img_index: 5,
//             x: (x * 10) as i32,
//             y: (y * 10) as i32,
//             scale: 10.0 / 60.0,
//             rot: 0.0 + angle,
//         };
//         shape.paste(state, target, spritesheet).unwrap();
//         let shape = Shape {
//             img_index: 5,
//             x: (x * 10) as i32,
//             y: (y * 10) as i32,
//             scale: 10.0 / 60.0,
//             rot: std::f32::consts::PI + angle,
//         };
//         shape.paste(state, target, spritesheet).unwrap();
//     }
// }
