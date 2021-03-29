use super::{premultiply, resize_image, Rgb as TermRgb};
use image::{Delay, DynamicImage, Frames, GenericImageView, Rgba};
use itertools::{IntoChunks, Itertools};
use std::borrow::Cow;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Charset {
    /// Use all "fractional" block characters, most of the ["Box Drawing"](https://en.wikipedia.org/wiki/Box_Drawing_(Unicode_block))
    /// and ["Block Elements"](https://en.wikipedia.org/wiki/Block_Elements#Compact_table) characters,
    /// and "slope characters"
    All,
    /// Same as `Charset::All`, but without "slopes" because some fonts render them wide, i.e. `◢`, `◣`, `◤`, `◥`.
    NoSlopes,
    /// Use the full spectrum of "fractional" block characters, i.e. `▁▂▃▄▅▆▇`
    Blocks,
    /// Use only "half" block characters, i.e. `▄` and `▀`
    ///
    /// (Technically, the implementation only use the "Lower half block". The upper half is created
    /// by the background color)
    Halfs,
}

impl Charset {
    const fn bitmap(&self) -> &'static [(u32, char)] {
        match self {
            Charset::All => &bitmaps::ALL,
            Charset::Blocks => &bitmaps::BLOCKS,
            Charset::Halfs => &bitmaps::HALFS,
            Charset::NoSlopes => &bitmaps::NO_SLOPES,
        }
    }
}

impl Default for Charset {
    fn default() -> Charset {
        Charset::All
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cell {
    pub ch: char,
    pub fg: TermRgb,
    pub bg: TermRgb,
}

fn process_block(
    sub_img: &impl GenericImageView<Pixel = Rgba<u8>>,
    bitmaps: &[(u32, char)],
    blend: bool,
    bg_premultiply_color: image::Rgb<u8>,
) -> Cell {
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

    // Then use the median of the range to find the average of the forground and background
    // The median value is used to convert the 4x8 image to a bitmap
    let mut fg_count = 0;
    let mut bg_count = 0;
    let mut fg_color = [0u32; 3];
    let mut bg_color = [0u32; 3];
    let mut bits = 0u32;

    for y in 0..sub_img.height() {
        for x in 0..sub_img.width() {
            bits <<= 1;
            let pixel = sub_img.get_pixel(x, y);
            let pixel = premultiply(pixel, bg_premultiply_color);
            if pixel[split_index] > split_value {
                bits |= 1;
                fg_count += 1;
                for i in 0..3 {
                    fg_color[i] += u32::from(pixel[i]);
                }
            } else {
                bg_count += 1;
                for i in 0..3 {
                    bg_color[i] += u32::from(pixel[i]);
                }
            }
        }
    }

    // Get the averages
    for i in 0..3 {
        if fg_count != 0 {
            fg_color[i] /= fg_count;
        }

        if bg_count != 0 {
            bg_color[i] /= bg_count;
        }
    }

    // A perfect match is 0x0 so start at max
    let mut best_diff = 0xffff_ffffu32;
    let mut best_char = ' ';
    // The best match may be inverted
    let mut invert = false;

    // Determine the difference between the calculated bitmap and the character map
    for (bitmap, ch) in bitmaps.iter() {
        let diff = (bitmap ^ bits).count_ones();
        if diff < best_diff {
            best_diff = diff;
            best_char = *ch;
            invert = false
        }
        // Check the inverted bitmap
        let inverted_diff = (!bitmap ^ bits).count_ones();
        if inverted_diff < best_diff {
            best_diff = inverted_diff;
            best_char = *ch;
            invert = true;
        }
    }

    if blend {
        // If the bitmap does not fit "well", use a gradient
        if best_diff > 10 {
            invert = false;
            best_char = [' ', '\u{2591}', '\u{2592}', '\u{2593}', '\u{2588}']
                [4.min(fg_count as usize * 5 / 32)];
        }
    }

    // If best map is inverted, swap the colors
    if invert {
        std::mem::swap(&mut fg_color, &mut bg_color);
    }

    Cell {
        ch: best_char,
        fg: TermRgb((fg_color[0] as u8, fg_color[1] as u8, fg_color[2] as u8)),
        bg: TermRgb((bg_color[0] as u8, bg_color[1] as u8, bg_color[2] as u8)),
    }
}

// x and y are block coordinates, not pixel coordinates
fn process_at(
    x: u32,
    y: u32,
    img: &image::DynamicImage,
    bitmap: &[(u32, char)],
    blend: bool,
    background_color: image::Rgb<u8>,
) -> Cell {
    let sub_img = img.view(x * 4, y * 8, 4, 8);
    process_block(&sub_img, bitmap, blend, background_color)
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct BlockOptions {
    pub char_set: Charset,
    /// Whether or not to use "blending characters", i.e. `░ ▒ ▓ █`
    pub blend: bool,
    /// The color to use when premultiply alpha channels.
    ///
    /// This should be the color of whatever background the text will be displayed on
    pub background_color: image::Rgb<u8>,
    pub size: (u16, u16),
}

/// Render an image using [Unicode box-drawing characters](https://en.wikipedia.org/wiki/Box-drawing_character)
///
/// Example with `Charset::All`:  
/// <img src="https://i.imgur.com/6DFX97t.png" alt="Lichtenstein" width="50%"/>
#[derive(Debug, Copy, Clone)]
pub struct Block;

impl Block {
    /// Render animated image without resizing
    pub fn animated_exact<'a>(
        options: &BlockOptions,
        frames: Frames<'a>,
    ) -> impl Iterator<Item = (Delay, IntoChunks<CellIter<'a>>)> + 'a {
        let options = *options;
        frames.flatten().map(move |frame| {
            let delay = frame.delay();
            let img = DynamicImage::ImageRgba8(frame.into_buffer());

            (delay, Self::img_exact(&options, Cow::Owned(img)))
        })
    }

    /// Render animated image, resizing to nearest cell width
    pub fn animated<'a>(
        options: &BlockOptions,
        frames: Frames<'a>,
    ) -> impl Iterator<Item = (Delay, IntoChunks<CellIter<'a>>)> + 'a {
        let options = *options;
        frames.flatten().map(move |frame| {
            let delay = frame.delay();
            let img = DynamicImage::ImageRgba8(frame.into_buffer());

            let img = resize_image(&img, (4, 8), options.size);

            (delay, Self::img_exact(&options, Cow::Owned(img)))
        })
    }

    /// Render image without resizing
    pub fn img_exact<'a>(
        options: &BlockOptions,
        img: Cow<'a, DynamicImage>,
    ) -> IntoChunks<CellIter<'a>> {
        let block_width = img.width() / 4;
        CellIter {
            img,
            options: *options,
            i: 0,
        }
        .into_iter()
        .chunks((block_width) as usize)
    }

    /// Render image, resizing to nearest cell width
    pub fn img<'a>(options: &BlockOptions, img: &DynamicImage) -> IntoChunks<CellIter<'a>> {
        let img = resize_image(&img, (4, 8), options.size);
        let block_width = img.width() / 4;
        CellIter {
            img: Cow::Owned(img),
            options: *options,
            i: 0,
        }
        .into_iter()
        .chunks((block_width) as usize)
    }
}

pub struct CellIter<'a> {
    options: BlockOptions,
    img: Cow<'a, DynamicImage>,
    i: u32,
}

