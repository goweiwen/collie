use std::path::Path;

/// Resize an image to the specified width, maintaining aspect ratio
pub fn resize_image(path: &Path, target_width: u32) -> Result<(), Box<dyn std::error::Error>> {
    use fast_image_resize::{Resizer, images::Image};
    use image::ImageReader;

    let img = ImageReader::open(path)?.decode()?;

    // Calculate new height maintaining aspect ratio
    let (width, height) = (img.width(), img.height());

    if width <= target_width {
        // Image is already smaller or equal, no need to resize
        return Ok(());
    }

    let target_height = (height as f64 * target_width as f64 / width as f64) as u32;

    // Convert to RGBA8 for fast_image_resize
    let rgba_img = img.to_rgba8();
    let src_image = Image::from_vec_u8(
        width,
        height,
        rgba_img.into_raw(),
        fast_image_resize::PixelType::U8x4,
    )?;

    // Create destination image
    let mut dst_image = Image::new(
        target_width,
        target_height,
        fast_image_resize::PixelType::U8x4,
    );

    // Create resizer with Lanczos3 filter (default)
    let mut resizer = Resizer::new();
    resizer.resize(&src_image, &mut dst_image, None)?;

    // Convert back to DynamicImage and save
    let resized = image::RgbaImage::from_raw(target_width, target_height, dst_image.into_vec())
        .ok_or("Failed to create image from resized data")?;

    image::DynamicImage::ImageRgba8(resized).save(path)?;

    Ok(())
}
