use clap::{App, Arg};
use image::ImageFormat;
use unicode_block::DrawMode;
use Options;

pub fn get_options() -> Options {
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
                .long("ansi")
                .alias("256")
                .short("a")
                .help("Use only ansi 256 colors"),
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
        .arg(
            Arg::with_name("braille")
                .long("braille")
                .short("d")
                .help("Enable unicode braille dot character rendering"),
        )
        .arg(Arg::with_name("file_name").required(true))
        .get_matches();

    let mut options = Options::new();

    options.truecolor = !matches.is_present("256_colors");
    options.file_name = matches.value_of("file_name").map(str::to_string);
    options.blend = !matches.is_present("no_blending");
    options.ignore_tty = matches.is_present("force_tty");
    options.animated = !matches.is_present("still");
    options.magic = !matches.is_present("no_magic");
    options.braille = matches.is_present("braille");
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
