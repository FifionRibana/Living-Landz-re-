use bevy::prelude::*;

use crate::ui::{
    carousel::components::{Carousel, CarouselItem},
    frosted_glass::FrostedGlassMaterial,
};

pub fn handle_carousel_scroll(
    mut mouse_wheel_events: MessageReader<bevy::input::mouse::MouseWheel>,
    mut query: Query<&mut Carousel>,
    time: Res<Time>,
) {
    let Ok(mut carousel) = query.single_mut() else {
        return;
    };

    for event in mouse_wheel_events.read() {
        // We adjust sensibility here (20px per mouse wheel tick)
        carousel.scroll_offset += event.x * 20.0;
        info!("scroll event: {}", carousel.scroll_offset);
    }
}

pub fn update_carousel_items(
    carousel_query: Query<(&Carousel, &ComputedNode)>,
    // On ajoute 'Children' pour pouvoir modifier l'opacité du contenu
    mut items_query: Query<
        (
            Entity,
            &CarouselItem,
            &mut Node,
            &MaterialNode<FrostedGlassMaterial>,
            Option<&Children>,
        ),
        Without<Carousel>,
    >,
    mut materials: ResMut<Assets<FrostedGlassMaterial>>,
    // On requête les Textes et les BackgroundColors des enfants
    mut text_query: Query<&mut TextColor>,
    mut ui_color_query: Query<&mut BackgroundColor>,
) {
    let Ok((carousel, computed_node)) = carousel_query.single() else {
        return;
    };

    let container_width = computed_node.size().x;
    if container_width <= 0.0 {
        return;
    }

    let slot_width = carousel.item_width + carousel.spacing;
    let total_content_width = slot_width * carousel.total_items as f32;
    let centering_offset = (container_width / 2.0) - (carousel.item_width / 2.0);

    // POINT DE TÉLÉPORTATION
    let teleport_boundary = total_content_width / 2.0;

    for (entity, item, mut node, material_handle, children) in items_query.iter_mut() {
        let base_x = item.index as f32 * slot_width;
        let mut x_pos = (base_x + carousel.scroll_offset) % total_content_width;

        if x_pos < 0.0 {
            x_pos += total_content_width;
        }
        if x_pos > teleport_boundary {
            x_pos -= total_content_width;
        }

        node.left = Val::Px(x_pos + centering_offset);
        node.position_type = PositionType::Absolute;

        if let Some(material) = materials.get_mut(material_handle) {
            let dist = x_pos.abs();

            // TODO: Adjust that only the last card has its edge fading
            // On commence à disparaître à 30% de la largeur,
            // et on est à zéro à la limite de téléportation
            let fade_start = teleport_boundary * 0.6;
            let fade_end = teleport_boundary; // Opacité 0 ici

            // Calcul de l'intensité (0.0 au centre, 1.0 au bord)
            let intensity = ((dist - fade_start) / (fade_end - fade_start)).clamp(0.0, 1.0);

            // 1. Correction de la règle d'intensité selon ta demande
            if x_pos < -fade_start {
                material.uniforms.edge_fade = -intensity;
            } else if x_pos > fade_start {
                material.uniforms.edge_fade = intensity;
            } else {
                material.uniforms.edge_fade = 0.0;
            }

            // 2. DISPARITION TOTALE DU VERRE
            // On multiplie l'opacité de base par (1.0 - intensity)
            let visibility = 1.0 - intensity;
            // Note: Il est préférable de stocker les opacités max (ex: 0.3 et 0.85)
            // quelque part, ici on utilise des constantes pour l'exemple
            material.uniforms.opacity_top = 0.3 * visibility;
            material.uniforms.opacity_bottom = 0.85 * visibility;

            // 3. DISPARITION DU CONTENU (Enfants)
            if let Some(children) = children {
                for child in children.iter() {
                    // Si c'est du texte
                    if let Ok(mut text_color) = text_query.get_mut(child) {
                        text_color.0.set_alpha(visibility);
                    }
                    // // Si c'est un fond ou une icône
                    // if let Ok(mut bg_color) = ui_color_query.get_mut(child) {
                    //     let current_alpha = bg_color.0.alpha();
                    //     bg_color.0.set_alpha(visibility);
                    // }
                }
            }
        }
    }
}
