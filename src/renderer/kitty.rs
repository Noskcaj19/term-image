use base64;
use image::Frames;
use std::fs;
use std::io::{self, stdin, stdout, BufReader, Read, Write};
use termion::raw::IntoRawMode;
use Options;

const PROTOCOL_START: &'static [u8] = b"\x1b_G";
const PROTOCOL_END: &'static [u8] = b"\x1b\\";
const MAX_BUFFER: usize = 2048;

fn print_cmd_payload(cmds: &[(&str, &str)], payload: &str) -> io::Result<()> {
    let cmds = cmds
        .iter()
        .map(|(l, r)| format!("{}={}", l, r))
        .collect::<Vec<_>>()
        .join(",");

    let stdout = stdout();
    let mut stdout = stdout.lock();

    stdout.write(PROTOCOL_START)?;
    let mut buf_reader = BufReader::new(payload.as_bytes());
    let mut buf = [0; MAX_BUFFER];
    let mut bytes_read = buf_reader.read(&mut buf)?;
    while bytes_read > MAX_BUFFER {
        stdout.write(PROTOCOL_START)?;
        write!(stdout, "{},m=1;", cmds)?;
        stdout.write(&buf)?;
        bytes_read = buf_reader.read(&mut buf)?;
    }
    write!(stdout, "{},m=0;", cmds)?;
    stdout.write(&buf[..bytes_read])?;
    stdout.write(PROTOCOL_END)?;
    stdout.flush()?;
    Ok(())
}

fn display_path(path: &str) -> io::Result<()> {
    let payload = base64::encode_config(path.as_bytes(), base64::STANDARD);

    print_cmd_payload(
        &[("f", "100"), ("a", "T"), ("i", "1"), ("t", "f")],
        &payload,
    )?;

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
    display_path(&fs::canonicalize(path)?.to_string_lossy())?;

    read_term_response()?;

    Ok(())
}

// TODO: Find a way to reduce duplication
pub fn print_frames(_options: &Options, _max_size: (u16, u16), _frames: Frames) {
    unimplemented!()
}
