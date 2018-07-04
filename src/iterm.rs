use gif::{Decoder, Encoder};
use image::ImageFormat;
use iterm2;
use std::fs::File;
use std::io::{self, Read};

use Options;

pub fn display(options: &Options, max_size: (u16, u16), path: &str) -> io::Result<()> {
    match options.image_format {
        Some(ImageFormat::GIF) if !options.animated => {
            let f = File::open(path)?;

            let decoder = Decoder::new(f);

            let mut reader = decoder.read_info().unwrap();

            let first_frame = reader.read_next_frame().unwrap().unwrap().clone();

            let width = reader.width();
            let height = reader.height();
            let global_palette = &(*reader.global_palette().clone().unwrap().clone());

            let mut buf = Vec::new();
            {
                let mut encoder = Encoder::new(&mut buf, width, height, global_palette).unwrap();
                encoder.write_frame(&first_frame)?;
            }

            iterm2::download_file(
                &[("inline", "1"), ("height", &max_size.1.to_string())],
                &buf,
            )
        }
        _ => {
            let mut f = File::open(path)?;
            let mut img_data = Vec::new();
            f.read_to_end(&mut img_data)?;

            iterm2::download_file(
                &[("inline", "1"), ("height", &max_size.1.to_string())],
                &img_data,
            )
        }
    }
}
