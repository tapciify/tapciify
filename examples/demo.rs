use std::error::Error;
use tapciify::{ratio_resize, AsciiConverter, AsciiConverterOptions, ResizingOptions};

fn main() -> Result<(), Box<dyn Error>> {
    let orig_img = image::open("./assets/examples/original.webp")?;

    let img = ratio_resize(
        &orig_img,
        &ResizingOptions {
            // Put your other options here
            width: Some(64),
            ..Default::default()
        },
    );

    let options = AsciiConverterOptions {
        // Put your other options here
        ..Default::default()
    };

    let result = AsciiConverter::convert(&img, &options)?;

    println!("{}", result.text);

    Ok(())
}
