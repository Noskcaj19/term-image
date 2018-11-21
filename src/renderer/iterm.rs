use gif::{Decoder, Encoder};
use iterm2;
use std::io::Read;

use options::Options;

use super::display;

pub struct Iterm;

impl display::TermDisplay for Iterm {
    fn animated(
        &self,
        _options: &Options,
        term_size: (u16, u16),
        mut img_src: display::ImageSource,
    ) -> display::Result<()> {
        let mut img_data = Vec::new();
        img_src.data().read_to_end(&mut img_data);
        iterm2::download_file(
            &[("inline", "1"), ("height", &term_size.1.to_string())],
            &img_data,
        )
        .map_err(|_| ())
    }

    fn still(
        &self,
        _options: &Options,
        term_size: (u16, u16),
        mut img_src: display::ImageSource,
    ) -> display::Result<()> {
        if img_src.path().ends_with(".gif") {
            let decoder = Decoder::new(img_src.data());

            let mut reader = decoder.read_info().unwrap();

            let first_frame = reader.read_next_frame().unwrap().unwrap().clone();

            let width = reader.width();
            let height = reader.height();

            let global_palette = reader.global_palette().unwrap();

            let mut buf = Vec::new();
            {
                let mut encoder = Encoder::new(&mut buf, width, height, global_palette).unwrap();
                encoder.write_frame(&first_frame).map_err(|_| ())?;
            }

            iterm2::download_file(
                &[("inline", "1"), ("height", &term_size.1.to_string())],
                &buf,
            )
            .map_err(|_| ())
        } else {
            let mut img_data = Vec::new();
            img_src.data().read_to_end(&mut img_data);
            iterm2::download_file(
                &[("inline", "1"), ("height", &term_size.1.to_string())],
                &img_data,
            )
            .map_err(|_| ())
        }
    }
}
