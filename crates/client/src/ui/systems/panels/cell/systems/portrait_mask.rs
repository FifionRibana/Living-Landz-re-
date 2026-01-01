use bevy::{prelude::*, render::render_resource::{Extent3d, TextureDimension}};

use crate::ui::components::PendingHexMask;

/// Apply hex mask to unit portraits
pub fn apply_hex_mask_to_portraits(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut pending_query: Query<(Entity, &PendingHexMask, &mut ImageNode)>,
) {
    let pending_count = pending_query.iter().count();
    if pending_count > 0 {
        info!("Processing {} portraits with PendingHexMask", pending_count);
    }

    for (entity, pending, mut image_node) in &mut pending_query {
        // Check if both images are loaded
        let Some(portrait_img) = images.get(&pending.portrait_handle) else {
            // Portrait not loaded yet
            continue;
        };
        let Some(mask_img) = images.get(&pending.mask_handle) else {
            // Mask not loaded yet
            continue;
        };

        info!(
            "Applying hex mask to portrait - portrait size: {:?}, mask size: {:?}",
            portrait_img.texture_descriptor.size, mask_img.texture_descriptor.size
        );

        // Get image data - in Bevy 0.17, data is Option<Vec<u8>>
        let Some(portrait_data) = &portrait_img.data else {
            warn!("Portrait image has no data");
            commands.entity(entity).remove::<PendingHexMask>();
            continue;
        };
        let Some(mask_data) = &mask_img.data else {
            warn!("Mask image has no data");
            commands.entity(entity).remove::<PendingHexMask>();
            continue;
        };

        info!(
            "Portrait data len: {}, Mask data len: {}, Portrait format: {:?}, Mask format: {:?}",
            portrait_data.len(),
            mask_data.len(),
            portrait_img.texture_descriptor.format,
            mask_img.texture_descriptor.format
        );

        // Target size for both images (112x130 as specified)
        const TARGET_WIDTH: u32 = 112;
        const TARGET_HEIGHT: u32 = 130;

        // Resize portrait to 112x130 if needed
        let portrait_resized = if portrait_img.texture_descriptor.size.width != TARGET_WIDTH
            || portrait_img.texture_descriptor.size.height != TARGET_HEIGHT
        {
            info!(
                "Resizing portrait from {}x{} to {}x{}",
                portrait_img.texture_descriptor.size.width,
                portrait_img.texture_descriptor.size.height,
                TARGET_WIDTH,
                TARGET_HEIGHT
            );

            let resized = resize_image_nearest_neighbor(
                portrait_data,
                portrait_img.texture_descriptor.size.width,
                portrait_img.texture_descriptor.size.height,
                TARGET_WIDTH,
                TARGET_HEIGHT,
            );
            resized
        } else {
            portrait_data.clone()
        };

        // Resize mask to 112x130 if needed
        let mask_resized = if mask_img.texture_descriptor.size.width != TARGET_WIDTH
            || mask_img.texture_descriptor.size.height != TARGET_HEIGHT
        {
            info!(
                "Resizing mask from {}x{} to {}x{}",
                mask_img.texture_descriptor.size.width,
                mask_img.texture_descriptor.size.height,
                TARGET_WIDTH,
                TARGET_HEIGHT
            );

            resize_image_nearest_neighbor(
                mask_data,
                mask_img.texture_descriptor.size.width,
                mask_img.texture_descriptor.size.height,
                TARGET_WIDTH,
                TARGET_HEIGHT,
            )
        } else {
            mask_data.clone()
        };

        // Clone the portrait data and apply mask
        let mut masked_data = portrait_resized.clone();

        // Apply mask to portrait (multiply alpha channels)
        // Assuming RGBA format (4 bytes per pixel)
        let mut pixels_modified = 0;
        for (i, (portrait_chunk, mask_chunk)) in portrait_resized
            .chunks_exact(4)
            .zip(mask_resized.chunks_exact(4))
            .enumerate()
        {
            // Get alpha from both images
            let portrait_alpha = portrait_chunk[3] as f32 / 255.0;
            let mask_alpha = mask_chunk[3] as f32 / 255.0;

            // Multiply alpha values
            let new_alpha = (portrait_alpha * mask_alpha * 255.0) as u8;

            if new_alpha != portrait_chunk[3] {
                pixels_modified += 1;
            }

            masked_data[i * 4 + 3] = new_alpha;
        }

        info!(
            "Hex masking complete: {} pixels modified out of {} total pixels",
            pixels_modified,
            masked_data.len() / 4
        );

        // Create a new image with the masked data (always 112x130)
        let masked_image = Image::new(
            Extent3d {
                width: TARGET_WIDTH,
                height: TARGET_HEIGHT,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            masked_data,
            portrait_img.texture_descriptor.format,
            portrait_img.asset_usage,
        );

        // Add the masked image to assets and update the ImageNode
        let masked_handle = images.add(masked_image);
        image_node.image = masked_handle;

        info!("Applied hex mask successfully, updated ImageNode");

        // Remove the PendingHexMask component since masking is done
        commands.entity(entity).remove::<PendingHexMask>();
    }
}


/// Resize an RGBA image using nearest neighbor interpolation
fn resize_image_nearest_neighbor(
    data: &[u8],
    src_width: u32,
    src_height: u32,
    dst_width: u32,
    dst_height: u32,
) -> Vec<u8> {
    let mut result = vec![0u8; (dst_width * dst_height * 4) as usize];

    for dst_y in 0..dst_height {
        for dst_x in 0..dst_width {
            // Map destination pixel to source pixel (nearest neighbor)
            let src_x = ((dst_x as f32 / dst_width as f32) * src_width as f32) as u32;
            let src_y = ((dst_y as f32 / dst_height as f32) * src_height as f32) as u32;

            // Ensure we don't go out of bounds
            let src_x = src_x.min(src_width - 1);
            let src_y = src_y.min(src_height - 1);

            // Calculate pixel indices
            let src_idx = ((src_y * src_width + src_x) * 4) as usize;
            let dst_idx = ((dst_y * dst_width + dst_x) * 4) as usize;

            // Copy RGBA values
            if src_idx + 3 < data.len() && dst_idx + 3 < result.len() {
                result[dst_idx] = data[src_idx]; // R
                result[dst_idx + 1] = data[src_idx + 1]; // G
                result[dst_idx + 2] = data[src_idx + 2]; // B
                result[dst_idx + 3] = data[src_idx + 3]; // A
            }
        }
    }

    result
}