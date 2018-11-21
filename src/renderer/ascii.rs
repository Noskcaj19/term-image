// TODO: Improve image output quality?

use std::io::{stdout, Write};
use std::thread;
use std::time::Duration;

use image::{DynamicImage, FilterType, GenericImageView};
use termion;
use termion::color::{self, Bg, Fg, Rgb};

use super::{display, draw_utils, DrawableCell};
use options::Options;
use utils;

struct Block {
    ch: char,
    fg: Option<Fg<Rgb>>,
}

impl DrawableCell for Block {
    fn print_truecolor(&self, stdout: &mut impl Write) {
        if let Some(fg) = self.fg {
            let _ = write!(stdout, "{}{}", fg, self.ch);
        } else {
            let _ = write!(stdout, "{}", self.ch);
        }
    }

    fn print_ansi(&self, stdout: &mut impl Write) {
        if let Some(fg) = self.fg {
            let _ = write!(stdout, "{}{}", Fg(draw_utils::rgb_to_ansi(fg.0)), self.ch);
        } else {
            let _ = write!(stdout, "{}", self.ch);
        }
    }
}

fn best_char(brightness: u8, font: &[(char, u8)]) -> Block {
    let mut diff = 100;
    let mut cand = font[0].0;
    for x in font {
        if (i16::from(x.1) - i16::from(brightness)).abs() < diff {
            diff = (i16::from(x.1) - i16::from(brightness)).abs();
            cand = x.0;
        }
    }
    Block { ch: cand, fg: None }
}

pub struct Ascii;

impl display::TermDisplay for Ascii {
    // TODO: Find a way to reduce duplication
    fn animated(
        &self,
        options: &Options,
        term_size: (u16, u16),
        mut img_src: display::ImageSource,
    ) -> display::Result<()> {
        let stdout = stdout();
        let mut stdout = stdout.lock();
        let mut frame_data = Vec::new();
        for frame in img_src.frames().ok_or(())? {
            let delay = u64::from(frame.delay().to_integer());
            let mut image = frame.into_buffer();
            let image = DynamicImage::ImageRgba8(image.clone());

            // Keep aspect ratio, fit in terminal
            let image = image.resize(
                u32::from(term_size.0) / 2,
                u32::from(term_size.1),
                FilterType::Nearest,
            );

            // Stretch out horizontally so it looks decent
            let image = image.resize_exact(image.width() * 2, image.height(), FilterType::Nearest);

            let mono = image.to_luma();

            let mut img_data = Vec::new();

            for y in 0..mono.height() {
                let mut inner = Vec::new();
                for x in 0..mono.width() {
                    let mono_pixel = mono.get_pixel(x, y);

                    let mut block = best_char(mono_pixel[0], &FONT);

                    let pixel = image.get_pixel(x, y);
                    let pixel = draw_utils::premultiply(pixel);
                    block.fg = Some(Fg(Rgb(pixel[0], pixel[1], pixel[2])));

                    inner.push(block);
                }
                img_data.push(inner);
            }

            frame_data.push((img_data, delay));
        }

        let _ = writeln!(stdout, "{}{}", termion::clear::All, termion::cursor::Hide);

        use std::sync::atomic::Ordering;
        let term = utils::get_quit_hook();

        'gif: loop {
            for (frame, delay) in &frame_data {
                let _ = writeln!(stdout, "{}", termion::cursor::Goto(1, 1));
                for line in frame {
                    for block in line {
                        block.print(options.truecolor, &mut stdout);
                    }
                    let _ = writeln!(stdout, "{}{}", Fg(color::Reset), Bg(color::Reset));
                }
                thread::sleep(Duration::from_millis(*delay));
                if term.load(Ordering::Relaxed) {
                    let _ = writeln!(stdout, "{}", termion::cursor::Show);
                    break 'gif;
                }
            }
        }
        Ok(())
    }
    fn still(
        &self,
        options: &Options,
        term_size: (u16, u16),
        mut img_src: display::ImageSource,
    ) -> display::Result<()> {
        let stdout = stdout();
        let mut stdout = stdout.lock();
        // Keep aspect ratio, fit in terminal
        let img = img_src.image().ok_or(())?.resize(
            u32::from(term_size.0) / 2,
            u32::from(term_size.1),
            FilterType::Nearest,
        );

        // Stretch out horizontally so it looks decent
        let img = img.resize_exact(img.width() * 2, img.height(), FilterType::Nearest);

        let mono = img.to_luma();

        for y in 0..mono.height() {
            for x in 0..mono.width() {
                let mono_pixel = mono.get_pixel(x, y);
                let mut block = best_char(mono_pixel[0], &FONT);
                let pixel = img.get_pixel(x, y);
                let pixel = draw_utils::premultiply(pixel);
                block.fg = Some(Fg(Rgb(pixel[0], pixel[1], pixel[2])));

                block.print(options.truecolor, &mut stdout);
            }
            let _ = writeln!(stdout);
        }
        Ok(())
    }
}

// TODO: Remove dupes?
const FONT: [(char, u8); 94] = [
    // (' ', 0),
    ('`', 16),
    ('.', 22),
    ('\'', 26),
    ('_', 32),
    ('-', 36),
    (',', 40),
    (':', 46),
    ('"', 52),
    ('^', 56),
    ('~', 68),
    (';', 70),
    ('|', 72),
    ('(', 76),
    (')', 76),
    ('/', 78),
    ('\\', 78),
    ('j', 80),
    ('*', 82),
    ('!', 84),
    ('r', 84),
    ('+', 88),
    ('[', 88),
    (']', 88),
    ('i', 88),
    ('<', 92),
    ('>', 92),
    ('=', 96),
    ('?', 100),
    ('l', 100),
    ('{', 100),
    ('}', 100),
    ('c', 102),
    ('v', 108),
    ('t', 112),
    ('z', 112),
    ('7', 114),
    ('L', 114),
    ('f', 114),
    ('x', 116),
    ('s', 118),
    ('Y', 122),
    ('J', 124),
    ('T', 124),
    ('1', 128),
    ('n', 128),
    ('u', 128),
    ('C', 130),
    ('y', 136),
    ('I', 138),
    ('F', 140),
    ('o', 140),
    ('2', 144),
    ('V', 148),
    ('e', 148),
    ('w', 148),
    ('%', 150),
    ('3', 150),
    ('h', 150),
    ('k', 150),
    ('a', 152),
    ('4', 156),
    ('Z', 156),
    ('5', 158),
    ('S', 158),
    ('X', 158),
    ('P', 166),
    ('$', 168),
    ('b', 170),
    ('d', 170),
    ('m', 170),
    ('p', 170),
    ('q', 170),
    ('A', 172),
    ('G', 172),
    ('E', 174),
    ('U', 174),
    ('&', 182),
    ('6', 182),
    ('K', 182),
    ('9', 184),
    ('g', 184),
    ('O', 186),
    ('H', 188),
    ('#', 190),
    ('Q', 190),
    ('D', 192),
    ('@', 194),
    ('8', 198),
    ('R', 198),
    ('0', 210),
    ('W', 212),
    ('N', 216),
    ('B', 218),
    ('M', 218),
];