impl<'a> Iterator for CellIter<'a> {
    type Item = Cell;

    fn next(&mut self) -> Option<Self::Item> {
        let block_width = self.img.width() / 4;
        let block_height = self.img.height() / 8;

        if self.i >= (block_width * block_height) {
            return None;
        }

        let cell = process_at(
            self.i % block_width,
            self.i / block_width,
            &self.img,
            self.options.char_set.bitmap(),
            self.options.blend,
            self.options.background_color,
        );

        self.i += 1;
        Some(cell)
    }
}

/// This module contains the bitmaps for each character in a "charset"
///
/// The bitmaps represent the "dark" section of each character.
///
/// For example, given the character `┫`, the associated bitmap is `0x666ee666`, which is a 1d
/// representation of the 2d bitmap:
/// ```code
/// 0110
/// 0110
/// 0110
/// 1110
/// 1110
/// 0110
/// 0110
/// 0110
/// ```
///
/// This Python function can print the 1d bitmaps in their original 2d forms
/// ```python
/// def print_bitmap(bitmap: int):
///     for c in hex(bitmap)[2:]: print('{:04b}'.format(int(c,16)))
/// ```
pub mod bitmaps {
    #[allow(clippy::unreadable_literal)]
    pub const HALFS: [(u32, char); 2] = [(0x00000000, ' '), (0x0000ffff, '▄')];

