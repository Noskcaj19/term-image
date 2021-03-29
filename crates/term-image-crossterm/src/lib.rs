use crossterm::{
    queue,
    style::{Color, SetBackgroundColor, SetForegroundColor},
};
use std::io::Write;

/// Print cells to the terminal with ansi 256-color escapes or truecolor (RGB) escapes
pub trait TermWriter {
    /// Write cell to `out`, optionally using truecolor
    fn write(&self, truecolor: bool, out: &mut impl Write) -> crossterm::Result<()> {
        if truecolor {
            self.write_truecolor(out)
        } else {
            self.write_256(out)
        }
    }

    /// Write cell to `out`, using truecolor.  Note that not all temrinals support truecolor
    fn write_truecolor(&self, out: &mut impl Write) -> crossterm::Result<()>;
    /// Write cell to `out`, using 256 color
    fn write_256(&self, out: &mut impl Write) -> crossterm::Result<()>;
}

impl TermWriter for term_image::block::Cell {
    fn write_truecolor(&self, out: &mut impl Write) -> crossterm::Result<()> {
        queue!(
            out,
            SetForegroundColor(Color::from(self.fg.0)),
            SetBackgroundColor(Color::from(self.bg.0))
        )?;
        write!(out, "{}", self.ch)?;
        Ok(())
    }

    fn write_256(&self, out: &mut impl Write) -> crossterm::Result<()> {
        queue!(
            out,
            SetForegroundColor(Color::AnsiValue(self.fg.as_256().0)),
            SetBackgroundColor(Color::AnsiValue(self.bg.as_256().0)),
        )?;
        write!(out, "{}", self.ch)?;
        Ok(())
    }
}

impl TermWriter for term_image::ascii::Cell {
    fn write_truecolor(&self, out: &mut impl Write) -> crossterm::Result<()> {
        queue!(out, SetForegroundColor(Color::from(self.fg.0)))?;
        write!(out, "{}", self.ch)?;
        Ok(())
    }

    fn write_256(&self, out: &mut impl Write) -> crossterm::Result<()> {
        queue!(
            out,
            SetForegroundColor(Color::AnsiValue(self.fg.as_256().0))
        )?;
        write!(out, "{}", self.ch)?;
        Ok(())
    }
}

impl TermWriter for term_image::braille::Cell {
    fn write_truecolor(&self, out: &mut impl Write) -> crossterm::Result<()> {
        queue!(out, SetForegroundColor(Color::from(self.fg.0)))?;
        write!(out, "{}", self.ch)?;
        Ok(())
    }

    fn write_256(&self, out: &mut impl Write) -> crossterm::Result<()> {
        queue!(
            out,
            SetForegroundColor(Color::AnsiValue(self.fg.as_256().0))
        )?;
        write!(out, "{}", self.ch)?;
        Ok(())
    }
}
