use std::thread;
use std::time::Duration;

use image::{DynamicImage, Frames, GenericImage, GenericImageView, Rgba};
use termion;
use termion::color::{self, Bg, Fg, Rgb};

use super::{draw_utils, DrawableCell};
use options::Options;
use renderer::CharSet;
use utils;

struct Block {
    ch: char,
    fg: Fg<Rgb>,
    bg: Bg<Rgb>,
}

impl DrawableCell for Block {
    fn print_truecolor(&self) {
        print!("{}{}{}", self.fg, self.bg, self.ch)
    }

    fn print_ansi(&self) {
        print!(
            "{}{}{}",
            Fg(draw_utils::rgb_to_ansi(self.fg.0)),
            Bg(draw_utils::rgb_to_ansi(self.bg.0)),
            self.ch
        )
    }
}

fn process_block(
    sub_img: &impl GenericImage<Pixel = Rgba<u8>>,
    bitmaps: &[(u32, char)],
    blend: bool,
) -> Block {
    // Determine the best color
    // First, determine the best color range
    let mut max = [0u8; 3];
    let mut min = [255u8; 3];
    for (_, _, p) in sub_img.pixels() {
        let p = draw_utils::premultiply(p);
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
            let pixel = draw_utils::premultiply(pixel);
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
        // If the bitmap does not fit "well", use a gradient,w
        if best_diff > 10 {
            invert = false;
            best_char = [' ', '\u{2591}', '\u{2592}', '\u{2593}', '\u{2588}']
                [4.min(fg_count as usize * 5 / 32)];
        }
    }

    // If best map is inverted, swap the colors
    if invert {
        ::std::mem::swap(&mut fg_color, &mut bg_color);
    }

    Block {
        ch: best_char,
        fg: Fg(Rgb(fg_color[0] as u8, fg_color[1] as u8, fg_color[2] as u8)),
        bg: Bg(Rgb(bg_color[0] as u8, bg_color[1] as u8, bg_color[2] as u8)),
    }
}

fn get_bitmap(char_set: CharSet) -> Vec<(u32, char)> {
    match char_set {
        CharSet::All => BITMAPS.to_vec(),
        CharSet::Blocks => BITMAPS_BLOCKS.to_vec(),
        CharSet::Halfs => BITMAPS_HALFS.to_vec(),
        CharSet::NoSlopes => BITMAPS_NO_SLOPES.to_vec(),
    }
}

pub fn print_image(options: &Options, max_size: (u16, u16), img: &DynamicImage) {
    let mut img = utils::resize_image(img, (4, 8), max_size);

    let bitmap = get_bitmap(options.char_set);

    for y in (0..img.height()).step_by(8) {
        for x in (0..img.width()).step_by(4) {
            let mut sub_img = img.sub_image(x, y, 4, 8);
            let block = process_block(&sub_img, &bitmap, options.blend);

            block.print(options.truecolor);
        }
        println!("{}{}", Fg(color::Reset), Bg(color::Reset));
    }
}

// TODO: Find a way to reduce duplication
pub fn print_frames(options: &Options, max_size: (u16, u16), frames: Frames) {
    let bitmap = get_bitmap(options.char_set);

    let mut frame_data = Vec::new();
    for frame in frames {
        let delay = u64::from(frame.delay().to_integer());
        let mut image = frame.into_buffer();
        let image = DynamicImage::ImageRgba8(image.clone());

        let mut image = utils::resize_image(&image, (4, 8), max_size);

        let mut img_data = Vec::new();

        for y in (0..image.height()).step_by(8) {
            let mut inner = Vec::new();
            for x in (0..image.width()).step_by(4) {
                let mut sub_img = image.sub_image(x, y, 4, 8);
                inner.push(process_block(&sub_img, &bitmap, options.blend));
            }
            img_data.push(inner);
        }

        frame_data.push((img_data, delay));
    }

    println!("{}{}", termion::clear::All, termion::cursor::Hide);

    use std::sync::atomic::Ordering;
    let term = utils::get_quit_hook();

    'gif: loop {
        for (frame, delay) in &frame_data {
            println!("{}", termion::cursor::Goto(1, 1));
            for line in frame {
                for block in line {
                    block.print(options.truecolor);
                }
                println!("{}{}", Fg(color::Reset), Bg(color::Reset));
            }
            thread::sleep(Duration::from_millis(*delay));
            if term.load(Ordering::Relaxed) {
                println!("{}", termion::cursor::Show);
                break 'gif;
            }
        }
    }
}

#[cfg_attr(feature = "cargo-clippy", allow(clippy::unreadable_literal))]
const BITMAPS_HALFS: [(u32, char); 2] = [(0x00000000, ' '), (0x0000ffff, '▄')];

#[cfg_attr(feature = "cargo-clippy", allow(clippy::unreadable_literal))]
const BITMAPS_BLOCKS: [(u32, char); 8] = [
    (0x00000000, ' '),
    (0x0000000f, '▁'),
    (0x000000ff, '▂'),
    (0x00000fff, '▃'),
    (0x0000ffff, '▄'),
    (0x000fffff, '▅'),
    (0x00ffffff, '▆'),
    (0x0fffffff, '▇'),
];

#[cfg_attr(feature = "cargo-clippy", allow(clippy::unreadable_literal))]
const BITMAPS_NO_SLOPES: [(u32, char); 51] = [
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

#[cfg_attr(feature = "cargo-clippy", allow(clippy::unreadable_literal))]
const BITMAPS: [(u32, char); 55] = [
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
    (0x000137f0, '\u{25e2}'), // Triangles
    (0x0008cef0, '\u{25e3}'),
    (0x000fec80, '\u{25e4}'),
    (0x000f7310, '\u{25e5}'),
];
