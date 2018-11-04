use image::{self, DynamicImage, FilterType, GenericImageView};
use libc;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

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
        (u32::from(max_size.0)) * cell_size.0,
        (u32::from(max_size.1)) * cell_size.1,
        FilterType::Nearest,
    );

    img.resize_exact(
        closest_mult(img.width(), cell_size.0),
        closest_mult(img.height(), cell_size.1),
        FilterType::Nearest,
    )
}

pub fn get_quit_hook() -> Arc<AtomicBool> {
    let atomic = Arc::new(AtomicBool::new(false));
    for signal in &[libc::SIGINT, libc::SIGQUIT, libc::SIGTERM, libc::SIGWINCH] {
        ::signal_hook::flag::register(*signal, Arc::clone(&atomic))
            .expect("Unable to hook a termination signal");
    }
    atomic
}
