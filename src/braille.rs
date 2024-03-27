use image::GenericImageView;

use crate::{AsciiArt, AsciiArtConverterError, AsciiArtPixel, SizeError};

#[cfg(feature = "rayon")]
use rayon::prelude::*;

/// Convert array of booleans into braille character
///
/// Grid of booleans placement
///
/// |---|---|
/// | 0 | 3 |
/// | 1 | 4 |
/// | 2 | 5 |
/// | 6 | 7 |
pub fn boolean_array_to_braille(array: &[bool; 8]) -> char {
    let mut codepoint: u32 = 0x2800; // Base codepoint for Braille Pattern

    // Calculate the codepoint based on the boolean array
    for (i, &value) in array.iter().enumerate() {
        if value {
            codepoint |= 1 << i;
        }
    }

    // Convert the codepoint to a char
    std::char::from_u32(codepoint).unwrap_or(' ')
}

pub trait BrailleArtConverter {
    fn braille_art(&self, block_radius: u32) -> Result<AsciiArt, AsciiArtConverterError>;
}

impl BrailleArtConverter for image::DynamicImage {
    fn braille_art(&self, block_radius: u32) -> Result<AsciiArt, AsciiArtConverterError> {
        self.to_luma8().braille_art(block_radius)
    }
}

impl BrailleArtConverter for image::GrayImage {
    fn braille_art(&self, block_radius: u32) -> Result<AsciiArt, AsciiArtConverterError> {
        if self.width() < 8 || self.height() < 8 {
            return Err(AsciiArtConverterError::SizeError(SizeError));
        }

        let img: image::DynamicImage =
            imageproc::contrast::adaptive_threshold(self, block_radius).into();

        let y_range = 0..(img.height() / 4);
        #[cfg(feature = "rayon")]
        let iter = y_range.into_par_iter();
        #[cfg(not(feature = "rayon"))]
        let iter = y_range.into_iter();

        let characters = iter
            .flat_map(|y| {
                let mut row = vec![];

                for x in 0..(img.width() / 2) {
                    let x = x * 2;
                    let y = y * 4;

                    let braille_array = &[
                        img.get_pixel(x, y),
                        img.get_pixel(x, y + 1),
                        img.get_pixel(x, y + 2),
                        img.get_pixel(x + 1, y),
                        img.get_pixel(x, y + 1),
                        img.get_pixel(x, y + 2),
                        img.get_pixel(x, y + 3),
                        img.get_pixel(x + 1, y + 3),
                    ]
                    .map(|p| p[0] > 123);

                    row.push(AsciiArtPixel {
                        character: boolean_array_to_braille(braille_array),
                        r: 255,
                        g: 255,
                        b: 255,
                        a: 255,
                    })
                }

                row
            })
            .collect();

        Ok(AsciiArt::new(
            characters,
            img.width() / 2,
            img.height() / 4,
            false,
        ))
    }
}
