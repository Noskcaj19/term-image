use crate::renderer::{CharSet, DrawStyle, MagicType};
use std::default::Default;

/// Store configuration
#[derive(Debug, Clone)]
pub struct Options {
    // General
    pub file_names: Option<Vec<String>>,
    pub width: Option<usize>,
    pub height: Option<usize>,
    pub isatty: bool,
    pub magic_type: Option<MagicType>,
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
            file_names: None,
            width: None,
            height: None,
            isatty: true,
            magic_type: None,
            truecolor: false,
            draw_style: DrawStyle::Magic,
            char_set: Default::default(),
            blend: true,
            animated: true,
        }
    }
}

impl Default for Options {
    fn default() -> Self {
        Self::new()
    }
}
