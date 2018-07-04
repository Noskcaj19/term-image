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
pub use options::Options;
mod unicode_block;
mod utils;

use std::env;

fn main() {
    let options = args::get_options();

    if !options.ignore_tty && !options.width.is_some() && !options.width.is_some() {
        if !termion::is_tty(&std::fs::File::create("/dev/stdout").unwrap()) {
            return;
        }
    }

    let term_size = if options.width.is_some() || options.height.is_some() {
        (
            options.width.unwrap_or(std::usize::MAX) as u16,
            options.height.unwrap_or(std::usize::MAX) as u16,
        )
    } else if options.ignore_tty {
        (80, 25)
    } else {
        match termion::terminal_size() {
            Ok(size) => (size.0 - 4, size.1 - 8),
            Err(_) => return,
        }
    };

    let file_name = &options.file_name.clone().unwrap();

    if options.magic {
        if let Ok(prog) = env::var("TERM_PROGRAM") {
            if prog == "iTerm.app" {
                iterm::display(&options, term_size, &file_name).unwrap();
                return;
            }
        }
    }

    match options.image_format {
        Some(image::ImageFormat::GIF) if options.animated => {
            let f = std::fs::File::open(&file_name).expect("File not found");

            let decoder = image::gif::Decoder::new(f);
            use image::ImageDecoder;
            let frames = decoder.into_frames().expect("error decoding gif");
            if options.braille {
                braille::print_frames(&options, term_size, frames);
            } else {
                unicode_block::print_frames(&options, term_size, frames);
            }
        }
        _ => {
            let img = match utils::load_image(&options) {
                Some(img) => img,
                None => {
                    eprintln!("Error: Unable to open file for reading");
                    return;
                }
            };

            if options.braille {
                braille::display(&options, term_size, &img);
            } else {
                unicode_block::print_image(&options, term_size, &img);
            }
        }
    }
}
