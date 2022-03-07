const TARGET: &str = "planet.jpeg";

// import the image and resize it to a fixed width

use image::{imageops::FilterType, DynamicImage, RgbImage, RgbaImage};
use image::{GenericImageView, ImageBuffer, Rgb, Rgba};

macro_rules! create_obj_ids {
    {$symbol:ident, [$($id:literal,)*]} => {
        pub const $symbol: &[(u16, &[u8])] = &[
            $(($id, include_bytes!(concat!("../objects/", stringify!($id), "/main.png")))),*
        ];
    };

}

create_obj_ids! {
    OBJ_IDS, [
        211, 259, 266, 273, 280, 693, 695, 697, 699, 701, 725, 1011, 1012, 1013, 1102, 1106, 1111,
        1112, 1113, 1114, 1115, 1116, 1117, 1118, 1348, 1351, 1352, 1353, 1354, 1355, 1442, 1443, 1461,
        1462, 1463, 1464, 1596, 1597, 1608, 1609, 1610, 1753, 1754, 1757, 1764, 1765, 1766, 1767, 1768,
        1769, 1837, 1835, 1869, 1870, 1871, 1874, 1875, 1886, 1887, 1888,
    ]
}
use rand::Rng;
use rayon::prelude::*;

#[derive(Debug, Clone, Copy)]
struct Shape {
    img_index: usize,
    x: i32,
    y: i32,
    scale: f32,
    rot: f32,
}

const CHUNK_SIZE: usize = 1024 * 3;
const CHUNK_SIZE4: usize = 1024 * 4;

impl Shape {
    fn paste(
        &self,
        img: &mut RgbImage,
        obj_imgs: &[ImageBuffer<Rgba<u8>, Vec<u8>>],
        target: &DynamicImage,
        img_alpha: f32,
    ) {
        let obj_img = &obj_imgs[self.img_index];
        let width = img.width();
        let height = img.height();
        let obj_width = obj_img.width();
        let obj_height = obj_img.height();

        let avg_color = obj_img
            .as_raw()
            .par_chunks(CHUNK_SIZE4)
            .enumerate()
            .map(|(chunk_i, chunk)| {
                let mut sum = ([0.0, 0.0, 0.0], 0u32);
                for i in (0..chunk.len()).step_by(4) {
                    let alpha = chunk[i + 3] as f32 / 255.0;
                    let index = (chunk_i * CHUNK_SIZE + i) / 3;
                    let mut x = (index % obj_width as usize) as f32;
                    let mut y = (index / obj_width as usize) as f32;

                    // translate to center
                    x -= obj_width as f32 / 2.0;
                    y -= obj_height as f32 / 2.0;

                    let (mut x, mut y) = rotate_point(x, y, -self.rot);

                    x += self.x as f32 / self.scale;
                    y += self.y as f32 / self.scale;

                    x *= self.scale;
                    y *= self.scale;

                    // continue if outside bounds
                    if x < 0.0 || x > width as f32 - 1.0 || y < 0.0 || y > height as f32 - 1.0 {
                        continue;
                    }

                    // get image pixel
                    let c = target.get_pixel(x as u32, y as u32);
                    sum.0[0] += c.0[0] as f32 / 255.0 * alpha;
                    sum.0[1] += c.0[1] as f32 / 255.0 * alpha;
                    sum.0[2] += c.0[2] as f32 / 255.0 * alpha;
                    sum.1 += 1;
                }
                sum
            })
            .reduce(
                || ([0.0, 0.0, 0.0], 0),
                |(sum, sc), (next, c)| {
                    (
                        [sum[0] + next[0], sum[1] + next[1], sum[2] + next[2]],
                        sc + c,
                    )
                },
            );
        //dbg!(avg_color);
        let avg_color = avg_color.0.map(|a| a / avg_color.1 as f32);
        //dbg!(avg_color);

        img.as_mut()
            .par_chunks_mut(CHUNK_SIZE)
            .enumerate()
            .for_each(|(chunk_i, chunk)| {
                for i in (0..chunk.len()).step_by(3) {
                    let index = (chunk_i * CHUNK_SIZE + i) / 3;
                    let x = (index % width as usize) as u32;
                    let y = (index / width as usize) as u32;

                    let mut obj_x = x as f32;
                    let mut obj_y = y as f32;
                    // scale around center
                    obj_x /= self.scale;
                    obj_y /= self.scale;
                    obj_x -= self.x as f32 / self.scale;
                    obj_y -= self.y as f32 / self.scale;

                    let (mut obj_x, mut obj_y) = rotate_point(obj_x, obj_y, self.rot);
                    // translate to center
                    obj_x += obj_width as f32 / 2.0;
                    obj_y += obj_height as f32 / 2.0;

                    //dbg!((obj_x, obj_y));

                    // return if out of bounds
                    if obj_x < 0.0
                        || obj_x >= obj_width as f32
                        || obj_y < 0.0
                        || obj_y >= obj_height as f32
                    {
                        continue;
                    }
                    let obj_pixel = *obj_img.get_pixel(obj_x as u32, obj_y as u32);

                    // set pixel
                    let alpha = (obj_pixel[3] as f32 / 255.0) * img_alpha;

                    chunk[i] = (obj_pixel[0] as f32 * avg_color[0] * alpha
                        + chunk[i] as f32 * (1.0 - alpha)) as u8;
                    chunk[i + 1] = (obj_pixel[1] as f32 * avg_color[1] * alpha
                        + chunk[i + 1] as f32 * (1.0 - alpha))
                        as u8;
                    chunk[i + 2] = (obj_pixel[2] as f32 * avg_color[2] * alpha
                        + chunk[i + 2] as f32 * (1.0 - alpha))
                        as u8;
                }
            });
    }

