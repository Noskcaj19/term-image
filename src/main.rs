#![feature(iterator_step_by)]

extern crate clap;
extern crate failure;
extern crate gif;
extern crate image;
extern crate iterm2;
extern crate libc;
extern crate signal_hook;
extern crate termion;

mod iterm;
mod options;
mod unicode_block;
mod utils;

use image::ImageFormat;
use unicode_block::DrawMode;

use std::env;

fn get_options() -> options::Options {
    use clap::{App, Arg};
    let matches = App::new("Terminal Image Viewer")
        .author("Noskcaj")
        .about("Shows images in your terminal")
        .arg(
            Arg::with_name("file_format_arg")
                .takes_value(true)
                .value_name("file_format")
                .short("t")
                .possible_values(&["jpg", "jpeg", "png", "gif", "ico"])
                .help("Sets the image type"),
        )
        .arg(
            Arg::with_name("256_colors")
                .long("256")
                .help("Use only 256 colors"),
        )
        .arg(
            Arg::with_name("no_blending")
                .long("noblend")
                .short("b")
                .help("Disable blending characters"),
        )
        .arg(
            Arg::with_name("all")
                .long("all")
                .help("Use all unicode drawing characters")
                .conflicts_with_all(&["no_slopes", "only_blocks", "only_halfs"]),
        )
        .arg(
            Arg::with_name("no_slopes")
                .long("noslopes")
                .help("Disable angled unicode character (if they are wide in your font)")
                .conflicts_with_all(&["all", "only_blocks", "only_halfs"]),
        )
        .arg(
            Arg::with_name("only_blocks")
                .long("blocks")
                .help("Only use unicode block characters")
                .conflicts_with_all(&["all", "no_slopes", "only_halfs"]),
        )
        .arg(
            Arg::with_name("only_halfs")
                .long("halfs")
                .help("Only use unicode half blocks")
                .conflicts_with_all(&["all", "no_slopes", "only_blocks"]),
        )
        .arg(
            Arg::with_name("width")
                .long("width")
                .short("w")
                .takes_value(true)
                .help("Override max display width (maintains aspect ratio)"),
        )
        .arg(
            Arg::with_name("height")
                .long("height")
                .short("h")
                .takes_value(true)
                .help("Override max display height"),
        )
        .arg(
            Arg::with_name("force_tty")
                .long("force-tty")
                .help("Don't detect tty"),
        )
        .arg(
            Arg::with_name("still")
                .long("still")
                .short("s")
                .help("Don't animate images"),
        )
        .arg(
            Arg::with_name("no_magic")
                .long("no-magic")
                .short("m")
                .help("Disable high-def rendering magic"),
        )
        .arg(Arg::with_name("file_name").required(true))
        .get_matches();

    let mut options = options::Options::new();

    options.ansi_256_color = matches.is_present("256_colors");
    options.file_name = matches.value_of("file_name").map(str::to_string);
    options.blend = !matches.is_present("no_blending");
    options.ignore_tty = matches.is_present("force_tty");
    options.animated = !matches.is_present("still");
    options.magic = !matches.is_present("no_magic");
    options.width = matches
        .value_of("width")
        .map(str::to_string)
        .and_then(|w| w.parse().ok());
    options.height = matches
        .value_of("height")
        .map(str::to_string)
        .and_then(|h| h.parse().ok());

    if let Some(name) = &options.file_name {
        if options.auto_detect_format {
            if name.ends_with(".gif") {
                options.image_format = Some(ImageFormat::GIF)
            }
        }
    }

    if let Some(format) = matches.value_of("file_format_arg") {
        options.auto_detect_format = false;
        options.image_format = match format.to_lowercase().as_str() {
            "jpg" | "jpeg" => Some(ImageFormat::JPEG),
            "png" => Some(ImageFormat::PNG),
            "gif" => Some(ImageFormat::GIF),
            "ico" => Some(ImageFormat::ICO),
            _ => panic!("Invalid image format in match"),
        }
    }

    if matches.is_present("no_slopes") {
        options.draw_mode = DrawMode::NoSlopes;
    } else if matches.is_present("only_blocks") {
        options.draw_mode = DrawMode::Blocks;
    } else if matches.is_present("only_halfs") {
        options.draw_mode = DrawMode::Halfs;
    }

    options
}

fn main() {
    let options = get_options();

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
            unicode_block::print_frames(&options, term_size, frames);
        }
        _ => {
            let img = match utils::load_image(&options) {
                Some(img) => img,
                None => {
                    eprintln!("Error: Unable to open file for reading");
                    return;
                }
            };

            unicode_block::print_image(&options, term_size, &img);
        }
    }
}
