use image::Pixel;

use crate::{
    braille::{boolean_array_to_braille, calc_braille_pixels},
    product, AsciiArt, AsciiArtPixel, SizeError,
};

#[cfg(feature = "rayon")]
use rayon::prelude::*;

pub enum DitheringBrailleThreshold {
    White,
    Gray,
    Black,
}

pub trait DitheringBraillePixel {
    fn dithering_braille_pixel(&self) -> DitheringBrailleThreshold;
}

impl DitheringBraillePixel for image::Luma<u8> {
    fn dithering_braille_pixel(&self) -> DitheringBrailleThreshold {
        if self[0] as f32 > 255.0 * 2.0 / 3.0 {
            return DitheringBrailleThreshold::White;
        } else if self[0] as f32 > 255.0 / 3.0 {
            return DitheringBrailleThreshold::Gray;
        }

        DitheringBrailleThreshold::Black
    }
}

impl DitheringBraillePixel for image::LumaA<u8> {
    fn dithering_braille_pixel(&self) -> DitheringBrailleThreshold {
        let p = (self[0] as u16 * self[1] as u16) as f32;

        if p > 255.0 * 255.0 * 2.0 / 3.0 {
            return DitheringBrailleThreshold::White;
        } else if p > 255.0 * 255.0 / 3.0 {
            return DitheringBrailleThreshold::Gray;
        }

        DitheringBrailleThreshold::Black
    }
}

impl DitheringBraillePixel for image::Rgb<u8> {
    fn dithering_braille_pixel(&self) -> DitheringBrailleThreshold {
        self.to_luma().dithering_braille_pixel()
    }
}

impl DitheringBraillePixel for image::Rgba<u8> {
    fn dithering_braille_pixel(&self) -> DitheringBrailleThreshold {
        self.to_luma_alpha().dithering_braille_pixel()
    }
}

pub trait DitheringBrailleConverter {
    fn dithering_braille_art(&self, colored: bool) -> Result<AsciiArt, SizeError>;
}

impl DitheringBrailleConverter for image::DynamicImage {
    fn dithering_braille_art(&self, colored: bool) -> Result<AsciiArt, SizeError> {
        self.to_rgba8().dithering_braille_art(colored)
    }
}

impl DitheringBrailleConverter for image::RgbaImage {
    fn dithering_braille_art(&self, colored: bool) -> Result<AsciiArt, SizeError> {
        let width = self.width();
        let height = self.height();

        if width < 4 || height < 8 {
            return Err(SizeError);
        }

        let x_range: Vec<u32> = (0..(width - width % 2)).step_by(2).collect();
        let y_range: Vec<u32> = (0..(height - height % 4)).step_by(4).collect();

        let width = x_range.clone().len() as u32;
        let height = y_range.clone().len() as u32;

        let range: Vec<(u32, u32)> = product![y_range, x_range].map(|(y, x)| (*y, *x)).collect();

        #[cfg(feature = "rayon")]
        let iter = range.into_par_iter();
        #[cfg(not(feature = "rayon"))]
        let iter = range.into_iter();

        let characters = iter
            .map(|(y, x)| {
                // Top left pixel
                let tlpx = self.get_pixel(x, y);

                let braille_array = calc_braille_pixels(x, y).map(|p| {
                    match self.get_pixel(p.0, p.1).dithering_braille_pixel() {
                        DitheringBrailleThreshold::Gray => p.0 % 2 == p.1 % 2,
                        DitheringBrailleThreshold::White => true,
                        DitheringBrailleThreshold::Black => false,
                    }
                });

                AsciiArtPixel {
                    character: boolean_array_to_braille(&braille_array),
                    r: tlpx.0[0],
                    g: tlpx.0[1],
                    b: tlpx.0[2],
                    a: tlpx.0[3],
                }
            })
            .collect();

        Ok(AsciiArt::new(characters, width, height, colored))
    }
}
