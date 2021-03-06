// TODO: Revamp this and add more options.
//       Probably extract it to a crate too

use super::resize_image;
use image::{DynamicImage, GenericImageView};
use std::io::{self, Write};

const PROTOCOL_START: &[u8] = b"\x1b_G";
const PROTOCOL_END: &[u8] = b"\x1b\\";
const MAX_BUFFER: usize = 2048;

fn print_cmd_payload(out: &mut impl Write, cmds: &[(&str, &str)], payload: &str) -> io::Result<()> {
    let cmds = cmds
        .iter()
        .map(|(l, r)| format!("{}={}", l, r))
        .collect::<Vec<_>>()
        .join(",");

    let mut payload = payload.as_bytes();
    while payload.len() > MAX_BUFFER {
        let (buf, new_payload) = payload.split_at(MAX_BUFFER);
        payload = new_payload;
        out.write_all(PROTOCOL_START)?;
        write!(out, "{},m=1;", cmds)?;
        out.write_all(&buf)?;
        out.write_all(PROTOCOL_END)?;
    }
    out.write_all(PROTOCOL_START)?;
    write!(out, "{},m=0;", cmds)?;
    out.write_all(&payload)?;
    out.write_all(PROTOCOL_END)?;

    Ok(())
}

/// Display an image from an canonical path
///
/// # Notes
/// Path _must_ be canonical, use std::fs::canonicalize
fn display_path(out: &mut impl Write, path: &str) -> io::Result<()> {
    let payload = base64::encode_config(
        std::fs::canonicalize(path)?
            .as_os_str()
            .to_string_lossy()
            .as_bytes(),
        base64::STANDARD,
    );

    print_cmd_payload(out, &[("f", "100"), ("a", "T"), ("t", "f")], &payload)?;
    out.flush()?;

    Ok(())
}

fn display_image(out: &mut impl Write, img: &DynamicImage, max_size: (u16, u16)) -> io::Result<()> {
    let img = resize_image(&img, (1, 1), max_size);
    let (width, height) = img.dimensions();
    let (data, bits) = match img {
        DynamicImage::ImageRgb8(rgb) => (rgb.to_vec(), 24),
        DynamicImage::ImageRgba8(rgba) => (rgba.to_vec(), 32),
        _ => unimplemented!("IMG Variant"),
    };
    let payload = base64::encode_config(&data, base64::STANDARD);

    print_cmd_payload(
        out,
        &[
            ("f", bits.to_string().as_str()),
            ("a", "T"),
            ("i", "0"),
            ("t", "d"),
            ("s", width.to_string().as_str()),
            ("v", height.to_string().as_str()),
        ],
        &payload,
    )?;
    Ok(())
}

// TODO: Find out why terminal isnt sending anything
// fn read_term_response() -> io::Result<()> {
//     let mut stdout = stdout().into_raw_mode()?;
//     let mut stdin = stdin();

//     let mut data = Vec::new();
//     let mut buf = [0; 1];
//     loop {
//         if data.len() >= 2 && &data[data.len() - 2..] == PROTOCOL_END {
//             break;
//         }
//         let _ = stdin.read(&mut buf)?;
//         data.push(buf[0]);
//     }
//     // Dont let the compiler remove stdout
//     stdout.flush()?;
//     Ok(())
// }

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct KittyOptions {
    pub size: (u16, u16),
}

#[derive(Debug, Copy, Clone)]
/// Kitty proprietary protocol renderer
///
/// Supports full resolution rendering, but only in terminals that support the [kitty protocol](https://sw.kovidgoyal.net/kitty/graphics-protocol.html)
pub struct Kitty;

impl Kitty {
    /// Render full resolution image in kitty
    pub fn img(
        options: &KittyOptions,
        img: &DynamicImage,
        out: &mut impl Write,
    ) -> std::io::Result<()> {
        display_image(out, img, options.size)
    }

    /// Render full resolution image in kitty from a path
    pub fn path(path: &str, out: &mut impl Write) -> std::io::Result<()> {
        display_path(out, path)
    }
}
