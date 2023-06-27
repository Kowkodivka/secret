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

fn hide_text_in_image(image: &DynamicImage, text: &str) -> DynamicImage {
    let (width, height) = image.dimensions();
    let mut hidden_image = image.to_rgb8();

    let required_pixels = (text.len() + 4) * 8;

    if required_pixels > (width * height).try_into().unwrap() {
        panic!("Insufficient space in the image to hide the text.");
    }

    let text_len = text.len() as u32;
    let text_len_bytes = text_len.to_be_bytes();

    let mut x = 0;
    let mut y = 0;

    for i in 0..4 {
        let pixel = hidden_image.get_pixel_mut(x, y);
        pixel[0] = (pixel[0] & 0xFC) | (text_len_bytes[i] >> 6);
        x += 1;
        if x >= width {
            x = 0;
            y += 1;
        }
    }

    for c in text.chars() {
        let char_byte = c as u8;
        for i in 0..8 {
            let pixel = hidden_image.get_pixel_mut(x, y);
            pixel[0] = (pixel[0] & 0xFE) | ((char_byte >> i) & 0x01);
            x += 1;
            if x >= width {
                x = 0;
                y += 1;
            }
        }
    }

    DynamicImage::ImageRgb8(hidden_image)
}

fn extract_text_from_image(image: &DynamicImage) -> String {
    let (width, height) = image.dimensions();
    let hidden_image = image.to_rgb8();
    let available_pixels = width * height;

    if available_pixels < 32 {
        panic!("The image is too small to contain the text length and the text itself.");
    }

    let mut extracted_text = String::new();

    let mut text_len_bytes = [0u8; 4];

    let mut x = 0;
    let mut y = 0;

    for i in 0..4 {
        let pixel = hidden_image.get_pixel(x, y);
        text_len_bytes[i] = (pixel[0] & 0x03) << 6;
        x += 1;
        if x >= width {
            x = 0;
            y += 1;
        }
    }

    let text_len = u32::from_be_bytes(text_len_bytes);

    for _ in 0..text_len {
        let mut char_byte = 0u8;
        for i in 0..8 {
            let pixel = hidden_image.get_pixel(x, y);
            char_byte |= (pixel[0] & 0x01) << i;
            x += 1;
            if x >= width {
                x = 0;
                y += 1;
            }
        }
        extracted_text.push(char_byte as char);
    }

    extracted_text
}

fn main() {
    let matches = Command::new("secret")
        .version("1.0")
        .author("lucin")
        .about("Hides and decrypts images")
        .arg_required_else_help(true)
        .subcommand(
            Command::new("hide_img")
                .about("Hides image")
                .arg(arg!(--source <SOURCE>))
                .arg(arg!(--secret <SECRET>))
                .arg(arg!(--output <OUTPUT>))
                .arg_required_else_help(true),
        )
        .subcommand(
            Command::new("decrypt_img")
                .about("Decrypts image")
                .arg(arg!(--source <SOURCE>))
                .arg(arg!(--output <OUTPUT>))
                .arg_required_else_help(true),
        )
        .subcommand(
            Command::new("hide_txt")
                .about("Hides text in an image")
                .arg(arg!(--image <IMAGE>))
                .arg(arg!(--output <OUTPUT>))
                .arg(arg!(--text <TEXT>...))
                .arg_required_else_help(true),
        )
        .subcommand(
            Command::new("decrypt_txt")
                .about("Decrypts text from an image")
                .arg(arg!(--image <IMAGE>))
                .arg_required_else_help(true),
        )
        .get_matches();

    match matches.subcommand() {
        Some(("hide_img", sub_matches)) => {
            let source = sub_matches.get_one::<String>("source").unwrap();
            let secret = sub_matches.get_one::<String>("secret").unwrap();
            let output = sub_matches.get_one::<String>("output").unwrap();

            let source_image =
                image::open(&Path::new(&source)).expect("Failed to open source image");
            let secret_image =
                image::open(&Path::new(&secret)).expect("Failed to open secret image");

            let normalized_image = normalize_image(&source_image);
            let hidden_image = hide_image(&normalized_image, &secret_image);

            hidden_image
                .save(&Path::new(&output))
                .expect("Failed to save hidden image");

            println!("Image hidden successfully");
        }
        Some(("decrypt_img", sub_matches)) => {
            let source = sub_matches.get_one::<String>("source").unwrap();
            let output = sub_matches.get_one::<String>("output").unwrap();

            let hidden_image =
                image::open(&Path::new(&source)).expect("Failed to open hidden image");

            let decrypted_image = decrypt_image(&hidden_image);
            decrypted_image
                .save(&Path::new(&output))
                .expect("Failed to save decrypted image");

            println!("Image decrypted successfully");
        }
        Some(("hide_txt", sub_matches)) => {
            let image_path = sub_matches.get_one::<String>("image").unwrap();
            let text = sub_matches.get_one::<String>("text").unwrap();
            let output_path = sub_matches.get_one::<String>("output").unwrap();

            let image = image::open(&Path::new(&image_path)).expect("Failed to open image");

            let hidden_image = hide_text_in_image(&image, &text);

            hidden_image
                .save(&Path::new(&output_path))
                .expect("Failed to save hidden image");

            println!("Text hidden successfully");
        }
        Some(("decrypt_txt", sub_matches)) => {
            let image_path = sub_matches.get_one::<String>("image").unwrap();
            let image = image::open(&Path::new(&image_path)).expect("Failed to open image");
            let extracted_text = extract_text_from_image(&image);

            println!("Extracted Text: {}", extracted_text);
        }
        _ => unreachable!(),
    }
}
