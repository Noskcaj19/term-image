use image;
use image::GenericImage;
use image::{DynamicImage, FilterType};
use std::{fs::File, io::BufReader};
use termion::color;
use Options;

pub fn closest_mult(x: u32, base: u32) -> u32 {
    base * ((x as f32) / base as f32).round() as u32
}

pub fn load_image(options: &Options) -> Option<image::DynamicImage> {
    if options.file_name == Some("-".to_owned()) {
        // Read stdin
        use std::io::{stdin, Read};
        let mut buf = Vec::new();
        stdin().read_to_end(&mut buf).ok()?;
        image::load_from_memory(&buf).ok()
    } else {
        if options.auto_detect_format {
            Some(image::open(options.file_name.clone()?).ok()?)
        } else {
            // TODO: Huge release mode compile time increase
            let file = File::open(options.file_name.clone()?).ok()?;
            let buf = BufReader::new(file);
            Some(image::load(buf, options.image_format?).ok()?)
            // panic!("Disabled")
        }
    }
}

pub fn resize_image(
    img: &DynamicImage,
    cell_size: (u32, u32),
    max_size: (u16, u16),
) -> DynamicImage {
    let img = img.resize(
        (max_size.0 as u32) * cell_size.0,
        (max_size.1 as u32) * cell_size.1,
        FilterType::Nearest,
    );

    img.resize_exact(
        closest_mult(img.width(), cell_size.0),
        closest_mult(img.height(), cell_size.1),
        FilterType::Nearest,
    )
}

pub fn rgb_to_ansi(color: color::Rgb) -> color::AnsiValue {
    let r = (u16::from(color.0) * 5 / 255) as u8;
    let g = (u16::from(color.1) * 5 / 255) as u8;
    let b = (u16::from(color.2) * 5 / 255) as u8;
    color::AnsiValue(16 + 36 * r + 6 * g + b)
}
