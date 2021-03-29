// TODO: Improve image output quality?
use super::{premultiply, resize_image, Rgb as TermRgb};
use image::{
    imageops::FilterType, Delay, DynamicImage, Frames, GenericImageView, GrayImage, ImageBuffer,
    Luma, Rgb,
};
use itertools::{IntoChunks, Itertools};
use std::borrow::Cow;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cell {
    pub ch: char,
    pub fg: TermRgb,
}

fn best_char(brightness: u8, font: &[(char, u8)]) -> char {
    let mut diff = 100;
    let mut cand = font[0].0;
    for x in font {
        if (i16::from(x.1) - i16::from(brightness)).abs() < diff {
            diff = (i16::from(x.1) - i16::from(brightness)).abs();
            cand = x.0;
        }
    }
    cand
}

fn process_at(
    x: u32,
    y: u32,
    mono: &ImageBuffer<Luma<u8>, Vec<u8>>,
    img: &DynamicImage,
    background_color: image::Rgb<u8>,
) -> Cell {
    let mono_pixel = mono.get_pixel(x, y);
    let char = best_char(mono_pixel[0], &FONT);
    let pixel = img.get_pixel(x, y);
    let pixel = premultiply(pixel, background_color);
    Cell {
        ch: char,
        fg: TermRgb((pixel[0], pixel[1], pixel[2])),
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct AsciiOptions {
    pub size: (u16, u16),
    /// The color to use when premultiply alpha channels.
    ///
    /// This should be the color of whatever background the text will be displayed on
    pub background_color: Rgb<u8>,
}

/// Render an image using only ASCII characters
#[derive(Debug, Copy, Clone)]
pub struct Ascii;

impl Ascii {
    pub fn animated_exact<'a>(
        options: &AsciiOptions,
        frames: Frames<'a>,
    ) -> impl Iterator<Item = (Delay, IntoChunks<CellIter<'a>>)> + 'a {
        let options = *options;
        frames.flatten().map(move |frame| {
            let delay = frame.delay();
            let img = DynamicImage::ImageRgba8(frame.into_buffer());

            (delay, Self::img_exact(&options, Cow::Owned(img)))
        })
    }

    pub fn animated<'a>(
        options: &AsciiOptions,
        frames: Frames<'a>,
    ) -> impl Iterator<Item = (Delay, IntoChunks<CellIter<'a>>)> + 'a {
        let options = *options;
        frames.flatten().map(move |frame| {
            let delay = frame.delay();
            let img = DynamicImage::ImageRgba8(frame.into_buffer());

            let img = resize_image(&img, (1, 1), options.size);

            (delay, Self::img(&options, &img))
        })
    }

    pub fn img_exact<'a>(
        options: &AsciiOptions,
        img: Cow<'a, DynamicImage>,
    ) -> IntoChunks<CellIter<'a>> {
        let mono = img.to_luma8();

        let width = mono.width();
        CellIter {
            options: *options,
            img,
            mono,
            i: 0,
        }
        .into_iter()
        .chunks(width as usize)
    }

    pub fn img<'a>(options: &AsciiOptions, img: &DynamicImage) -> IntoChunks<CellIter<'a>> {
        // Keep aspect ratio, fit in terminal
        let img = img.resize(
            u32::from(options.size.0) / 2,
            u32::from(options.size.1),
            FilterType::Nearest,
        );

        // Stretch out horizontally so it looks decent
        let img = img.resize_exact(img.width() * 2, img.height(), FilterType::Nearest);

        let mono = img.to_luma8();

        let width = mono.width();

        CellIter {
            options: *options,
            img: Cow::Owned(img),
            mono,
            i: 0,
        }
        .into_iter()
        .chunks(width as usize)
    }
}

pub struct CellIter<'a> {
    options: AsciiOptions,
    img: Cow<'a, DynamicImage>,
    mono: GrayImage,
    i: u32,
}

impl<'a> Iterator for CellIter<'a> {
    type Item = Cell;

    fn next(&mut self) -> Option<Self::Item> {
        if self.i >= (self.img.width() * self.img.height()) {
            return None;
        }

        let cell = process_at(
            self.i % self.img.width(),
            self.i / self.img.width(),
            &self.mono,
            &self.img,
            self.options.background_color,
        );

        self.i += 1;
        Some(cell)
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
