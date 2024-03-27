//! Utils for playing multiple frames.
//! Probably you only need to use that if you are creating a CLI
//!
//! # Examples
//!
//! ```
//! use std::error::Error;
//! use tapciify::{AsciiPlayer, AsciiPlayerOptions};
//!
//! # fn main() -> Result<(), Box<dyn Error>> {
//! let paths = ["./assets/examples/original.webp".into()];
//!
//! AsciiPlayer::play(
//!     &paths,
//!     &AsciiPlayerOptions {
//!         width: Some(64),
//!         // Put your other options here
//!         ..Default::default()
//!     },
//! )?;
//!
//! # Ok(())
//! # }
//! ```

use crate::{
    ascii::{
        AsciiArt, AsciiArtConverter, AsciiArtConverterError, AsciiArtConverterOptions,
        ReverseString, DEFAULT_ASCII_STRING,
    },
    braille::BrailleArtConverter,
    CustomRatioResize, DEFAULT_FONT_RATIO,
};
use crossterm::{cursor::MoveUp, execute};
use image::imageops::FilterType;
use imageproc::contrast::adaptive_threshold;
use indicatif::ProgressStyle;
use std::{io::stdout, path::PathBuf, time::Instant};
use thiserror::Error;

#[cfg(feature = "rayon")]
use rayon::prelude::*;

#[cfg(feature = "rayon")]
use indicatif::ParallelProgressIterator;
#[cfg(not(feature = "rayon"))]
use indicatif::ProgressIterator;

/// Calculate frame time in millis (1 / framerate)
///
/// # Examples
///
/// ```
/// use tapciify::player::calculate_frame_time;
///
/// let result = calculate_frame_time(Some(20.0));
///
/// assert_eq!(result, 50)
/// ```
pub fn calculate_frame_time(framerate: Option<f64>) -> u64 {
    framerate.map_or(0, |framerate| (1000f64 / framerate) as u64)
}

/// Player to convert and play frames
#[derive(Debug, Clone)]
pub struct AsciiPlayer {}

