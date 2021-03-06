#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct ItermOptions {
    pub size: (Option<u16>, Option<u16>),
}

/// iTerm2 proprietary protocol renderer
///
/// Supports full resolution rendering, but only in terminals that support the [iTerm2 protocol](https://iterm2.com/documentation-images.html)
#[derive(Debug, Copy, Clone)]
pub struct Iterm;

impl Iterm {
    /// Render image from raw image contents
    ///
    /// Supports any image file type that macos supports (i.e. JPG, GIF, PDF, PICT, EPS, etc.)
    pub fn data(options: &ItermOptions, data: &[u8]) -> std::io::Result<()> {
        let mut file = iterm2::File::new(data);
        if let Some(width) = options.size.0 {
            file.width(iterm2::Dimension::Cells(width as u32));
        }
        if let Some(height) = options.size.1 {
            file.height(iterm2::Dimension::Cells(height as u32));
        }
        file.show()?;
        Ok(())
    }

    /// Render image from a path
    ///
    /// Supports any image file type that macos supports (i.e. JPG, GIF, PDF, PICT, EPS, etc.)
    pub fn path(options: &ItermOptions, path: &str) -> std::io::Result<()> {
        let mut file = iterm2::File::read(path)?;
        if let Some(width) = options.size.0 {
            file.width(iterm2::Dimension::Cells(width as u32));
        }
        if let Some(height) = options.size.1 {
            file.height(iterm2::Dimension::Cells(height as u32));
        }
        file.show()?;
        Ok(())
    }
}
