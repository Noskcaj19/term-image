use gif::{Decoder, Encoder};
use iterm2;
use std::fs::File;
use std::io::{self, Read};

use options::Options;

pub fn display(options: &Options, max_size: (u16, u16), path: &str) -> io::Result<()> {
    // TODO: Fix depending on extension
    if path.ends_with(".gif") && !options.animated {
        let f = File::open(path)?;

        let decoder = Decoder::new(f);

        let mut reader = decoder.read_info().unwrap();

        let first_frame = reader.read_next_frame().unwrap().unwrap().clone();

        let width = reader.width();
        let height = reader.height();

        let global_palette = reader.global_palette().unwrap();

        let mut buf = Vec::new();
        {
            let mut encoder = Encoder::new(&mut buf, width, height, global_palette).unwrap();
            encoder.write_frame(&first_frame)?;
        }

        iterm2::download_file(
            &[("inline", "1"), ("height", &max_size.1.to_string())],
            &buf,
        )
    } else {
        let img_data = if path == "-" {
            use std::io::{stdin, Read};
            let mut buf = Vec::new();
            stdin().read_to_end(&mut buf)?;
            buf
        } else {
            let mut f = File::open(path)?;
            let mut buf = Vec::new();
            f.read_to_end(&mut buf)?;
            buf
        };

        iterm2::download_file(
            &[("inline", "1"), ("height", &max_size.1.to_string())],
            &img_data,
        )
    }
}
