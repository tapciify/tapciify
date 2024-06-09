#[cfg(not(target_family = "windows"))]
use std::path::PathBuf;

use clap::{error::ErrorKind, CommandFactory, Parser};

use tapciify::braille::DEFAULT_BRAILLE_FONT_RATIO;
use tapciify::cli::Cli;
use tapciify::player::{calculate_frame_time, AsciiPlayer, AsciiPlayerOptions};
use tapciify::DEFAULT_FONT_RATIO;

fn main() {
    let cli = Cli::parse();
    let mut cmd = Cli::command();

    #[cfg(target_family = "windows")]
    let images_paths = tapciify::cli::glob_to_paths(&cli.input)
        .unwrap_or_else(|err| cmd.error(ErrorKind::InvalidValue, err).exit());
    #[cfg(not(target_family = "windows"))]
    let images_paths: Vec<PathBuf> = cli.input.into_iter().map(PathBuf::from).collect();

    let font_ratio = match (cli.font_ratio, cli.braille) {
        (Some(ratio), _) => ratio,
        (None, true) => DEFAULT_BRAILLE_FONT_RATIO,
        (None, false) => DEFAULT_FONT_RATIO,
    };

    let (ascii_string, colored) = match (cli.reverse, cli.pixels) {
        (true, false) => (cli.ascii_string.chars().rev().collect(), cli.colored),
        (false, false) => (cli.ascii_string, cli.colored),
        (_, true) => ("█".to_owned(), true),
    };

    let frame_time = calculate_frame_time(cli.framerate);
    let options = AsciiPlayerOptions {
        width: cli.width,
        height: cli.height,
        ascii_string,
        colored,
        frame_time,
        pre_render: cli.pre_render,
        font_ratio,
        looped: cli.looped,
        threshold: cli.threshold,
        braille: cli.braille,
        background_string: cli.background_string,
        dithering_braille: cli.dithering_braille,
        ..Default::default()
    };

    if let Err(err) = AsciiPlayer::play(&images_paths, &options) {
        cmd.error(ErrorKind::Io, err).exit()
    }
}
