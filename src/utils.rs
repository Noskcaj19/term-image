use image::imageops::FilterType;
use image::{DynamicImage, GenericImageView};
use libc;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

/// Returns the closest multiple of a base
pub fn closest_mult(x: u32, base: u32) -> u32 {
    base * ((x as f32) / base as f32).round() as u32
}

/// Resizes an image to fit within a max size, then scales an image to fit within a block size
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

/// Returns a Arc reference to a boolean value that is set to true when a "exit"
/// signal is recieved (currently INT, QUIT, TERM, and WINCH).
pub fn get_quit_hook() -> Arc<AtomicBool> {
    let atomic = Arc::new(AtomicBool::new(false));
    for signal in &[libc::SIGINT, libc::SIGQUIT, libc::SIGTERM, libc::SIGWINCH] {
        ::signal_hook::flag::register(*signal, Arc::clone(&atomic))
            .expect("Unable to hook a termination signal");
    }
    atomic
}
