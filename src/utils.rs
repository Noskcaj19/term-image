use image;
use image::GenericImage;
use image::{DynamicImage, FilterType};
use libc;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use termion::color;

pub fn closest_mult(x: u32, base: u32) -> u32 {
    base * ((x as f32) / base as f32).round() as u32
}

pub fn load_image(path: &str) -> Option<image::DynamicImage> {
    // Read stdin
    if path == "-" {
        use std::io::{stdin, Read};
        let mut buf = Vec::new();
        stdin().read_to_end(&mut buf).ok()?;
        image::load_from_memory(&buf).ok()
    } else {
        Some(image::open(path).ok()?)
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

pub fn get_quit_hook() -> Arc<AtomicBool> {
    let atomic = Arc::new(AtomicBool::new(false));
    for signal in &[libc::SIGINT, libc::SIGQUIT, libc::SIGTERM, libc::SIGWINCH] {
        ::signal_hook::flag::register(*signal, Arc::clone(&atomic))
            .expect("Unable to hook a termination signal");
    }
    atomic
}