impl AsciiPlayer {
    #[deprecated(
        since = "3.1.0",
        note = "Use `tapciify::ascii::ReverseString::reverse` instead"
    )]
    /// Reverse ASCII string
    pub fn reverse_ascii_string(ascii_string: String) -> String {
        ascii_string.reverse()
    }

    /// Play paths as ASCII arts
    pub fn play_frames(
        paths: &[PathBuf],
        options: &AsciiPlayerOptions,
    ) -> Result<(), AsciiPlayerError> {
        let mut first_frame = true;

        let converter_options = options.to_owned().into();

        loop {
            for path in paths.iter() {
                let start = Instant::now();
                let img = image::open(path)?;

                let ascii_art = match options.braille {
                    true => {
                        let resized_img = img.resize(
                            options.width.map_or(u32::MAX, |width| width * 2),
                            options.height.map_or(u32::MAX, |height| height * 4),
                            options.filter,
                        );

                        resized_img.braille_art(options.threshold.unwrap_or(5))?
                    }
                    false => {
                        let prepared_img = if let Some(threshold) = options.threshold {
                            image::DynamicImage::from(adaptive_threshold(
                                &img.to_luma8(),
                                threshold,
                            ))
                        } else {
                            img
                        }
                        .resize_custom_ratio(
                            options.width,
                            options.height,
                            options.font_ratio,
                            options.filter,
                        );

                        prepared_img.ascii_art(&converter_options)?
                    }
                };

                if !first_frame {
                    execute!(stdout(), MoveUp(ascii_art.height.try_into().unwrap()))
                        .unwrap_or_default();
                } else {
                    first_frame = false;
                }

                println!("{}", ascii_art);

                while options.frame_time > start.elapsed().as_millis().try_into().unwrap() {}
            }

            if !options.looped {
                break;
            }
        }

        Ok(())
    }

    /// Convert paths to of ASCII arts
    fn pre_render(
        paths: &[PathBuf],
        options: &AsciiPlayerOptions,
    ) -> Result<Vec<AsciiArt>, AsciiPlayerError> {
        let converter_options = options.to_owned().into();

        #[cfg(feature = "rayon")]
        let iter = paths.into_par_iter();
        #[cfg(not(feature = "rayon"))]
        let iter = paths.iter();

        // Moving this into the function argument breaks rustfmt
        let progress_bar_style = ProgressStyle::with_template(
            "{elapsed_precise} | {wide_bar} {percent}% | ETA: {eta} | FPS: {per_sec} | {pos}/{len}",
        )
        .unwrap_or_else(|_| ProgressStyle::default_bar());

        let frames = iter
            .progress_with_style(progress_bar_style)
            .map(|path| {
                let img = image::open(path)?;
                let ascii_art = match options.braille {
                    true => {
                        let resized_img = img.resize(
                            options.width.map_or(u32::MAX, |width| width * 2),
                            options.height.map_or(u32::MAX, |height| height * 4),
                            options.filter,
                        );

                        resized_img.braille_art(options.threshold.unwrap_or(5))?
                    }
                    false => {
                        let prepared_img = if let Some(threshold) = options.threshold {
                            image::DynamicImage::from(adaptive_threshold(
                                &img.to_luma8(),
                                threshold,
                            ))
                        } else {
                            img
                        }
                        .resize_custom_ratio(
                            options.width,
                            options.height,
                            options.font_ratio,
                            options.filter,
                        );

                        prepared_img.ascii_art(&converter_options)?
                    }
                };

                Ok(ascii_art)
            })
            .collect::<Result<Vec<AsciiArt>, AsciiPlayerError>>()?;

        Ok(frames)
    }

    /// Convert paths to of ASCII arts and play them
    pub fn play_pre_rendered_frames(
        paths: &[PathBuf],
        options: &AsciiPlayerOptions,
    ) -> Result<(), AsciiPlayerError> {
        let mut first_frame = true;

        let frames = AsciiPlayer::pre_render(paths, options)?;

        loop {
            frames.iter().for_each(|ascii_art| {
                let start = Instant::now();

                if !first_frame {
                    execute!(stdout(), MoveUp(ascii_art.height.try_into().unwrap()))
                        .unwrap_or_default();
                } else {
                    first_frame = false;
                }

                println!("{}", ascii_art);

                while options.frame_time > start.elapsed().as_millis().try_into().unwrap() {}
            });

            if !options.looped {
                break;
            }
        }

        Ok(())
    }

    /// Play frames
    ///
    /// Calls [`AsciiPlayer::play_frames`] or [`AsciiPlayer::play_pre_rendered_frames`], depending on [`AsciiPlayerOptions`]
    ///
    /// # Examples
    ///
    /// ```
    /// use tapciify::{AsciiPlayer, AsciiPlayerOptions};
    ///
    /// let paths = vec!["./assets/examples/original.webp".into()];
    ///
    /// let options = AsciiPlayerOptions {
    ///     width: Some(128),
    ///     ..Default::default()
    /// };
    ///
    /// assert!(AsciiPlayer::play(&paths, &options).is_ok())
    /// ```
    pub fn play(paths: &[PathBuf], options: &AsciiPlayerOptions) -> Result<(), AsciiPlayerError> {
        if options.pre_render {
            return AsciiPlayer::play_pre_rendered_frames(paths, options);
        }

        AsciiPlayer::play_frames(paths, options)
    }
}

/// Options of player to convert and play frames
#[derive(Debug, Clone)]
pub struct AsciiPlayerOptions {
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub ascii_string: String,
    pub colored: bool,
    pub frame_time: u64,
    pub pre_render: bool,
    pub font_ratio: f64,
    pub looped: bool,
    pub filter: FilterType,
    pub threshold: Option<u32>,
    pub braille: bool,
}

impl Default for AsciiPlayerOptions {
    fn default() -> AsciiPlayerOptions {
        AsciiPlayerOptions {
            width: None,
            height: None,
            ascii_string: DEFAULT_ASCII_STRING.to_owned(),
            colored: false,
            frame_time: 0,
            pre_render: false,
            font_ratio: DEFAULT_FONT_RATIO,
            looped: false,
            filter: FilterType::Triangle,
            threshold: None,
            braille: false,
        }
    }
}

impl From<AsciiPlayerOptions> for AsciiArtConverterOptions {
    fn from(o: AsciiPlayerOptions) -> AsciiArtConverterOptions {
        AsciiArtConverterOptions {
            ascii_string: o.ascii_string,
            colored: o.colored,
        }
    }
}

/// Error caused by [`AsciiPlayer`]
#[derive(Debug, Error)]
pub enum AsciiPlayerError {
    #[error("{0}")]
    Image(#[from] image::ImageError),
    #[error("{0}")]
    AsciiConverter(#[from] AsciiArtConverterError),
}
