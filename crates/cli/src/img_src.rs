use image::{codecs::gif::GifDecoder, AnimationDecoder, DynamicImage};
use std::{
    fs::File,
    io::{stdin, Read},
    result::Result::{Err, Ok},
};

pub struct ImageSource(String);

impl ImageSource {
    pub fn new(filename: String) -> Self {
        Self(filename)
    }

    pub fn has_path(&self) -> bool {
        self.0 != "-" && std::path::PathBuf::from(&self.0).exists()
    }

    pub fn has_frames(&self) -> bool {
        self.0.ends_with(".gif")
    }

    pub fn path(&self) -> &str {
        &self.0
    }

    pub fn img(&self) -> DynamicImage {
        if self.0 == "-" {
            let mut buf = Vec::new();
            if let Err(e) = stdin().read_to_end(&mut buf) {
                eprintln!("Error occurred reading file: {}", e);
                std::process::exit(1)
            };

            match image::load_from_memory(&buf) {
                Ok(img) => img,
                Err(e) => {
                    eprintln!("Error occurred parsing file: {}", e);
                    std::process::exit(1)
                }
            }
        } else {
            match image::open(&self.0) {
                Ok(img) => img,
                Err(e) => {
                    eprintln!("Error occurred opening image: {}", e);
                    std::process::exit(1)
                }
            }
        }
    }

    pub fn raw(&self) -> Vec<u8> {
        let mut out = Vec::new();
        if self.0 == "-" {
            if let Err(e) = stdin().read_to_end(&mut out) {
                eprintln!("Error occurred reading file: {}", e);
                std::process::exit(1);
            }
        } else {
            match File::open(&self.0) {
                Ok(mut file) => {
                    if let Err(e) = file.read_to_end(&mut out) {
                        eprintln!("Error occurred reading file: {}", e);
                        std::process::exit(1);
                    }
                }
                Err(e) => {
                    eprintln!("Error occurred opening file: {}", e);
                    std::process::exit(1);
                }
            }
        }
        out
    }

    pub fn frames(&self) -> image::Frames {
        // To get frames, we need a gif::Decoder, which
        // takes a type that is Read
        if self.0 == "-" {
            GifDecoder::new(stdin()).ok().unwrap().into_frames()
        } else {
            let f = File::open(&self.0).expect("File not found");

            GifDecoder::new(f).ok().unwrap().into_frames()
        }
    }
}
