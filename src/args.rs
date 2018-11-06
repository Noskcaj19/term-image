use clap::{App, Arg};
use options::Options;
use renderer::{CharSet, DrawStyle, MagicType};
use std::{env, fs::File};
use termion;

/// Parse cli args into Options
pub fn get_options() -> Options {
    let matches = App::new("Terminal Image Viewer")
        .author("Noskcaj")
        .about("Shows images in your terminal")
        .arg(
            Arg::with_name("256_colors")
                .long("ansi")
                .visible_alias("256")
                .short("a")
                .help("Use only ansi 256 colors"),
        )
        .arg(
            Arg::with_name("truecolor_override")
                .long("truecolor")
                .short("t")
                .help("Force truecolor even in non-supportive terminals"),
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
                .conflicts_with_all(&["no_slopes", "only_blocks", "only_halfs"])
                .requires_ifs(&[("draw_style", "block"), ("draw_style", "b")]),
        )
        .arg(
            Arg::with_name("no_slopes")
                .long("no-slopes")
                .help("Disable angled unicode character (if they are wide in your font)")
                .conflicts_with_all(&["all", "only_blocks", "only_halfs"])
                .requires_ifs(&[("draw_style", "block"), ("draw_style", "b")]),
        )
        .arg(
            Arg::with_name("only_blocks")
                .long("blocks")
                .help("Only use unicode block characters")
                .conflicts_with_all(&["all", "no_slopes", "only_halfs"])
                .requires_ifs(&[("draw_style", "block"), ("draw_style", "b")]),
        )
        .arg(
            Arg::with_name("only_halfs")
                .long("halfs")
                .help("Only use unicode half blocks")
                .conflicts_with_all(&["all", "no_slopes", "only_blocks"])
                .requires_ifs(&[("draw_style", "block"), ("draw_style", "b")]),
        )
        .arg(
            Arg::with_name("width")
                .long("width")
                .short("w")
                .takes_value(true)
                .help("Override max display width in cells (maintains aspect ratio)"),
        )
        .arg(
            Arg::with_name("height")
                .long("height")
                .short("h")
                .takes_value(true)
                .help("Override max display height in cells (maintains aspect ratio)"),
        )
        .arg(
            Arg::with_name("still")
                .long("still")
                .short("s")
                .help("Don't animate images"),
        )
        .arg(
            Arg::with_name("draw_style")
                .long("mode")
                .short("m")
                .takes_value(true)
                .default_value("magic")
                .possible_values(&["block", "b", "dots", "d", "ascii", "a", "magic", "m"])
                .help("Display mode"),
        )
        .arg(
            Arg::with_name("file_names")
                .required(true)
                .multiple(true)
                .help("Input file name, - for stdin"),
        )
        .get_matches();

    let mut options = Options::new();

    options.truecolor = !matches.is_present("256_colors");
    options.file_names = matches
        .values_of("file_names")
        .map(|values| values.map(str::to_string).collect());
    options.blend = !matches.is_present("no_blending");
    options.isatty = termion::is_tty(&File::create("/dev/stdout").unwrap());
    options.animated = !matches.is_present("still");
    options.width = matches
        .value_of("width")
        .map(str::to_string)
        .and_then(|w| w.parse().ok());
    options.height = matches
        .value_of("height")
        .map(str::to_string)
        .and_then(|h| h.parse().ok());

    if matches.is_present("no_slopes") {
        options.char_set = CharSet::NoSlopes;
    } else if matches.is_present("only_blocks") {
        options.char_set = CharSet::Blocks;
    } else if matches.is_present("only_halfs") {
        options.char_set = CharSet::Halfs;
    }

    // Terminal checks
    if let Ok(prog) = env::var("TERM_PROGRAM") {
        if prog == "iTerm.app" {
            options.magic_type = Some(MagicType::Iterm);
        }
    }

    if let Ok(term) = env::var("TERM") {
        if term.contains("kitty") {
            options.magic_type = Some(MagicType::Kitty);
        }
    }

    // Terminal does not support magic, fallback
    if options.magic_type.is_none() {
        options.draw_style = DrawStyle::UnicodeBlock;
    }

    options.draw_style = match matches.value_of("draw_style").unwrap_or("") {
        "block" | "b" => DrawStyle::UnicodeBlock,
        "dots" | "d" => DrawStyle::Braille,
        "ascii" | "a" => DrawStyle::Ascii,
        "magic" | "m" => DrawStyle::Magic,
        s => panic!("Impossible draw style in match: {:?}", s),
    };

    // Check if truecolor support is reported
    if let Ok(colorterm) = env::var("COLORTERM") {
        if colorterm.to_ascii_lowercase() != "truecolor"
            && !matches.is_present("truecolor_override")
        {
            options.truecolor = false
        }
    } else if !matches.is_present("truecolor_override") {
        options.truecolor = false
    }

    options
}
