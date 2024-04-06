//! > **Tool to convert your images into ASCII art**
//!
//! Useful functions, when using as lib
//! - [`AsciiArtConverter::ascii_art`]
//! - [`CustomRatioResize::resize_custom_ratio`]
//!
//! ## Installation
//!
//! ```bash
//! cargo install tapciify
//! ```
//!
//! ## Converting image
//!
//! 1. Run: `tapciify -i imagePath -w imageWidth` for image.
//!
//! 2. Run: `tapciify -i imagePath -w imageWidth -r` for reversed colors.
//!
//! ## Converting video
//!
//! In this example I set framerate to 24 (but you can use any another)
//!
//! > Requires ffmpeg
//!
//! 1. Make frames from video into dir:
//!
//!    ```bash
//!    mkdir frames; ffmpeg -i badapple.mkv frames/%08d.jpeg
//!    ```
//!
//! 2. Run:
//!
//!    ```bash
//!    tapciify -i frames/* -w videoWidth -f 24
//!    ```
//!
//! # Examples
//!
//! Demo:
//!
//! ```
//! use image::imageops::FilterType;
//! # use std::error::Error;
//! use tapciify::{
//!     AsciiArtConverter, AsciiArtConverterOptions, CustomRatioResize, DEFAULT_FONT_RATIO,
//! };
//!
//! # fn main() -> Result<(), Box<dyn Error>> {
//! let img = image::open("./assets/examples/original.webp")?;
//!
//! let result = img
//!     .resize_custom_ratio(Some(64), None, DEFAULT_FONT_RATIO, FilterType::Triangle)
//!     .ascii_art(&AsciiArtConverterOptions {
//!         // Put your other options here
//!         ..Default::default()
//! })?;
//!
//! println!("{}", result);
//! # Ok(())
//! # }
//! ````
//!
//! Colored:
//!
//! ```
//! use image::imageops::FilterType;
//! # use std::error::Error;
//! use tapciify::{
//!     AsciiArtConverter, AsciiArtConverterOptions, CustomRatioResize, DEFAULT_FONT_RATIO,
//! };
//!
//! # fn main() -> Result<(), Box<dyn Error>> {
//! let img = image::open("./assets/examples/original.webp")?;
//!
//! let result = img
//!     .resize_custom_ratio(Some(64), None, DEFAULT_FONT_RATIO, FilterType::Triangle)
//!     .ascii_art(&AsciiArtConverterOptions {
//!         // Put your other options here
//!         colored: true,
//!         ..Default::default()
//!     })?;
//!
//! println!("{}", result);
//! # Ok(())
//! # }
//! ````

pub mod ascii;
pub mod resize;

#[cfg(feature = "player")]
pub mod player;

#[cfg(feature = "braille")]
pub mod braille;

#[cfg(feature = "background_string")]
pub mod background_string;

#[cfg(feature = "threshold_utils")]
pub mod threshold_utils;

#[cfg(feature = "player")]
pub mod cli;

#[allow(deprecated)]
#[doc(inline)]
pub use ascii::{
    ascii_character, AsciiArt, AsciiArtConverter, AsciiArtConverterError, AsciiArtConverterOptions,
    AsciiArtPixel, AsciiStringError, ReverseString, SizeError, ToAsciiArtPixel,
    DEFAULT_ASCII_STRING,
};
#[doc(inline)]
pub use resize::{CustomRatioResize, DEFAULT_FONT_RATIO};

#[doc(inline)]
#[cfg(feature = "player")]
pub use player::{AsciiPlayer, AsciiPlayerError, AsciiPlayerOptions};
