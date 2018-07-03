use image::ImageFormat;
use std::default::Default;
use unicode_block::DrawMode;

#[derive(Debug, Clone)]
pub struct Options {
    pub file_name: Option<String>,
    pub auto_detect_format: bool,
    pub image_format: Option<ImageFormat>,
    pub ansi_256_color: bool,
    pub ignore_tty: bool,
    pub width: Option<usize>,
    pub height: Option<usize>,
    pub magic: bool,
    // Unicode block
    pub draw_mode: DrawMode,
    pub blend: bool,
    // GIF
    pub animated: bool,
}

impl Options {
    pub fn new() -> Options {
        Options {
            file_name: None,
            auto_detect_format: true,
            image_format: None,
            ansi_256_color: false,
            ignore_tty: false,
            width: None,
            height: None,
            magic: true,
            draw_mode: Default::default(),
            blend: true,
            animated: true,
        }
    }
}
