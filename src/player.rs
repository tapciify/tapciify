//! Utils for playing multiple frames.
//! Probably you only need to use that if you are creating a CLI
//!
//! # Examples
//!
//! ```
//! use tapciify::{AsciiPlayer, AsciiPlayerOptions};
//!
//! # use std::error::Error;
//!
//! # fn main() -> Result<(), Box<dyn Error>> {
//! let paths = ["./assets/examples/ferris.webp".into()];
//!
//! AsciiPlayer::play(
//!     &paths,
//!     &AsciiPlayerOptions {
//!         width: Some(64),
//!         // Put your other options here
//!         ..Default::default()
//!     },
//! )?;
//! # Ok(())
//! # }
//! ```

use std::io::stdout;
use std::path::PathBuf;
use std::time::Instant;
use std::{error, fmt};

use crossterm::cursor::MoveUp;
use crossterm::execute;
use image::imageops::FilterType;
use imageproc::contrast::adaptive_threshold;
#[cfg(feature = "rayon")]
use indicatif::ParallelProgressIterator;
#[cfg(not(feature = "rayon"))]
use indicatif::ProgressIterator;
use indicatif::ProgressStyle;
#[cfg(feature = "rayon")]
use rayon::prelude::*;

use crate::background_string::BackgroundStringArtConverter;
use crate::braille::BrailleArtConverter;
use crate::dithering_braille::DitheringBrailleConverter;
use crate::{
    AsciiArt, AsciiArtConverter, AsciiArtConverterError, AsciiArtConverterOptions,
    AsciiStringError, CustomRatioResize, SizeError, DEFAULT_ASCII_STRING, DEFAULT_FONT_RATIO,
};

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
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct AsciiPlayer {}

impl AsciiPlayer {
    #[deprecated(since = "3.1.0", note = "Use `Iterator::rev` instead")]
    /// Reverse ASCII string
    pub fn reverse_ascii_string(ascii_string: String) -> String {
        ascii_string.chars().rev().collect()
    }

    /// Renders frame using [`AsciiPlayerOptions`]
    pub fn render_frame(
        path: &PathBuf,
        options: &AsciiPlayerOptions,
        converter_options: &AsciiArtConverterOptions,
    ) -> Result<AsciiArt, AsciiPlayerError> {
        let img = image::open(path)?;
        let processed_img = match options.threshold {
            Some(threshold) => {
                image::DynamicImage::from(adaptive_threshold(&img.to_luma8(), threshold))
            }
            None => img,
        };

        let prepared_img = processed_img.resize_custom_ratio(
            options.width,
            options.height,
            options.font_ratio,
            options.filter,
        );

        let ascii_art = match (
            options.clone().background_string,
            options.dithering_braille,
            options.braille,
        ) {
            (Some(background_string), _, _) => {
                prepared_img.background_string_art(&background_string, options.colored)?
            }
            (None, true, _) => prepared_img.dithering_braille_art(options.colored)?,
            (None, false, true) => prepared_img.braille_art(options.colored)?,
            (None, false, false) => prepared_img.ascii_art(converter_options)?,
        };

        Ok(ascii_art)
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

                let ascii_art = AsciiPlayer::render_frame(path, options, &converter_options)?;

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

        let progress_bar_style = ProgressStyle::with_template(
            "{elapsed_precise} | {wide_bar} {percent}% | ETA: {eta} | FPS: {per_sec} | {pos}/{len}",
        )
        .unwrap_or_else(|_| ProgressStyle::default_bar());

        let frames = iter
            .progress_with_style(progress_bar_style)
            .map(|path| AsciiPlayer::render_frame(path, options, &converter_options))
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
    /// let paths = vec!["./assets/examples/ferris.webp".into()];
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
#[derive(Debug, Clone, PartialEq)]
pub struct AsciiPlayerOptions {
    /// Resize image into desired width
    pub width: Option<u32>,
    /// Resize image into desired width
    pub height: Option<u32>,
    /// String to represent lightness of pixels
    pub ascii_string: String,
    /// Make ASCII art colored
    pub colored: bool,
    /// Time to sleep after each frame
    pub frame_time: u64,
    /// Render before starting slideshow
    pub pre_render: bool,
    /// Ratio for ASCII characters. Used for auto-resizing
    pub font_ratio: f64,
    /// Repeat after the end of slideshow
    pub looped: bool,
    /// Filter used for resizing
    pub filter: FilterType,
    /// Threshold block radius
    pub threshold: Option<u32>,
    /// Use braille characters for displaying image
    pub braille: bool,
    /// Text to show as background on light pixels
    pub background_string: Option<String>,
    pub dithering_braille: bool,
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
            dithering_braille: false,
            background_string: None,
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
#[derive(Debug)]
pub enum AsciiPlayerError {
    /// Error caused by [`image`] ([`image::ImageError`])
    Image(image::ImageError),
    // TODO: Rename into AsciiArtConverter
    /// Error caused by [`AsciiArtConverter`] ([`AsciiArtConverterError`])
    AsciiConverter(AsciiArtConverterError),
}

impl error::Error for AsciiPlayerError {}

impl fmt::Display for AsciiPlayerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AsciiPlayerError::Image(err) => {
                write!(f, "Image error: {}", err)
            }
            AsciiPlayerError::AsciiConverter(err) => {
                write!(f, "ASCII art converter error: {}", err)
            }
        }
    }
}

impl From<image::ImageError> for AsciiPlayerError {
    fn from(err: image::ImageError) -> AsciiPlayerError {
        AsciiPlayerError::Image(err)
    }
}

impl From<AsciiArtConverterError> for AsciiPlayerError {
    fn from(err: AsciiArtConverterError) -> AsciiPlayerError {
        AsciiPlayerError::AsciiConverter(err)
    }
}

impl From<AsciiStringError> for AsciiPlayerError {
    fn from(err: AsciiStringError) -> AsciiPlayerError {
        AsciiPlayerError::AsciiConverter(AsciiArtConverterError::AsciiStringError(err))
    }
}

impl From<SizeError> for AsciiPlayerError {
    fn from(err: SizeError) -> AsciiPlayerError {
        AsciiPlayerError::AsciiConverter(AsciiArtConverterError::SizeError(err))
    }
}
