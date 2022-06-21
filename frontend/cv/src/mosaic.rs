use std::fs::File;
use anyhow::bail;
use glob::glob;
use image::{ColorType, DynamicImage, GenericImageView, imageops, ImageOutputFormat};
use image::imageops::FilterType;
use opencv::*;

pub fn generate_mosaic() -> anyhow::Result<()> {
    let mut images = Vec::new();
    let (grid_width, grid_height) = (4, 2);
    let count = grid_width * grid_height;

    println!("Looking for images");
    glob("**/*.jpg")?.flatten().take(count)
        .filter(|path| !path.file_name().unwrap().to_str().unwrap().contains("result"))
        .for_each(|path| images.push(image::open(path).unwrap()));

    if images.len() != count {
        bail!("Needed {} images, found {}!", count, images.len());
    }

    println!("Rotating images");
    for img in &mut images[0..4] { *img = img.rotate90(); }
    for img in &mut images[4..8] { *img = img.rotate270(); }

    println!("Resizing images");
    images.iter_mut().for_each(|image| *image = image.resize(1280, 720, FilterType::Lanczos3));

    let (big_x, big_y) = images.iter().map(|it| it.dimensions()).max().unwrap();
    println!("Max dimensions are ({}, {})", big_x, big_y);

    let extra_space = 10;

    let (image_width, image_height) = ((big_x + extra_space) * grid_width as u32 + extra_space, (big_y + extra_space) * grid_height as u32 + extra_space);
    println!("Mosaic dimensions: ({}, {})", image_width, image_height);

    let mut output_image = DynamicImage::new_rgba8(image_width, image_height);

    println!("Processing images");

    let mut iter = images.iter();
    for y in 0..grid_height {
        for x in 0..grid_width {
            if let Some(image) = iter.next() {
                let (width, height) = image.dimensions();
                let (center_x, center_y) = (width as u32 / 2, height as u32 / 2);
                let (actual_center_x, actual_center_y) = ((big_x + extra_space) * x as u32 + extra_space + big_x / 2, (big_y + extra_space) * y as u32 + extra_space + big_y / 2);
                let (pos_x, pos_y) = (actual_center_x - center_x, actual_center_y - center_y);

                imageops::overlay(&mut output_image, image, pos_x as i64, pos_y as i64)
            }
        }
    }

    output_image = output_image.resize(1600, 900, FilterType::Lanczos3);

    println!("Writing mosaic");
    image::write_buffer_with_format(&mut File::create("result.png")?, output_image.as_rgba8().unwrap(), output_image.width(), output_image.height(), ColorType::Rgba8, ImageOutputFormat::Jpeg(85))?;
    let image = imgcodecs::imread("result.png", imgcodecs::IMREAD_COLOR)?;
    highgui::imshow("mosaic", &image)?;
    highgui::wait_key(0)?;

    println!("Complete!");

    Ok(())
}
