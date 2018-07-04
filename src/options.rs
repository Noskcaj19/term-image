use image::ImageFormat;
use std::default::Default;
use unicode_block::CharSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrawStyle {
    UnicodeBlock,
    Braille,
    Magic,
}

#[derive(Debug, Clone)]
pub struct Options {
    // General
    pub file_name: Option<String>,
    pub auto_detect_format: bool,
    pub image_format: Option<ImageFormat>,
    pub width: Option<usize>,
    pub height: Option<usize>,
    pub no_tty: bool,
    // Display
    pub truecolor: bool,
    pub draw_style: DrawStyle,
    // Unicode block
    pub char_set: CharSet,
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
            width: None,
            height: None,
            no_tty: false,
            truecolor: false,
            draw_style: DrawStyle::Magic,
            char_set: Default::default(),
            blend: true,
            animated: true,
        }
    }
}
