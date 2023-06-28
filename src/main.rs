use std::path::Path;

use clap::{arg, Command};
use image::{imageops::FilterType::Lanczos3, DynamicImage, GenericImageView, ImageBuffer, Rgb};

fn hide_image(
    source_image: &DynamicImage,
    secret_image: &DynamicImage,
    resize: bool,
    expand: bool,
) -> DynamicImage {
    let (source_width, source_height) = source_image.dimensions();
    let (secret_width, secret_height) = secret_image.dimensions();

    let (resized_source_image, resized_secret_image) =
        if source_image.dimensions() < secret_image.dimensions() {
            if resize {
                (
                    source_image.resize_exact(secret_width, secret_height, Lanczos3),
                    secret_image.clone(),
                )
            } else if expand {
                (
                    expand_image(source_image, secret_width, secret_height),
                    secret_image.clone(),
                )
            } else {
                (source_image.clone(), secret_image.clone())
            }
        } else {
            if resize {
                (
                    source_image.clone(),
                    secret_image.resize_exact(source_width, source_height, Lanczos3),
                )
            } else if expand {
                (
                    source_image.clone(),
                    expand_image(secret_image, source_width, source_height),
                )
            } else {
                (source_image.clone(), secret_image.clone())
            }
        };

    let source_buffer = resized_source_image.to_rgb8();
    let secret_buffer = resized_secret_image.to_rgb8();

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

    for byte in &text_len_bytes {
        for bit in 0..8 {
            let pixel = hidden_image.get_pixel_mut(x, y);
            let old_value = pixel[0];
            let new_value = (old_value & 0xFE) | ((byte >> (7 - bit)) & 1);
            pixel[0] = new_value;
            x += 1;
            if x >= width {
                x = 0;
                y += 1;
            }
        }
    }

    for byte in text.bytes() {
        for bit in 0..8 {
            let pixel = hidden_image.get_pixel_mut(x, y);
            let old_value = pixel[0];
            let new_value = (old_value & 0xFE) | ((byte >> (7 - bit)) & 1);
            pixel[0] = new_value;
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

    let mut x = 0;
    let mut y = 0;

    let mut text_len_bytes = [0u8; 4];
    for byte in &mut text_len_bytes {
        let mut extracted_byte = 0u8;
        for _ in 0..8 {
            let pixel = hidden_image.get_pixel(x, y);
            let lsb = pixel[0] & 1;
            extracted_byte = (extracted_byte << 1) | lsb;
            x += 1;
            if x >= width {
                x = 0;
                y += 1;
            }
        }
        *byte = extracted_byte;
    }

    let text_len = u32::from_be_bytes(text_len_bytes) as usize;

    for _ in 0..text_len {
        let mut extracted_byte = 0u8;
        for _ in 0..8 {
            let pixel = hidden_image.get_pixel(x, y);
            let lsb = pixel[0] & 1;
            extracted_byte = (extracted_byte << 1) | lsb;
            x += 1;
            if x >= width {
                x = 0;
                y += 1;
            }
        }
        extracted_text.push(extracted_byte as char);
    }

    extracted_text
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

fn expand_image(
    source_image: &DynamicImage,
    target_width: u32,
    target_height: u32,
) -> DynamicImage {
    let (source_width, source_height) = source_image.dimensions();

    let source_buffer = source_image.to_rgb8();
    let mut expanded_buffer = ImageBuffer::new(target_width, target_height);

    for (x, y, pixel) in expanded_buffer.enumerate_pixels_mut() {
        if x < source_width && y < source_height {
            let source_pixel = source_buffer.get_pixel(x, y);
            *pixel = *source_pixel;
        } else {
            *pixel = Rgb([0, 0, 0]);
        }
    }

    DynamicImage::ImageRgb8(expanded_buffer)
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
                .arg(arg!(--resize "Resizes the image"))
                .arg(arg!(--expand "Expands the image"))
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

            let resize = sub_matches.get_flag("resize");
            let expand = sub_matches.get_flag("expand");

            let source_image =
                image::open(&Path::new(&source)).expect("Failed to open source image");
            let secret_image =
                image::open(&Path::new(&secret)).expect("Failed to open secret image");

            let normalized_image = normalize_image(&source_image);
            let hidden_image = hide_image(&normalized_image, &secret_image, resize, expand);

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
