use std::path::Path;

use clap::{arg, Command};
use image::{DynamicImage, GenericImageView, ImageBuffer, Rgb};

fn hide_image(source_image: &DynamicImage, secret_image: &DynamicImage) -> DynamicImage {
    let (source_width, source_height) = source_image.dimensions();
    let (secret_width, secret_height) = secret_image.dimensions();

    if source_width < secret_width || source_height < secret_height {
        panic!("The size of the secret image exceeds the size of the original image");
    }

    let source_buffer = source_image.to_rgb8();
    let secret_buffer = secret_image.to_rgb8();

    let mut hidden_buffer = ImageBuffer::new(source_width, source_height);

    for (x, y, source_pixel) in source_buffer.enumerate_pixels() {
        let mut hidden_pixel = Rgb([0u8; 3]);

        let secret_pixel = secret_buffer.get_pixel(x, y);

        for i in 0..3 {
            let source_value = source_pixel[i];
            let secret_value = secret_pixel[i];
            let hidden_value = (source_value & 0xFC) | (secret_value >> 6);

            hidden_pixel[i] = hidden_value;
        }

        hidden_buffer.put_pixel(x, y, hidden_pixel);
    }

    DynamicImage::ImageRgb8(hidden_buffer)
}

fn normalize_image(hidden_image: &DynamicImage) -> DynamicImage {
    let hidden_buffer = hidden_image.to_rgb8();
    let mut normalized_buffer = ImageBuffer::new(hidden_buffer.width(), hidden_buffer.height());

    let mut min_value = 255u8;
    let mut max_value = 0u8;

    for (_, _, pixel) in hidden_buffer.enumerate_pixels() {
        for i in 0..3 {
            let value = pixel[i];
            min_value = min_value.min(value);
            max_value = max_value.max(value);
        }
    }

    for (x, y, pixel) in hidden_buffer.enumerate_pixels() {
        let mut normalized_pixel = Rgb([0u8; 3]);

        for i in 0..3 {
            let value = pixel[i];
            let normalized_value =
                ((value - min_value) as f32 / (max_value - min_value) as f32 * 255.0) as u8;
            normalized_pixel[i] = normalized_value;
        }

        normalized_buffer.put_pixel(x, y, normalized_pixel);
    }

    DynamicImage::ImageRgb8(normalized_buffer)
}

fn decrypt_image(hidden_image: &DynamicImage) -> DynamicImage {
    let hidden_buffer = hidden_image.to_rgb8();
    let mut decrypted_buffer = ImageBuffer::new(hidden_buffer.width(), hidden_buffer.height());

    for (x, y, hidden_pixel) in hidden_buffer.enumerate_pixels() {
        let mut decrypted_pixel = Rgb([0u8; 3]);

        for i in 0..3 {
            let hidden_value = hidden_pixel[i];

            let secret_value = hidden_value & 0x03;

            decrypted_pixel[i] = secret_value * 85;
        }

        decrypted_buffer.put_pixel(x, y, decrypted_pixel);
    }

    DynamicImage::ImageRgb8(decrypted_buffer)
}

fn main() {
    let matches = Command::new("limage")
        .version("1.0")
        .author("lucin")
        .about("Hide and decrypt images")
        .arg_required_else_help(true)
        .subcommand(
            Command::new("hide")
                .about("Hides image")
                .arg(arg!(source: <SOURCE>))
                .arg(arg!(secret: <SECRET>))
                .arg(arg!(output: <OUTPUT>))
                .arg_required_else_help(true),
        )
        .subcommand(
            Command::new("decrypt")
                .about("Decrypts image")
                .arg(arg!(source: <SOURCE>))
                .arg(arg!(output: <OUTPUT>))
                .arg_required_else_help(true),
        )
        .get_matches();

    match matches.subcommand() {
        Some(("hide", sub_matches)) => {
            let source = sub_matches.get_one::<String>("source").unwrap();
            let secret = sub_matches.get_one::<String>("secret").unwrap();
            let output = sub_matches.get_one::<String>("output").unwrap();

            let source_image =
                image::open(&Path::new(&source)).expect("Failed to open source image");
            let secret_image =
                image::open(&Path::new(&secret)).expect("Failed to open secret image");

            let hidden_image = hide_image(&source_image, &secret_image);
            let normalized_image = normalize_image(&hidden_image);

            normalized_image
                .save(&Path::new(&output))
                .expect("Failed to save hidden image");

            println!("Image hidden successfully!");
        }
        Some(("decrypt", sub_matches)) => {
            let source = sub_matches.get_one::<String>("source").unwrap();
            let output = sub_matches.get_one::<String>("output").unwrap();

            let hidden_image =
                image::open(&Path::new(&source)).expect("Failed to open hidden image");

            let decrypted_image = decrypt_image(&hidden_image);
            decrypted_image
                .save(&Path::new(&output))
                .expect("Failed to save decrypted image");

            println!("Image decrypted successfully!");
        }
        _ => unreachable!(),
    }
}
