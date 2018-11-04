extern crate term_image;

use term_image::{args, renderer};

fn main() {
    let options = args::get_options();

    if !options.isatty && options.width.is_none() && options.width.is_none() {
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
