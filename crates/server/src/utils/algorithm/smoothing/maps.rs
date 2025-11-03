use image::{ImageBuffer, Luma};

pub fn open_binary_map(
    binary_map: &ImageBuffer<Luma<u8>, Vec<u8>>,
    radius: u32,
) -> ImageBuffer<Luma<u8>, Vec<u8>> {
    let eroded = erode_binary_map(binary_map, radius);
    dilate_binary_map(&eroded, radius + 2)
}

pub fn erode_binary_map(
    binary_map: &ImageBuffer<Luma<u8>, Vec<u8>>,
    radius: u32,
) -> ImageBuffer<Luma<u8>, Vec<u8>> {
    let mut result = ImageBuffer::new(binary_map.width(), binary_map.height());
    let width = binary_map.width();
    let height = binary_map.height();

    const THRESHOLD: u8 = ((u8::MAX - u8::MIN) as f32 / 2.).ceil() as u8;
    for y in radius..height - radius {
        for x in radius..width - radius {
            let mut is_white = true;

            // ✨ Vérifier uniquement les 4 directions cardinales (croix)
            for r in 1..=radius {
                if binary_map.get_pixel(x, y + r)[0] < THRESHOLD ||      // Bas
                   binary_map.get_pixel(x, (y as i32 - r as i32) as u32)[0] < THRESHOLD || // Haut
                   binary_map.get_pixel(x + r, y)[0] < THRESHOLD ||      // Droite
                   binary_map.get_pixel((x as i32 - r as i32) as u32, y)[0] < THRESHOLD
                {
                    // Gauche
                    is_white = false;
                    break;
                }
            }

            result.put_pixel(x, y, Luma(if is_white { [u8::MAX] } else { [u8::MIN] }));
        }
    }
    result
}

pub fn dilate_binary_map(
    binary_map: &ImageBuffer<Luma<u8>, Vec<u8>>,
    radius: u32,
) -> ImageBuffer<Luma<u8>, Vec<u8>> {
    let mut result = binary_map.clone();
    let width = binary_map.width();
    let height = binary_map.height();

    for y in 0..height {
        for x in 0..width {
            if binary_map.get_pixel(x, y)[0] == 255 {
                for dy in -(radius as i32)..=(radius as i32) {
                    for dx in -(radius as i32)..=(radius as i32) {
                        let nx = (x as i32 + dx) as u32;
                        let ny = (y as i32 + dy) as u32;
                        if nx < width && ny < height {
                            result.put_pixel(nx, ny, Luma([255]));
                        }
                    }
                }
            }
        }
    }
    result
}

pub fn mask_luma_map(
    source_image: &mut ImageBuffer<Luma<u8>, Vec<u8>>,
    mask_image: &ImageBuffer<Luma<u8>, Vec<u8>>,
) {
    for (x, y, pixel) in source_image.enumerate_pixels_mut() {
        if x < mask_image.width() && y < mask_image.height() {
            let in_mask = mask_image.get_pixel(x, y)[0] > 30;
            if !in_mask {
                *pixel = Luma([u8::MIN]);
            }
        }
    }
}
