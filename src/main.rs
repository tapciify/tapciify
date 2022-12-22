pub mod utils;

use clap::Parser;
use crossterm::cursor::MoveUp;
use crossterm::execute;
use std::io::stdout;
use std::{fs, thread, time};
use utils::{calc_new_height, render_frame};

use crate::utils::print_colored_frame;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Arguments {
    #[clap(short, short, value_parser)]
    input: String,
    #[clap(short, short, value_parser)]
    width: u32,
    #[clap(short, long, action)]
    dir: bool,
    #[clap(short, long, action)]
    prerender: bool,
    #[clap(short, long)]
    fps: Option<f64>,
    #[clap(short, long, action)]
    reverse: bool,
    #[clap(short, long, action)]
    colored: bool,
}

fn main() {
    let args = Arguments::parse();
    let mut ascii_string = " .,:;+*?%S#@".to_string();

    if args.reverse {
        ascii_string = ascii_string.chars().rev().collect::<String>().to_owned();
    }

    if args.colored {
        if args.dir {
            let mut image_paths: Vec<String> = Vec::new();

            let images_paths = fs::read_dir(args.input).unwrap();
            for image_path in images_paths {
                match image_path.unwrap().path().to_str() {
                    Some(x) => image_paths.push(x.to_string()),
                    None => println!(),
                }
            }

            let frametime: u64;

            if let Some(fps) = args.fps {
                frametime = (1f64 / fps * 1000f64) as u64;
            } else {
                panic!("Frametime is None");
            }

            let mut image;
            let mut height;

            for image_path in image_paths {
                image = image::open(image_path).unwrap().to_rgb8();
                height = calc_new_height(args.width, image.width(), image.height());
                print_colored_frame(image.clone(), args.width, height, &ascii_string);

                execute!(stdout(), MoveUp(height as u16 + 1)).expect("");

                thread::sleep(time::Duration::from_millis(frametime));
            }
        } else {
            let image = image::open(args.input).unwrap().to_rgb8();
            let height = calc_new_height(args.width, image.width(), image.height());

            print_colored_frame(image.clone(), args.width, height, &ascii_string);
        }
    } else {
        if args.dir {
            let mut image_paths: Vec<String> = Vec::new();

            let images_paths = fs::read_dir(args.input).unwrap();
            for image_path in images_paths {
                match image_path.unwrap().path().to_str() {
                    Some(x) => image_paths.push(x.to_string()),
                    None => println!(),
                }
            }

            let frametime: u64;

            if let Some(fps) = args.fps {
                frametime = (1f64 / fps * 1000f64) as u64;
            } else {
                panic!("Frametime is None");
            }

            match args.prerender {
                true => {
                    let mut image;
                    let mut height: Option<u32> = None;

                    let mut frames: Vec<String> = Vec::new();

                    for image_path in image_paths {
                        image = image::open(image_path.clone()).unwrap().to_rgb8();
                        frames.push(render_frame(
                            image.clone(),
                            args.width,
                            calc_new_height(args.width, image.width(), image.height()),
                            &ascii_string,
                        ));

                        height = Some(calc_new_height(args.width, image.width(), image.height()));

                        println!("Rendered {}", image_path);
                    }

                    for frame in frames {
                        println!("{}", frame);
                        if let Some(h) = height {
                            execute!(stdout(), MoveUp(h as u16 + 1)).expect("");
                        }

                        thread::sleep(time::Duration::from_millis(frametime));
                    }
                }
                false => {
                    let mut image;
                    let mut height;

                    for image_path in image_paths {
                        image = image::open(image_path).unwrap().to_rgb8();
                        height = calc_new_height(args.width, image.width(), image.height());
                        println!(
                            "{}",
                            render_frame(image.clone(), args.width, height, &ascii_string)
                        );
                        execute!(stdout(), MoveUp(height as u16 + 1)).expect("");

                        thread::sleep(time::Duration::from_millis(frametime));
                    }
                }
            }
        } else {
            let image = image::open(args.input).unwrap().to_rgb8();
            let height = calc_new_height(args.width, image.width(), image.height());

            println!(
                "{}",
                render_frame(image.clone(), args.width, height, &ascii_string)
            )
        }
    }
}
