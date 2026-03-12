use bevy::{
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension},
};

use crate::ui::components::PendingHexMask;
use crate::ui::components::PendingLayerComposition;

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

/// Compose 4 portrait layers + apply hex mask for lord portraits.
/// Runs every frame, polling until all images are loaded.
pub fn compose_portrait_layers(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut query: Query<(Entity, &PendingLayerComposition, &mut ImageNode)>,
) {
    for (entity, pending, mut image_node) in &mut query {
        // Check if ALL layer images + mask are loaded
        let all_loaded = pending
            .layer_handles
            .iter()
            .all(|h| images.get(h).is_some())
            && pending
                .mask_handle
                .as_ref()
                .map_or(true, |h| images.get(h).is_some());

        if !all_loaded {
            continue;
        }

        const TARGET_WIDTH: u32 = 112;
        const TARGET_HEIGHT: u32 = 130;

        // Start with a transparent base
        let mut composed = vec![0u8; (TARGET_WIDTH * TARGET_HEIGHT * 4) as usize];

        // Composite each layer (bottom to top: bust, face, clothes, hair)
        for handle in &pending.layer_handles {
            let layer_img = images.get(handle).unwrap();
            let Some(layer_data) = &layer_img.data else {
                continue;
            };

            let src_w = layer_img.texture_descriptor.size.width;
            let src_h = layer_img.texture_descriptor.size.height;

            // Resize layer to target size if needed
            let resized = if src_w != TARGET_WIDTH || src_h != TARGET_HEIGHT {
                resize_image_nearest_neighbor(layer_data, src_w, src_h, TARGET_WIDTH, TARGET_HEIGHT)
            } else {
                layer_data.clone()
            };

            // Alpha composite: layer over composed (premultiplied-like blending)
            for (dst_chunk, src_chunk) in composed.chunks_exact_mut(4).zip(resized.chunks_exact(4))
            {
                let src_a = src_chunk[3] as f32 / 255.0;
                if src_a < 0.001 {
                    continue; // fully transparent pixel, skip
                }

                let dst_a = dst_chunk[3] as f32 / 255.0;
                let out_a = src_a + dst_a * (1.0 - src_a);

                if out_a > 0.001 {
                    for c in 0..3 {
                        let src_c = src_chunk[c] as f32 / 255.0;
                        let dst_c = dst_chunk[c] as f32 / 255.0;
                        let out_c = (src_c * src_a + dst_c * dst_a * (1.0 - src_a)) / out_a;
                        dst_chunk[c] = (out_c * 255.0).min(255.0) as u8;
                    }
                    dst_chunk[3] = (out_a * 255.0).min(255.0) as u8;
                }
            }
        }

        // Apply hex mask
        if let Some(ref mask_handle) = pending.mask_handle
            && let Some(mask_img) = images.get(mask_handle)
        {
            if let Some(mask_data) = &mask_img.data {
                let mask_w = mask_img.texture_descriptor.size.width;
                let mask_h = mask_img.texture_descriptor.size.height;

                let mask_resized = if mask_w != TARGET_WIDTH || mask_h != TARGET_HEIGHT {
                    resize_image_nearest_neighbor(
                        mask_data,
                        mask_w,
                        mask_h,
                        TARGET_WIDTH,
                        TARGET_HEIGHT,
                    )
                } else {
                    mask_data.clone()
                };

                for (composed_chunk, mask_chunk) in composed
                    .chunks_exact_mut(4)
                    .zip(mask_resized.chunks_exact(4))
                {
                    let mask_alpha = mask_chunk[3] as f32 / 255.0;
                    let current_alpha = composed_chunk[3] as f32 / 255.0;
                    composed_chunk[3] = (current_alpha * mask_alpha * 255.0) as u8;
                }
            }
        }

        // Create the final image
        let composed_image = Image::new(
            Extent3d {
                width: TARGET_WIDTH,
                height: TARGET_HEIGHT,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            composed,
            bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
            Default::default(),
        );

        let composed_handle = images.add(composed_image);
        image_node.image = composed_handle;
        image_node.color = Color::WHITE; // Make visible

        info!("✓ Composed lord portrait layers + hex mask");

        commands.entity(entity).remove::<PendingLayerComposition>();
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
