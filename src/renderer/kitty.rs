use base64;
use image::Frames;
use std::fs;
use std::io::{self, stdin, stdout, Read, Write};
use termion::raw::IntoRawMode;
use Options;

const PROTOCOL_START: &'static [u8] = b"\x1b_G";
const PROTOCOL_END: &'static [u8] = b"\x1b\\";

fn display_from_path(path: &str) -> io::Result<()> {
    let mut stdout = stdout();
    stdout.write(PROTOCOL_START)?;
    stdout.write(b"f=100,a=T,i=1,t=f;")?;
    let payload = base64::encode_config(path.as_bytes(), base64::STANDARD);
    stdout.write(payload.as_bytes())?;
    stdout.write(PROTOCOL_END)?;
    stdout.flush()?;

    Ok(())
}

fn read_term_response() -> io::Result<()> {
    let mut stdout = stdout().into_raw_mode()?;
    let mut stdin = stdin();

    let mut data = Vec::new();
    let mut buf = [0; 1];
    loop {
        if data.len() >= 2 {
            if &data[data.len() - 2..] == PROTOCOL_END {
                break;
            }
        }
        stdin.read(&mut buf)?;
        data.push(buf[0]);
    }
    // Dont let the compiler remove stdout
    stdout.flush()?;
    Ok(())
}

pub fn display(_options: &Options, _max_size: (u16, u16), path: &str) -> io::Result<()> {
    display_from_path(&fs::canonicalize(path)?.to_string_lossy())?;

    read_term_response()?;

    Ok(())
}

// TODO: Find a way to reduce duplication
pub fn print_frames(_options: &Options, _max_size: (u16, u16), _frames: Frames) {
    unimplemented!()
}
