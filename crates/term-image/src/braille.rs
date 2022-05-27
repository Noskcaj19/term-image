use super::{premultiply, resize_image, Rgb as TermRgb};
use image::{
    imageops::colorops::{self, BiLevel},
    Delay, DynamicImage, GenericImageView, GrayImage, ImageBuffer, Luma, Rgb, Rgba,
};
use itertools::{IntoChunks, Itertools};
use std::borrow::Cow;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cell {
    pub ch: char,
    pub fg: TermRgb,
}

fn slice_to_braille(data: &[u8]) -> char {
    let mut v = 0;
    for i in &[0, 2, 4, 1, 3, 5, 6, 7] {
        v <<= 1;
        v |= data[*i as usize];
    }
    ::std::char::from_u32(0x2800 + u32::from(v)).unwrap()
}

fn process_cell(
    sub_img: &impl GenericImageView<Pixel = Rgba<u8>>,
    sub_mono_img: &impl GenericImageView<Pixel = Luma<u8>>,
    bg_premultiply_color: image::Rgb<u8>,
) -> Cell {
    let mut data = [0; 8];
    // Map each mono pixel to a single braille dot
    for (x, y, p) in sub_mono_img.pixels() {
        data[((y * 2) + x) as usize] = if p[0] == 0 { 0 } else { 1 }
    }

    // Determine the best color
    // First, determine the best color range
    let mut max = [0u8; 3];
    let mut min = [255u8; 3];
    for (_, _, p) in sub_img.pixels() {
        let p = premultiply(p, bg_premultiply_color);
        for i in 0..3 {
            max[i] = max[i].max(p[i]);
            min[i] = min[i].min(p[i]);
        }
    }

    let mut split_index = 0;
    let mut best_split = 0;
    for i in 0..3 {
        let diff = max[i] - min[i];
        if diff > best_split {
            best_split = diff;
            split_index = i
        }
    }

    let split_value = min[split_index] + best_split / 2;

    // Then use the median of the range to find the average of the forground
    let mut fg_count = 0;
    let mut fg_color = [0u32; 3];

    for y in 0..sub_img.height() {
        for x in 0..sub_img.width() {
            let pixel = sub_img.get_pixel(x, y);
            let pixel = premultiply(pixel, bg_premultiply_color);
            if pixel[split_index] > split_value {
                fg_count += 1;
                for i in 0..3 {
                    fg_color[i] += u32::from(pixel[i]);
                }
            }
        }
    }

    // Get the average
    for fg in &mut fg_color {
        if fg_count != 0 {
            *fg /= fg_count;
        }
    }

    Cell {
        ch: slice_to_braille(&data),
        fg: TermRgb((fg_color[0] as u8, fg_color[1] as u8, fg_color[2] as u8)),
    }
}

fn process_at(
    x: u32,
    y: u32,
    mono: &ImageBuffer<Luma<u8>, Vec<u8>>,
    img: &DynamicImage,
    background_color: Rgb<u8>,
) -> Cell {
    let sub_img = img.view(x * 2, y * 4, 2, 4);
    let sub_mono_img = mono.view(x * 2, y * 4, 2, 4);

    process_cell(&*sub_img, &*sub_mono_img, background_color)
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct BrailleOptions {
    pub size: (u16, u16),
    /// The color to use when premultiply alpha channels.
    ///
    /// This should be the color of whatever background the text will be displayed on
    pub background_color: Rgb<u8>,
}

/// Render an image using [Unicode Braille characters](https://en.wikipedia.org/wiki/Braille_Patterns#Block)
#[derive(Debug, Copy, Clone)]
pub struct Braille;

impl Braille {
    /// Render animated image without resizing
    pub fn animated_exact<'a>(
        options: &BrailleOptions,
        frames: image::Frames<'a>,
    ) -> impl Iterator<Item = (Delay, IntoChunks<impl Iterator<Item = Cell> + 'a>)> + 'a {
        let options = *options;
        frames.flatten().map(move |frame| {
            let delay = frame.delay();
            let img = DynamicImage::ImageRgba8(frame.into_buffer());

            (delay, Self::img_exact(&options, Cow::Owned(img)))
        })
    }

    /// Render animated image, resizing to nearest cell width
    pub fn animated<'a>(
        options: &BrailleOptions,
        frames: image::Frames<'a>,
    ) -> impl Iterator<Item = (Delay, IntoChunks<impl Iterator<Item = Cell> + 'a>)> + 'a {
        let options = *options;
        frames.flatten().map(move |frame| {
            let delay = frame.delay();
            let img = DynamicImage::ImageRgba8(frame.into_buffer());

            let img = resize_image(&img, (4, 8), options.size);

            (delay, Self::img(&options, &img))
        })
    }

    /// Render image without resizing
    pub fn img_exact<'a>(
        options: &BrailleOptions,
        img: Cow<'a, DynamicImage>,
    ) -> IntoChunks<CellIter<'a>> {
        let block_width = img.width() / 2;

        let mut mono = img.to_luma8();
        let map = BiLevel;

        colorops::dither(&mut mono, &map);

        CellIter {
            options: *options,
            img,
            mono,
            i: 0,
        }
        .chunks(block_width as usize)
    }

    /// Render image, resizing to nearest cell width
    pub fn img<'a>(
        options: &BrailleOptions,
        img: &DynamicImage,
    ) -> IntoChunks<impl Iterator<Item = Cell> + 'a> {
        let img = resize_image(img, (2, 4), options.size);
        let block_width = img.width() / 2;

        let mut mono = img.to_luma8();
        let map = BiLevel;

        colorops::dither(&mut mono, &map);

        CellIter {
            options: *options,
            img: Cow::Owned(img),
            mono,
            i: 0,
        }
        .chunks(block_width as usize)
    }
}

pub struct CellIter<'a> {
    options: BrailleOptions,
    img: Cow<'a, DynamicImage>,
    mono: GrayImage,
    i: u32,
}

impl<'a> Iterator for CellIter<'a> {
    type Item = Cell;

    fn next(&mut self) -> Option<Self::Item> {
        let block_width = self.img.width() / 2;
        let block_height = self.img.height() / 4;

        if self.i >= (block_width * block_height) {
            return None;
        }

        let cell = process_at(
            self.i % block_width,
            self.i / block_width,
            &self.mono,
            &self.img,
            self.options.background_color,
        );

        self.i += 1;
        Some(cell)
    }
}
