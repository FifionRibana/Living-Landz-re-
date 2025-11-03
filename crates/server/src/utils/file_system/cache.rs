use std::fs::File;
use std::io::{Read, Write};

pub fn save_to_disk<P>(image: &image::ImageBuffer<P, Vec<u8>>, path: &str) -> std::io::Result<()>
where
    P: image::Pixel<Subpixel = u8>,
{
    let mut file = File::create(path)?;

    // Sauvegarder dimensions + données
    file.write_all(&image.width().to_le_bytes())?;
    file.write_all(&image.height().to_le_bytes())?;
    file.write_all(&image)?;

    Ok(())
}

pub fn load_from_disk<P>(path: &str) -> std::io::Result<image::ImageBuffer<P, Vec<u8>>>
where
    P: image::Pixel<Subpixel = u8>,
{
    let mut file = File::open(path)?;
    let mut buf = [0u8; 4];

    file.read_exact(&mut buf)?;
    let width = u32::from_le_bytes(buf);

    file.read_exact(&mut buf)?;
    let height = u32::from_le_bytes(buf);

    let bytes_per_pixel = std::mem::size_of::<P>();
    let total_bytes = (width * height) as usize * bytes_per_pixel;

    let mut pixels = vec![0u8; total_bytes];
    file.read_exact(&mut pixels)?;

    Ok(image::ImageBuffer::from_vec(width, height, pixels).unwrap())
}
