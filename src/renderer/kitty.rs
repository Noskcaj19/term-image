use std::io::{self, stdin, stdout, Read, Write};

use base64;
use image::{DynamicImage, GenericImageView};
use termion::raw::IntoRawMode;

use super::display;
use crate::options::Options;

const PROTOCOL_START: &[u8] = b"\x1b_G";
const PROTOCOL_END: &[u8] = b"\x1b\\";
const MAX_BUFFER: usize = 2048;

fn print_cmd_payload(cmds: &[(&str, &str)], payload: &str) -> io::Result<()> {
    let cmds = cmds
        .iter()
        .map(|(l, r)| format!("{}={}", l, r))
        .collect::<Vec<_>>()
        .join(",");

    let stdout = stdout();
    let mut stdout = stdout.lock();

    let mut payload = payload.as_bytes();
    while payload.len() > MAX_BUFFER {
        let (buf, new_payload) = payload.split_at(MAX_BUFFER);
        payload = new_payload;
        stdout.write_all(PROTOCOL_START)?;
        write!(stdout, "{},m=1;", cmds)?;
        stdout.write_all(&buf)?;
        stdout.write_all(PROTOCOL_END)?;
    }
    stdout.write_all(PROTOCOL_START)?;
    write!(stdout, "{},m=0;", cmds)?;
    stdout.write_all(&payload)?;
    stdout.write_all(PROTOCOL_END)?;

    stdout.flush()?;
    Ok(())
}

/// Display an image from an canonical path
///
/// # Notes
/// Path _must_ be canonical, use std::fs::canonicalize
#[allow(dead_code)]
fn display_path(path: &str) -> io::Result<()> {
    let payload = base64::encode_config(path.as_bytes(), base64::STANDARD);

    print_cmd_payload(
        &[("f", "100"), ("a", "T"), ("i", "0"), ("t", "f")],
        &payload,
    )?;

    Ok(())
}

fn display_image(img: &DynamicImage, _max_size: (u16, u16)) -> io::Result<()> {
    let (width, height) = img.dimensions();
    let (data, bits) = match img {
        DynamicImage::ImageRgb8(rgb) => (rgb.to_vec(), 24),
        DynamicImage::ImageRgba8(rgba) => (rgba.to_vec(), 32),
        _ => unimplemented!("IMG Variant"),
    };
    let payload = base64::encode_config(&data, base64::STANDARD);

    print_cmd_payload(
        &[
            ("f", &bits.to_string()),
            ("a", "T"),
            ("i", "0"),
            ("t", "d"),
            ("s", &width.to_string()),
            ("v", &height.to_string()),
        ],
        &payload,
    )?;
    Ok(())
}

// TODO: Find out why terminal isnt sending anything
#[allow(dead_code)]
fn read_term_response() -> io::Result<()> {
    let mut stdout = stdout().into_raw_mode()?;
    let mut stdin = stdin();

    let mut data = Vec::new();
    let mut buf = [0; 1];
    loop {
        if data.len() >= 2 && &data[data.len() - 2..] == PROTOCOL_END {
            break;
        }
        let _ = stdin.read(&mut buf)?;
        data.push(buf[0]);
    }
    // Dont let the compiler remove stdout
    stdout.flush()?;
    Ok(())
}

pub struct Kitty;

impl display::TermDisplay for Kitty {
    fn animated(
        &self,
        _options: &Options,
        _term_size: (u16, u16),
        _img_src: display::ImageSource,
    ) -> display::Result<()> {
        // TODO: Implement
        unimplemented!();
    }

    fn still(
        &self,
        _options: &Options,
        term_size: (u16, u16),
        mut img_src: display::ImageSource,
    ) -> display::Result<()> {
        display_image(img_src.image().ok_or(())?, term_size).map_err(|_| ())
    }
}
