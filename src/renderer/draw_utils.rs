use image::Rgba;
use termion::color;

/// Convert full color rgb to 256 color
pub fn rgb_to_ansi(color: color::Rgb) -> color::AnsiValue {
    let r = (u16::from(color.0) * 5 / 255) as u8;
    let g = (u16::from(color.1) * 5 / 255) as u8;
    let b = (u16::from(color.2) * 5 / 255) as u8;
    color::AnsiValue(16 + 36 * r + 6 * g + b)
}

/// Perform alpha premuliplication on a Rgba pixel to remove the alpha
pub fn premultiply(p: Rgba<u8>) -> Rgba<u8> {
    if p[3] == 255 {
        // No transparency
        return p;
    }

    let mut p = p;
    let alpha = f32::from(p[3]) / 255.;
    let bg = 0.;

    for pixel in p.0.iter_mut() {
        *pixel = (((1. - alpha) * bg) + (alpha * f32::from(*pixel))) as u8
    }

    p
}
