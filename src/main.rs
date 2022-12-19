pub mod utils;

use clap::Parser;
use crossterm::{cursor::MoveTo, execute};
use std::io::stdout;
use std::{fs, thread, time};
use utils::render_frame;

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
}

fn main() {
    let args = Arguments::parse();
    let mut ascii_string = " .,:;+*?%S#@".to_string();

    if args.reverse {
        ascii_string = ascii_string.chars().rev().collect::<String>().to_owned();
    }

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
                let mut frames: Vec<String> = Vec::new();

                for image_path in image_paths {
                    frames.push(render_frame(image_path.clone(), args.width, &ascii_string));

                    println!("Rendered {}", image_path);
                }

                for frame in frames {
                    println!("{}", render_frame(frame, args.width, &ascii_string));

                    execute!(stdout(), MoveTo(0, 0)).expect("");

                    thread::sleep(time::Duration::from_millis(frametime));
                }
            }
            false => {
                for image_path in image_paths {
                    println!("{}", render_frame(image_path, args.width, &ascii_string));

                    execute!(stdout(), MoveTo(0, 0)).expect("");

                    thread::sleep(time::Duration::from_millis(frametime));
                }
            }
        }
    } else {
        println!("{}", render_frame(args.input, args.width, &ascii_string))
    }
}
