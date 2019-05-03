mod ascii;
mod braille;
mod iterm;
mod kitty;
mod unicode_block;

mod display;
mod draw_utils;

use crate::options::Options;

use std::io::Write;

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
    Kitty,
    Iterm,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrawStyle {
    UnicodeBlock,
    Braille,
    Ascii,
    Magic,
}

trait DrawableCell {
    fn print(&self, truecolor: bool, stdout: &mut impl Write) {
        if truecolor {
            self.print_truecolor(stdout);
        } else {
            self.print_ansi(stdout);
        }
    }

    fn print_truecolor(&self, stdout: &mut impl Write);
    fn print_ansi(&self, stdout: &mut impl Write);
}

pub fn render_image(options: &Options, term_size: (u16, u16)) {
    let file_names = &options.file_names.clone().unwrap();
    for file_name in file_names {
        let display: Box<display::TermDisplay> = match options.draw_style {
            DrawStyle::Ascii => Box::new(ascii::Ascii),
            DrawStyle::Braille => Box::new(braille::Braille),
            DrawStyle::UnicodeBlock => Box::new(unicode_block::UnicodeBlock),
            DrawStyle::Magic => match options.magic_type {
                Some(MagicType::Iterm) => Box::new(iterm::Iterm),
                Some(MagicType::Kitty) => Box::new(kitty::Kitty),
                None => {
                    eprintln!("No known magic display modes");
                    continue;
                }
            },
        };

        let img_src = self::display::ImageSource::new(file_name);
        if file_name.ends_with(".gif") && options.animated {
            let _ = display.animated(&options, term_size, img_src);
        } else {
            let _ = display.still(&options, term_size, img_src);
        }

        if file_names.len() > 1 {
            println!();
        }
    }
}
