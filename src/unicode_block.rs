use image::{DynamicImage, FilterType, GenericImage, Rgba};
use options::Options;
use std::fmt;
use termion::color::{self, Bg, Fg, Rgb};

use utils;

#[derive(Debug, Clone, Copy)]
pub enum DrawMode {
    All,
    NoSlopes,
    Blocks,
    Halfs,
}

impl Default for DrawMode {
    fn default() -> DrawMode {
        DrawMode::All
    }
}

fn color_to_8bit(color: color::Rgb) -> color::AnsiValue {
    let r = (u16::from(color.0) * 5 / 255) as u8;
    let g = (u16::from(color.1) * 5 / 255) as u8;
    let b = (u16::from(color.2) * 5 / 255) as u8;
    color::AnsiValue(16 + 36 * r + 6 * g + b)
}

struct Block {
    ch: char,
    truecolor: bool,
    fg: Fg<Rgb>,
    bg: Bg<Rgb>,
}

impl fmt::Debug for Block {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.truecolor {
            write!(f, "{}{}{}", self.fg, self.bg, self.ch)
        } else {
            write!(
                f,
                "{}{}{}",
                Fg(color_to_8bit(self.fg.0)),
                Bg(color_to_8bit(self.bg.0)),
                self.ch
            )
        }
    }
}

fn process_block(
    sub_img: &impl GenericImage<Pixel = Rgba<u8>>,
    bitmaps: &[(u32, char)],
    use_blend: bool,
    truecolor: bool,
) -> Block {
    let mut max = [0u8; 3];
    let mut min = [255u8; 3];
    for (_, _, p) in sub_img.pixels() {
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

    let mut fg_count = 0;
    let mut bg_count = 0;
    let mut fg_color = [0u32; 3];
    let mut bg_color = [0u32; 3];
    let mut bits = 0u32;

    for y in 0..sub_img.height() {
        for x in 0..sub_img.width() {
            bits <<= 1;
            let pixel = sub_img.get_pixel(x, y);
            if pixel[split_index] > split_value {
                bits |= 1;
                fg_count += 1;
                for i in 0..3 {
                    fg_color[i] += pixel[i] as u32;
                }
            } else {
                bg_count += 1;
                for i in 0..3 {
                    bg_color[i] += pixel[i] as u32;
                }
            }
        }
    }

    for i in 0..3 {
        if fg_count != 0 {
            fg_color[i] /= fg_count;
        }

        if bg_count != 0 {
            bg_color[i] /= bg_count;
        }
    }

    let mut best_diff = 0xffffffffu32;
    let mut best_char = ' ';
    let mut invert = false;

    for (bitmap, char) in bitmaps.iter() {
        let diff = (bitmap ^ bits).count_ones();
        if diff < best_diff {
            best_diff = diff;
            best_char = *char;
            invert = false
        }
        let inverted_diff = (!bitmap ^ bits).count_ones();
        if inverted_diff < best_diff {
            best_diff = inverted_diff;
            best_char = *char;
            invert = true;
        }
    }

    if use_blend {
        if best_diff > 10 {
            invert = false;
            best_char = [' ', '\u{2591}', '\u{2592}', '\u{2593}', '\u{2588}']
                [4.min(fg_count as usize * 5 / 32)];
        }
    }

    if invert {
        ::std::mem::swap(&mut fg_color, &mut bg_color);
    }

    Block {
        ch: best_char,
        truecolor,
        fg: Fg(Rgb(fg_color[0] as u8, fg_color[1] as u8, fg_color[2] as u8)),
        bg: Bg(Rgb(bg_color[0] as u8, bg_color[1] as u8, bg_color[2] as u8)),
    }
}

pub fn print_image(options: &Options, img: DynamicImage) {
    let mut img = img.resize_exact(
        utils::closest_mult(img.width(), 4),
        utils::closest_mult(img.height(), 8),
        FilterType::Nearest,
    );

    let bitmap = match options.draw_mode {
        DrawMode::All => BITMAPS.to_vec(),
        DrawMode::Blocks => BITMAPS_BLOCKS.to_vec(),
        DrawMode::Halfs => BITMAPS_HALFS.to_vec(),
        DrawMode::NoSlopes => BITMAPS_NO_SLOPES.to_vec(),
    };

    for y in (0..img.height()).step_by(8) {
        for x in (0..img.width()).step_by(4) {
            let sub_img = img.sub_image(x, y, 4, 8);
            print!(
                "{:?}",
                process_block(&sub_img, &bitmap, options.blend, !options.ansi_256_color)
            );
        }
        println!("{}{}", Fg(color::Reset), Bg(color::Reset));
    }
}

const BITMAPS_HALFS: [(u32, char); 2] = [(0x00000000, ' '), (0x0000ffff, '▄')];

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
