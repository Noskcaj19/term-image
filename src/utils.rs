use image;
use options::Options;
use std::{fs::File, io::BufReader};

pub fn closest_mult(x: u32, base: u32) -> u32 {
    base * ((x as f32) / base as f32).round() as u32
}

pub fn load_image(options: &Options) -> Option<image::DynamicImage> {
    Some(if options.auto_detect_format {
        image::open(options.file_name.clone()?).ok()?
    } else {
        // TODO: Huge release mode compile time increase
        let file = File::open(options.file_name.clone()?).ok()?;
        let buf = BufReader::new(file);
        image::load(buf, options.image_format?).ok()?
        // panic!("Disabled")
    })
}
