use options::Options;
use utils;

use image::imageops::colorops::{self, BiLevel};
use image::{DynamicImage, Frames, GenericImage, Luma, Rgba};
use std::fmt;
use std::thread;
use std::time::Duration;
use termion;
use termion::color::{self, Bg, Fg, Rgb};

struct Block {
    ch: char,
    fg: Fg<Rgb>,
}

impl fmt::Debug for Block {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if f.alternate() {
            write!(f, "{}{}", Fg(utils::rgb_to_ansi(self.fg.0)), self.ch)
        } else {
            write!(f, "{}{}", self.fg, self.ch)
        }
    }
}

fn slice_to_braille(data: &[u8]) -> char {
    let mut v = 0;
    for i in &[0, 2, 4, 1, 3, 5, 6, 7] {
        v <<= 1;
        v |= data[*i as usize];
    }
    ::std::char::from_u32(0x2800 + v as u32).unwrap()
}

fn process_block(
    sub_img: &impl GenericImage<Pixel = Rgba<u8>>,
    sub_mono_img: &impl GenericImage<Pixel = Luma<u8>>,
) -> Block {
    let mut data = [0; 8];
    for (x, y, p) in sub_mono_img.pixels() {
        data[((y * 2) + x) as usize] = if p[0] == 0 { 0 } else { 1 }
    }

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
    let mut fg_color = [0u32; 3];

    for y in 0..sub_img.height() {
        for x in 0..sub_img.width() {
            let pixel = sub_img.get_pixel(x, y);
            if pixel[split_index] > split_value {
                fg_count += 1;
                for i in 0..3 {
                    fg_color[i] += pixel[i] as u32;
                }
            }
        }
    }

    for i in 0..3 {
        if fg_count != 0 {
            fg_color[i] /= fg_count;
        }
    }

    Block {
        ch: slice_to_braille(&data),
        fg: Fg(Rgb(fg_color[0] as u8, fg_color[1] as u8, fg_color[2] as u8)),
    }
}

pub fn display(options: &Options, max_size: (u16, u16), img: &DynamicImage) {
    let mut img = utils::resize_image(img, 2, 4, max_size);

    let mut mono = img.to_luma();
    let map = BiLevel;

    colorops::dither(&mut mono, &map);

    for y in (0..img.height()).step_by(4) {
        for x in (0..img.width()).step_by(2) {
            let sub_img = img.sub_image(x, y, 2, 4);
            let sub_mono_img = mono.sub_image(x, y, 2, 4);

            let block = process_block(&sub_img, &sub_mono_img);
            if options.ansi_256_color {
                print!("{:#?}", block);
            } else {
                print!("{:?}", block);
            }
        }
        println!();
    }
}

pub fn print_frames(options: &Options, max_size: (u16, u16), frames: Frames) {
    let mut frame_data = Vec::new();
    for frame in frames {
        let delay = frame.delay().to_integer() as u64;
        let mut image = frame.into_buffer();
        let image = DynamicImage::ImageRgba8(image.clone());

        let mut image = utils::resize_image(&image, 2, 4, max_size);

        let mut mono = image.to_luma();
        let map = BiLevel;

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

    println!("{}{}", termion::clear::All, termion::cursor::Hide);

    use libc;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    let term = Arc::new(AtomicBool::new(false));
    for signal in &[libc::SIGINT, libc::SIGQUIT, libc::SIGTERM] {
        ::signal_hook::flag::register(*signal, Arc::clone(&term)).expect("Unable to hook SIGINT");
    }

    'gif: loop {
        for (frame, delay) in &frame_data {
            println!("{}", termion::cursor::Goto(1, 1));
            for line in frame {
                for block in line {
                    if options.ansi_256_color {
                        print!("{:#?}", block);
                    } else {
                        print!("{:?}", block);
                    }
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
