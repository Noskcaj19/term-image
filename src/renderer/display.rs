use image;
use image::AnimationDecoder;

use std::fs::File;

use crate::options::Options;

pub type Result<E> = std::result::Result<(), E>;

pub struct ImageSource {
    path: String,
    image: Option<::image::DynamicImage>,
}

impl ImageSource {
    pub fn new(path: &str) -> ImageSource {
        ImageSource {
            path: path.to_owned(),
            image: None,
        }
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn image(&mut self) -> Option<&image::DynamicImage> {
        if let Some(ref image) = self.image {
            return Some(image);
        }

        self.image = if self.path == "-" {
            use std::io::{stdin, Read};
            let mut buf = Vec::new();
            stdin().read_to_end(&mut buf).ok()?;
            image::load_from_memory(&buf).ok()
        } else {
            Some(image::open(&self.path).ok()?)
        };

        self.image.as_ref()
    }

    pub fn frames(&mut self) -> Option<image::Frames> {
        // To get frames, we need a gif::Decoder, which
        // takes a type that is Read
        use image::ImageDecoder;
        if self.path == "-" {
            use std::io::stdin;
            Some(image::gif::Decoder::new(stdin()).ok()?.into_frames())
        } else {
            let f = File::open(&self.path).expect("File not found");

            Some(image::gif::Decoder::new(f).ok()?.into_frames())
        }
    }

    pub fn data(&mut self) -> Box<dyn std::io::Read> {
        use std::io::stdin;
        if self.path == "-" {
            Box::new(stdin())
        } else {
            Box::new(File::open(&self.path).expect("File not found"))
        }
    }
}

// TODO: Error type
pub trait TermDisplay {
    fn animated(
        &self,
        options: &Options,
        term_size: (u16, u16),
        img_src: ImageSource,
    ) -> Result<()>;
    fn still(&self, options: &Options, term_size: (u16, u16), img_src: ImageSource) -> Result<()>;
}