    fn new_random(width: u32, height: u32, img_index: usize) -> Shape {
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

    fn adjust_random(&mut self) {
        self.x = self.x as i32 + rand::thread_rng().gen_range(-3i32..=3);
        self.y = self.y as i32 + rand::thread_rng().gen_range(-3i32..=3);
        self.scale *= rand::thread_rng().gen_range(0.9..1.1);
        self.rot += rand::thread_rng().gen_range(-0.1..0.1);
    }
}

fn rotate_point(x: f32, y: f32, angle: f32) -> (f32, f32) {
    let cos = angle.cos();
    let sin = angle.sin();
    (x * cos - y * sin, x * sin + y * cos)
}

const SHAPES_PER_OBJ: usize = 50;
const ITERATIONS: usize = 5;
const SHAPES_ADJUSTED: usize = 10;
const ADJUSTMENTS: usize = 100;

fn main() {
    let obj_imgs = OBJ_IDS
        .par_iter()
        .map(|id| {
            let img = image::open(format!("objects/{}/main.png", id)).unwrap();
            img.to_rgba8()
        })
        .collect::<Vec<RgbaImage>>();

    let width = 200;
    let img = image::open(TARGET).unwrap();
    let aspect_ratio = img.width() as f32 / img.height() as f32;
    let height: u32 = (width as f32 * aspect_ratio) as u32;

    let target = img.resize(width, height, FilterType::Nearest);

    //dbg!(img_difference_par(&target.clone().into_rgb8(), &target));

    // create new blank image
    let mut new_img = RgbImage::new(target.width(), target.height());
    let img_alpha = 1.0;

    // let shape = Shape {
    //     img_index: 3,
    //     x: 110,
    //     y: 60,
    //     scale: 0.5,
    //     rot: 0.0,
    // };

    // let shape2 = Shape {
    //     img_index: 3,
    //     x: 130,
    //     y: 70,
    //     scale: 0.7,
    //     rot: 0.6,
    // };

    // let shape3 = Shape {
    //     img_index: 3,
    //     x: 100,
    //     y: 50,
    //     scale: 0.7,
    //     rot: 0.1,
    // };
    // shape.paste(&mut new_img, &obj_imgs, &target, img_alpha);
    // shape2.paste(&mut new_img, &obj_imgs, &target, img_alpha);
    // shape3.paste(&mut new_img, &obj_imgs, &target, img_alpha);
    // dbg!(shape);
    // new_img.save("output.png").unwrap();

    for _ in 0..ITERATIONS {
        let mut shapes = Vec::new();
        for img_index in 0..20 {
            for _ in 0..SHAPES_PER_OBJ {
                shapes.push(Shape::new_random(
                    target.width(),
                    target.height(),
                    img_index,
                ));
            }
        }

        let mut evaled = shapes
            .par_iter()
            .map(|shape| {
                let mut new_img = new_img.clone();
                shape.paste(&mut new_img, &obj_imgs, &target, img_alpha);
                let diff = img_difference_par(&new_img, &target);
                (*shape, diff)
            })
            .collect::<Vec<(Shape, f32)>>();
        // sort by diff
        evaled.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

        println!("pick: {}", evaled[0].1);
        // take best

        let mut evaled = evaled[..SHAPES_ADJUSTED]
            .par_iter()
            .map(|(mut shape, mut diff)| {
                for _ in 0..ADJUSTMENTS {
                    let mut new_shape = shape;
                    new_shape.adjust_random();
                    let mut new_img = new_img.clone();
                    new_shape.paste(&mut new_img, &obj_imgs, &target, img_alpha);

                    let new_diff = img_difference_par(&new_img, &target);
                    if new_diff < diff {
                        shape = new_shape;
                        diff = new_diff;
                    }
                }

                (shape, diff)
            })
            .collect::<Vec<(Shape, f32)>>();
        // sort by diff
        evaled.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        println!("adjusted pick: {}", evaled[0].1);
        dbg!(evaled[0].0);
        // take best
        evaled[0]
            .0
            .paste(&mut new_img, &obj_imgs, &target, img_alpha);
        new_img.save("output.png").unwrap();
    }

    //shapes[0].paste(&mut new_img, &obj_imgs, img_alpha);
}

fn img_difference_par(img: &RgbImage, target: &DynamicImage) -> f32 {
    let total_difference: f32 = img
        .as_raw()
        .par_chunks(CHUNK_SIZE)
        .enumerate()
        .map(|(chunk_i, chunk)| {
            let mut total_difference = 0.0;
            for i in (0..chunk.len()).step_by(3) {
                let p1 = [chunk[i], chunk[i + 1], chunk[i + 2]];
                let index = (chunk_i * CHUNK_SIZE + i) / 3;
                let p2 = target
                    .get_pixel(
                        (index % target.width() as usize) as u32,
                        (index / target.width() as usize) as u32,
                    )
                    .0;

                let difference = color_diff(&p1, &p2);
                // + (p2[3] - p1[3]).pow(2)) as f32;
                total_difference += difference;
            }
            total_difference
        })
        .sum();

    total_difference / (img.width() * img.height()) as f32 //.sqrt()
}

#[inline(always)]
fn color_diff(p1: &[u8], p2: &[u8]) -> f32 {
    let dr = p1[0] as f32 - p2[0] as f32;
    let dg = p1[1] as f32 - p2[1] as f32;
    let db = p1[2] as f32 - p2[2] as f32;
    let rdash = (p1[0] as f32 - p2[0] as f32) * 0.5;
    ((2.0 + rdash / 256.0) * dr * dr + 4.0 * dg * dg + (2.0 + (255.0 - rdash) / 256.0) * db * db)
}
