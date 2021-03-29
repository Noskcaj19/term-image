use crate::args::RendererOption;
use crossterm::{cursor, queue, style, terminal};
use image::Delay;
use img_src::ImageSource;
use itertools::IntoChunks;
use std::{
    io::{stdout, Write},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};
use term_image::{ascii::Ascii, block::Block, braille::Braille, iterm::Iterm, kitty::Kitty};
use term_image_crossterm::TermWriter;

mod args;
mod img_src;

fn main() {
    let options = args::get_options();
    let animated = !options.still;

    let stdout = stdout();
    let mut stdout = stdout.lock();

    let src = ImageSource::new(options.path);

    match options.renderer_options {
        RendererOption::Block(block_options) => {
            if src.has_frames() && animated {
                write_animated(
                    Block::animated(&block_options, src.frames()),
                    options.truecolor,
                );
            } else {
                write_still(
                    Block::img(&block_options, &src.img()).into_iter(),
                    options.truecolor,
                );
            }
        }
        RendererOption::Ascii(ascii_options) => {
            if src.has_frames() && animated {
                write_animated(
                    Ascii::animated(&ascii_options, src.frames()),
                    options.truecolor,
                );
            } else {
                write_still(
                    Ascii::img(&ascii_options, &src.img()).into_iter(),
                    options.truecolor,
                );
            }
        }
        RendererOption::Braille(braille_options) => {
            if src.has_frames() && animated {
                write_animated(
                    Braille::animated(&braille_options, src.frames()),
                    options.truecolor,
                );
            } else {
                write_still(
                    Braille::img(&braille_options, &src.img()).into_iter(),
                    options.truecolor,
                );
            }
        }
        RendererOption::Kitty(kitty_options) => {
            if let Err(e) = Kitty::img(&kitty_options, &src.img(), &mut stdout) {
                eprintln!("An error occurred printing image: {}", e)
            }
        }
        RendererOption::Iterm(iterm_options) => {
            let err = if src.has_path() {
                Iterm::path(&iterm_options, src.path())
            } else {
                Iterm::data(&iterm_options, &src.raw())
            };
            if let Err(e) = err {
                eprintln!("An error occurred printing image: {}", e)
            }
        }
    }
}

/// Returns a Arc reference to a boolean value that is set to true when a "exit"
/// signal is recieved (currently INT, QUIT, TERM, and WINCH).
fn get_quit_hook() -> Arc<AtomicBool> {
    let atomic = Arc::new(AtomicBool::new(false));
    for signal in &[
        signal_hook::consts::SIGINT,
        signal_hook::consts::SIGQUIT,
        signal_hook::consts::SIGTERM,
        signal_hook::consts::SIGWINCH,
    ] {
        signal_hook::flag::register(*signal, Arc::clone(&atomic))
            .expect("Unable to hook a termination signal");
    }
    atomic
}

fn write_still(iter: impl Iterator<Item = impl Iterator<Item = impl TermWriter>>, truecolor: bool) {
    let stdout = std::io::stdout();
    let mut stdout = stdout.lock();
    for row in iter.into_iter() {
        for block in row {
            let _ = block.write(truecolor, &mut stdout);
        }
        let _ = queue!(stdout, crossterm::style::ResetColor);
        let _ = writeln!(stdout);
    }
    let _ = stdout.flush();
}

fn write_animated(
    frames: impl Iterator<
        Item = (
            Delay,
            IntoChunks<impl Iterator<Item = impl TermWriter + Clone>>,
        ),
    >,
    truecolor: bool,
) {
    let stdout = std::io::stdout();
    let mut stdout = stdout.lock();
    let iter = frames.collect::<Vec<_>>();
    let iter = iter
        .into_iter()
        .map(|(delay, frame)| {
            (
                delay,
                frame
                    .into_iter()
                    .map(|row| row.collect::<Vec<_>>())
                    .collect::<Vec<_>>(),
            )
        })
        .collect::<Vec<_>>();

    let stopping = get_quit_hook();
    let _ = queue!(
        stdout,
        terminal::Clear(terminal::ClearType::All),
        cursor::Hide
    );

    // TODO: Hide cursor, save cursor
    for (delay, frame) in iter.into_iter().cycle() {
        if stopping.load(Ordering::Relaxed) {
            break;
        }
        let _ = queue!(stdout, cursor::MoveTo(0, 0));
        for row in frame.into_iter() {
            for block in row {
                let _ = block.write(truecolor, &mut stdout);
            }
            let _ = queue!(stdout, style::ResetColor);
            let _ = writeln!(stdout);
        }
        let _ = stdout.flush();
        std::thread::sleep(Duration::from(delay));
    }
    let _ = queue!(
        stdout,
        terminal::Clear(terminal::ClearType::All),
        cursor::Show
    );
}
