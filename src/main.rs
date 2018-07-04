#![feature(iterator_step_by)]

extern crate clap;
extern crate failure;
extern crate gif;
extern crate image;
extern crate iterm2;
extern crate libc;
extern crate signal_hook;
extern crate termion;

mod args;
mod braille;
mod iterm;
mod options;
use options::DrawStyle;
pub use options::*;
mod unicode_block;
mod utils;

fn main() {
    let options = args::get_options();

    if !options.no_tty && !options.width.is_some() && !options.width.is_some() {
        if !termion::is_tty(&std::fs::File::create("/dev/stdout").unwrap()) {
            return;
        }
    }

    let term_size = if options.width.is_some() || options.height.is_some() {
        (
            options.width.unwrap_or(std::usize::MAX) as u16,
            options.height.unwrap_or(std::usize::MAX) as u16,
        )
    } else if options.no_tty {
        (80, 25)
    } else {
        match termion::terminal_size() {
            Ok(size) => (size.0 - 4, size.1 - 8),
            Err(_) => return,
        }
    };

    let file_name = &options.file_name.clone().unwrap();

    if file_name.ends_with(".gif") && options.animated {
        let f = std::fs::File::open(&file_name).expect("File not found");

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
        let img = match utils::load_image(&file_name) {
            Some(img) => img,
            None => {
                eprintln!("Error: Unable to open file for reading");
                return;
            }
        };
        match options.draw_style {
            DrawStyle::Braille => {
                braille::display(&options, term_size, &img);
            }
            DrawStyle::UnicodeBlock => {
                unicode_block::print_image(&options, term_size, &img);
            }
            DrawStyle::Magic => match options.magic_type {
                Some(MagicType::Iterm) => {
                    iterm::display(&options, term_size, &file_name).unwrap();
                }
                None => {
                    eprintln!("No known magic display modes");
                }
            },
        }
    }
}
