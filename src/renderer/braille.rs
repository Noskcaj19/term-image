use std::io::{stdout, Write};
use std::thread;
use std::time::Duration;

use image::imageops::colorops::{self, BiLevel};
use image::{DynamicImage, GenericImage, GenericImageView, Luma, Rgba};
use termion;
use termion::color::{self, Bg, Fg, Rgb};

use super::{display, draw_utils, DrawableCell};
use options::Options;
use utils;

struct Block {
    ch: char,
    fg: Fg<Rgb>,
}

impl DrawableCell for Block {
    fn print_truecolor(&self, stdout: &mut impl Write) {
        let _ = write!(stdout, "{}{}", self.fg, self.ch);
    }

    fn print_ansi(&self, stdout: &mut impl Write) {
        let _ = write!(
            stdout,
            "{}{}",
            Fg(draw_utils::rgb_to_ansi(self.fg.0)),
            self.ch
        );
    }
}

fn slice_to_braille(data: &[u8]) -> char {
    let mut v = 0;
    for i in &[0, 2, 4, 1, 3, 5, 6, 7] {
        v <<= 1;
        v |= data[*i as usize];
    }
    ::std::char::from_u32(0x2800 + u32::from(v)).unwrap()
}

fn process_block(
    sub_img: &impl GenericImage<Pixel = Rgba<u8>>,
    sub_mono_img: &impl GenericImage<Pixel = Luma<u8>>,
) -> Block {
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

    // Then use the median of the range to find the average of the forground
    let mut fg_count = 0;
    let mut fg_color = [0u32; 3];

    for y in 0..sub_img.height() {
        for x in 0..sub_img.width() {
            let pixel = sub_img.get_pixel(x, y);
            let pixel = draw_utils::premultiply(pixel);
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

    Block {
        ch: slice_to_braille(&data),
        fg: Fg(Rgb(fg_color[0] as u8, fg_color[1] as u8, fg_color[2] as u8)),
    }
}

pub struct Braille;

impl display::TermDisplay for Braille {
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

            let mut image = utils::resize_image(&image, (2, 4), term_size);

            let mut mono = image.to_luma();
            let map = BiLevel;

            // Dither with Floyd-Steinberg
            colorops::dither(&mut mono, &map);

            let mut img_data = Vec::new();

            for y in (0..image.height()).step_by(4) {
                let mut inner = Vec::new();
                for x in (0..image.width()).step_by(2) {
                    let sub_img = image.sub_image(x, y, 2, 4);
                    let sub_mono_img = mono.sub_image(x, y, 2, 4);
                    inner.push(process_block(&sub_img, &sub_mono_img));
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
        let mut img = utils::resize_image(img_src.image().ok_or(())?, (2, 4), term_size);

        let mut mono = img.to_luma();
        let map = BiLevel;

        colorops::dither(&mut mono, &map);

        for y in (0..img.height()).step_by(4) {
            for x in (0..img.width()).step_by(2) {
                // `sub_image()` is a cheap reference
                let sub_img = img.sub_image(x, y, 2, 4);
                let sub_mono_img = mono.sub_image(x, y, 2, 4);

                let block = process_block(&sub_img, &sub_mono_img);
                block.print(options.truecolor, &mut stdout);
            }
            writeln!(stdout);
        }
        Ok(())
    }
}