    #[allow(clippy::unreadable_literal)]
    pub const BLOCKS: [(u32, char); 8] = [
        (0x00000000, ' '),
        (0x0000000f, '▁'),
        (0x000000ff, '▂'),
        (0x00000fff, '▃'),
        (0x0000ffff, '▄'),
        (0x000fffff, '▅'),
        (0x00ffffff, '▆'),
        (0x0fffffff, '▇'),
    ];

    #[allow(clippy::unreadable_literal)]
    pub const NO_SLOPES: [(u32, char); 51] = [
        (0x00000000, ' '),
        (0x0000000f, '▁'),
        (0x000000ff, '▂'),
        (0x00000fff, '▃'),
        (0x0000ffff, '▄'),
        (0x000fffff, '▅'),
        (0x00ffffff, '▆'),
        (0x0fffffff, '▇'),
        (0xeeeeeeee, '▊'),
        (0xcccccccc, '▌'),
        (0x88888888, '▎'),
        (0x0000cccc, '▖'),
        (0x00003333, '▗'),
        (0xcccc0000, '▘'),
        (0xcccc3333, '▚'),
        (0x33330000, '▝'),
        (0x000ff000, '━'),
        (0x66666666, '┃'),
        (0x00077666, '┏'),
        (0x000ee666, '┓'),
        (0x66677000, '┗'),
        (0x666ee000, '┛'),
        (0x66677666, '┣'),
        (0x666ee666, '┫'),
        (0x000ff666, '┳'),
        (0x666ff000, '┻'),
        (0x666ff666, '╋'),
        (0x000cc000, '╸'),
        (0x00066000, '╹'),
        (0x00033000, '╺'),
        (0x00066000, '╻'),
        (0x06600660, '╏'),
        (0x000f0000, '─'),
        (0x0000f000, '─'),
        (0x44444444, '│'),
        (0x22222222, '│'),
        (0x000e0000, '╴'),
        (0x0000e000, '╴'),
        (0x44440000, '╵'),
        (0x22220000, '╵'),
        (0x00030000, '╶'),
        (0x00003000, '╶'),
        (0x00004444, '╵'),
        (0x00002222, '╵'),
        (0x44444444, '⎢'),
        (0x22222222, '⎥'),
        (0x0f000000, '⎺'),
        (0x00f00000, '⎻'),
        (0x00000f00, '⎼'),
        (0x000000f0, '⎽'),
        (0x00066000, '▪'),
    ];

    #[allow(clippy::unreadable_literal)]
    pub const ALL: [(u32, char); 55] = [
        (0x00000000, ' '),
        (0x0000000f, '▁'),
        (0x000000ff, '▂'),
        (0x00000fff, '▃'),
        (0x0000ffff, '▄'),
        (0x000fffff, '▅'),
        (0x00ffffff, '▆'),
        (0x0fffffff, '▇'),
        (0xeeeeeeee, '▊'),
        (0xcccccccc, '▌'),
        (0x88888888, '▎'),
        (0x0000cccc, '▖'),
        (0x00003333, '▗'),
        (0xcccc0000, '▘'),
        (0xcccc3333, '▚'),
        (0x33330000, '▝'),
        (0x000ff000, '━'),
        (0x66666666, '┃'),
        (0x00077666, '┏'),
        (0x000ee666, '┓'),
        (0x66677000, '┗'),
        (0x666ee000, '┛'),
        (0x66677666, '┣'),
        (0x666ee666, '┫'),
        (0x000ff666, '┳'),
        (0x666ff000, '┻'),
        (0x666ff666, '╋'),
        (0x000cc000, '╸'),
        (0x00066000, '╹'),
        (0x00033000, '╺'),
        (0x00066000, '╻'),
        (0x06600660, '╏'),
        (0x000f0000, '─'),
        (0x0000f000, '─'),
        (0x44444444, '│'),
        (0x22222222, '│'),
        (0x000e0000, '╴'),
        (0x0000e000, '╴'),
        (0x44440000, '╵'),
        (0x22220000, '╵'),
        (0x00030000, '╶'),
        (0x00003000, '╶'),
        (0x00004444, '╵'),
        (0x00002222, '╵'),
        (0x44444444, '⎢'),
        (0x22222222, '⎥'),
        (0x0f000000, '⎺'),
        (0x00f00000, '⎻'),
        (0x00000f00, '⎼'),
        (0x000000f0, '⎽'),
        (0x00066000, '▪'),
        (0x000137f0, '\u{25e2}'), // Slopes
        (0x0008cef0, '\u{25e3}'),
        (0x000fec80, '\u{25e4}'),
        (0x000f7310, '\u{25e5}'),
    ];
}
