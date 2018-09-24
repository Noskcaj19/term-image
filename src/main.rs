extern crate base64;
extern crate clap;
extern crate failure;
extern crate gif;
extern crate image;
extern crate iterm2;
extern crate libc;
extern crate signal_hook;
extern crate termion;

mod args;
mod options;
mod renderer;
pub use options::*;
mod utils;

fn main() {
    let options = args::get_options();

    if !options.isatty && !options.width.is_some() && !options.width.is_some() {
        return;
    }

    // TODO: If-let chains
    let term_size = if options.width.is_some() || options.height.is_some() {
        (
            // safe unwraps
            options.width.unwrap() as u16,
            options.height.unwrap() as u16,
        )
    } else if !options.isatty {
        (80, 25)
    } else {
        match termion::terminal_size() {
            // Adds some padding
            Ok(size) => (size.0 - 4, size.1 - 8),
            Err(_) => return,
        }
    };

    renderer::render_image(&options, term_size);
}
