use image::{imageops::FilterType, DynamicImage, Rgb as RgbPixel, Rgba};

pub mod ascii;
pub mod block;
pub mod braille;
pub mod iterm;
pub mod kitty;

/// Returns the closest multiple of a base
pub fn closest_mult(x: u32, base: u32) -> u32 {
    base * ((x as f32) / base as f32).round() as u32
}

/// Resizes an image to fit within a max size
pub fn resize_image(
    img: &DynamicImage,
    cell_size: (u32, u32),
    max_size: (u16, u16),
) -> DynamicImage {
    img.resize(
        (u32::from(max_size.0)) * cell_size.0,
        (u32::from(max_size.1)) * cell_size.1,
        FilterType::Nearest,
    )
}

/// Represents a 24bit rgb color
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Rgb(pub (u8, u8, u8));

impl Rgb {
    /// Convert 24bit rgb color to 256 color (8 bit)
    pub fn as_256(&self) -> Ansi {
        let (r, g, b) = self.0;
        let r = (u16::from(r) * 5 / 255) as u8;
        let g = (u16::from(g) * 5 / 255) as u8;
        let b = (u16::from(b) * 5 / 255) as u8;
        Ansi(16 + 36 * r + 6 * g + b)
    }
}

/// Represents a 8bit (ANSI) color
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Ansi(pub u8);

/// Perform alpha premuliplication on a Rgba pixel to remove the alpha
fn premultiply(p: Rgba<u8>, bg: RgbPixel<u8>) -> Rgba<u8> {
    if p[3] == 255 {
        // No transparency
        return p;
    }

    let mut p = p;
    let alpha = f32::from(p[3]) / 255.;

    // eprintln!("{:#?}", bg);
    for (subpixel, bg) in p.0.iter_mut().zip(bg.0.iter().map(|s| f32::from(*s))) {
        *subpixel = (((1. - alpha) * bg) + (alpha * f32::from(*subpixel))) as u8
    }

    p
}
