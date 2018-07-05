mod ascii;
mod braille;
mod iterm;
mod unicode_block;

use image;
use std::fs::File;
use utils;
use Options;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CharSet {
    All,
    NoSlopes,
    Blocks,
    Halfs,
}

impl Default for CharSet {
    fn default() -> CharSet {
        CharSet::All
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MagicType {
    Iterm,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrawStyle {
    UnicodeBlock,
    Braille,
    Ascii,
    Magic,
}

pub fn render_image(options: &Options, term_size: (u16, u16)) {
    let file_name = &options.file_name.clone().unwrap();

    if file_name.ends_with(".gif") && options.animated {
        let f = File::open(&file_name).expect("File not found");

        let decoder = image::gif::Decoder::new(f);
        use image::ImageDecoder;
        let frames = decoder.into_frames().expect("error decoding gif");

        match options.draw_style {
            DrawStyle::Braille => {
                braille::print_frames(&options, term_size, frames);
            }
            DrawStyle::UnicodeBlock => {
                unicode_block::print_frames(&options, term_size, frames);
            }
            DrawStyle::Ascii => {
                ascii::print_frames(&options, term_size, frames);
            }
            DrawStyle::Magic => match options.magic_type {
                Some(MagicType::Iterm) => {
                    iterm::display(&options, term_size, file_name).unwrap();
                }
                None => {
                    eprintln!("No known magic display modes");
                }
            },
        }
    } else {
        match options.draw_style {
            DrawStyle::Magic => match options.magic_type {
                Some(MagicType::Iterm) => {
                    iterm::display(&options, term_size, &file_name).unwrap();
                }
                None => {
                    eprintln!("No known magic display modes");
                }
            },
            style => {
                let img = match utils::load_image(&file_name) {
                    Some(img) => img,
                    None => {
                        eprintln!("Error: Unable to open file for reading");
                        return;
                    }
                };
                match style {
                    DrawStyle::Braille => {
                        braille::display(&options, term_size, &img);
                    }
                    DrawStyle::UnicodeBlock => {
                        unicode_block::print_image(&options, term_size, &img);
                    }
                    DrawStyle::Ascii => {
                        ascii::display(&options, term_size, &img);
                    }
                    DrawStyle::Magic => panic!("Impossible state"),
                }
            }
        }
    }
}
