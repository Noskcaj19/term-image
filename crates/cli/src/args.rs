use clap::{Command, Arg};
use crossterm::tty::IsTty;
use image::Rgb;
use std::{env, io::stdout};
use term_image::{
    ascii::AsciiOptions, block::BlockOptions, block::Charset, braille::BrailleOptions,
    iterm::ItermOptions, kitty::KittyOptions,
};

#[derive(Debug)]
pub struct Options {
    pub path: String,
    pub truecolor: bool,
    pub renderer_options: RendererOption,
    pub still: bool,
}

#[derive(Debug)]
pub enum RendererOption {
    Block(BlockOptions),
    Ascii(AsciiOptions),
    Braille(BrailleOptions),
    Kitty(KittyOptions),
    Iterm(ItermOptions),
}

/// Parse cli args into Options
pub fn get_options() -> Options {
    let matches = Command::new("Terminal Image Viewer")
        .author("Noskcaj")
        .about("Shows images in your terminal")
        .arg(
            Arg::new("256_colors")
                .long("ansi")
                .visible_alias("256")
                .short('a')
                .help("Use only ansi 256 colors"),
        )
        .arg(
            Arg::new("force_truecolor")
                .long("truecolor")
                .short('t')
                .help("Force truecolor even in unsupported terminals"),
        )
        .arg(
            Arg::new("no_blending")
                .long("noblend")
                .short('b')
                .help("Disable blending characters"),
        )
        .arg(
            Arg::new("all")
                .long("all")
                .help("Use all unicode drawing characters")
                .conflicts_with_all(&["no_slopes", "only_blocks", "only_halfs"])
                .requires_ifs(&[("block", "renderer"), ("b", "renderer")]),
        )
        .arg(
            Arg::new("no_slopes")
                .long("no-slopes")
                .help("Disable sloped unicode characters (if they are wide in your font)")
                .conflicts_with_all(&["all", "only_blocks", "only_halfs"])
                .requires_ifs(&[("block", "renderer"), ("b", "renderer")]),
        )
        .arg(
            Arg::new("only_blocks")
                .long("blocks")
                .help("Only use unicode fractional block characters")
                .conflicts_with_all(&["all", "no_slopes", "only_halfs"])
                .requires_ifs(&[("block", "renderer"), ("b", "renderer")]),
        )
        .arg(
            Arg::new("only_halfs")
                .long("halfs")
                .help("Only use unicode half blocks")
                .conflicts_with_all(&["all", "no_slopes", "only_blocks"])
                .requires_ifs(&[("block", "renderer"), ("b", "renderer")]),
        )
        .arg(
            Arg::new("width")
                .long("width")
                .short('w')
                .takes_value(true)
                .help("Override max display width in cells (maintains aspect ratio)"),
        )
        .arg(
            Arg::new("height")
                .long("height")
                .short('h')
                .takes_value(true)
                .help("Override max display height in cells (maintains aspect ratio)"),
        )
        .arg(
            Arg::new("still")
                .long("still")
                .short('s')
                .help("Don't animate images"),
        )
        .arg(
            Arg::new("background_color")
                .long("bg")
                .takes_value(true)
                .help("Comma seperated rgb value to use when rendering transparency")
                .validator(validate_rgb_triplet),
        )
        .arg(
            Arg::new("renderer")
                .short('r')
                .long("renderer")
                .takes_value(true)
                .default_value("terminal")
                .possible_values(&[
                    "block", "b", "dots", "d", "ascii", "a", "kitty", "k", "iterm", "i",
                    "terminal", "t",
                ])
                .help("Renderer to use"),
        )
        .arg(
            Arg::new("file_name")
                .required(true)
                .help("Input file name, - for stdin"),
        )
        .get_matches();

    let char_set = if matches.is_present("no_slopes") {
        Charset::NoSlopes
    } else if matches.is_present("only_blocks") {
        Charset::Blocks
    } else if matches.is_present("only_halfs") {
        Charset::Halfs
    } else {
        Charset::default()
    };

    let iterm = env::var("TERM_PROGRAM")
        .map(|prog| prog == "iTerm.app")
        .unwrap_or(false);
    let kitty = env::var("TERM")
        .map(|term| term.contains("kitty"))
        .unwrap_or(false);

    let maybe_kitty_renderer = matches
        .value_of("renderer")
        .map(|r| matches!(r, "t" | "terminal" | "k" | "kitty"))
        .unwrap_or(true);

    let maybe_iterm_renderer = matches
        .value_of("renderer")
        .map(|r| matches!(r, "t" | "terminal" | "i" | "iterm"))
        .unwrap_or(true);

    // multiply default or terminal size by this value to make kitty renderer defaults more reasonable
    let (width_multiplier, height_multiplier) = if kitty && maybe_kitty_renderer {
        (12, 24)
    } else {
        (1, 1)
    };

    let tty = stdout().is_tty();
    let optional_term_size = match (
        matches.value_of_t::<u16>("width"),
        matches.value_of_t::<u16>("height"),
    ) {
        (width, height) if width.is_ok() || height.is_ok() => (width.ok(), height.ok()),
        (_, _) if iterm && maybe_iterm_renderer => (None, None),
        (_, _) if !tty => (Some(80 * width_multiplier), Some(25 * height_multiplier)),
        _ => crossterm::terminal::size()
            .map(|(w, h)| {
                (
                    Some((w - 4) * width_multiplier),
                    Some((h - 8) * height_multiplier),
                )
            })
            .unwrap_or((Some(80 * width_multiplier), Some(25 * height_multiplier))),
    };

    let term_size = (
        optional_term_size.0.unwrap_or(u16::MAX),
        optional_term_size.1.unwrap_or(u16::MAX),
    );

    let background_color = matches
        .value_of("background_color")
        .and_then(parse_rgb_triplet)
        .unwrap_or_else(|| [0, 0, 0].into());

    let renderer_options = match matches.value_of("renderer").unwrap_or("t") {
        "block" | "b" => RendererOption::Block(BlockOptions {
            char_set,
            blend: !matches.is_present("no_blending"),
            background_color,
            size: term_size,
        }),
        "dots" | "d" => RendererOption::Braille(BrailleOptions {
            size: term_size,
            background_color,
        }),
        "ascii" | "a" => RendererOption::Ascii(AsciiOptions {
            size: term_size,
            background_color,
        }),
        "kitty" | "k" => RendererOption::Kitty(KittyOptions { size: term_size }),
        "iterm" | "i" => RendererOption::Iterm(ItermOptions {
            size: optional_term_size,
        }),
        "terminal" | "t" => {
            if kitty {
                RendererOption::Kitty(KittyOptions { size: term_size })
            } else if iterm {
                RendererOption::Iterm(ItermOptions {
                    size: optional_term_size,
                })
            } else {
                RendererOption::Block(BlockOptions {
                    char_set,
                    blend: !matches.is_present("no_blending"),
                    background_color,
                    size: term_size,
                })
            }
        }
        _ => unreachable!(),
    };

    let truecolor = !matches.is_present("256_colors")
        && env::var("COLORTERM")
            .map(|c| c.to_ascii_lowercase() == "truecolor")
            .unwrap_or(false);

    Options {
        path: matches
            .value_of("file_name")
            .expect("required by clap")
            .into(),
        truecolor,
        renderer_options,
        still: matches.is_present("still"),
    }
}

fn parse_rgb_triplet(v: &str) -> Option<Rgb<u8>> {
    let mut parts = v.split(',').map(str::parse).flatten();

    Some([parts.next()?, parts.next()?, parts.next()?].into())
}

fn validate_rgb_triplet(v: &str) -> Result<Rgb<u8>, String> {
    parse_rgb_triplet(v).ok_or_else(|| "background color not in R,G,B format".into())
}
