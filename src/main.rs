const TARGET: &str = "planet.jpeg";

// import the image and resize it to a fixed width

use image::{imageops::FilterType, DynamicImage, RgbaImage};
use image::{GenericImageView, Rgba};

fn main() {
    let width = 100;
    let img = image::open(TARGET).unwrap();
    let aspect_ratio = img.width() as f32 / img.height() as f32;
    let height: u32 = (width as f32 * aspect_ratio) as u32;

    let target = img.resize(width, height, FilterType::Nearest);

    // create new blank image
    let mut new_img = RgbaImage::new(target.width(), target.height());
}

fn img_difference(img: &RgbaImage, target: &DynamicImage) -> f32 {
    // compare every pixel and output the average difference
    let mut total_difference = 0.0;
    for (x, y, Rgba(p1)) in img.enumerate_pixels() {
        let p2 = target.get_pixel(x, y).0.map(|c| c as i32);
        let p1 = p1.map(|c| c as i32);
        let difference = ((p2[0] - p1[1]).pow(2)
            + (p2[1] - p1[1]).pow(2)
            + (p2[2] - p1[1]).pow(2)
            + (p2[3] - p1[3]).pow(2)) as f32;
        total_difference += difference.sqrt();
    }
    let avg = total_difference / (img.width() * img.height()) as f32;
    avg / 510.0
}
