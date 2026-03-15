use bevy::prelude::*;

use crate::ui::{
    carousel::components::{Carousel, CarouselAlpha, CarouselItem},
    frosted_glass::{FrostedGlassMaterial, WipeMaterial},
};

pub fn handle_carousel_scroll(
    mut mouse_wheel_events: MessageReader<bevy::input::mouse::MouseWheel>,
    mut query: Query<&mut Carousel>,
) {
    for event in mouse_wheel_events.read() {
        // Use both axes: x for trackpad, y for mouse wheel
        let delta = if event.x.abs() > event.y.abs() {
            event.x
        } else {
            -event.y
        };
        // let delta = event.x;

        if delta.abs() < 0.001 {
            continue;
        }

        for mut carousel in query.iter_mut() {
            carousel.target_scroll += delta * 40.0;
            carousel.snap_timer = 0.0;
        }
    }
}

pub fn update_carousel_items(
    mut carousel_query: Query<(&mut Carousel, &ComputedNode)>,
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
    children_query: Query<&Children>,
    mut faders_query: Query<(
        &CarouselAlpha,
        Option<&mut TextColor>,
        Option<&mut BackgroundColor>,
        Option<&mut ImageNode>,
    )>,
    // mut text_query: Query<&mut TextColor>,
    mut materials: ResMut<Assets<FrostedGlassMaterial>>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();

    for (mut carousel, computed_node) in carousel_query.iter_mut() {
        if !carousel.enabled {
            continue;
        }
        // --- ÉTAPE DE SMOOTHING ---
        // On fait glisser current_scroll vers target_scroll

        carousel.current_scroll = carousel
            .current_scroll
            .lerp(carousel.target_scroll, dt * carousel.lerp_speed);

        let container_width = computed_node.size().x;
        if container_width <= 0.0 {
            continue;
        }

        let slot_width = carousel.item_width + carousel.spacing;
        let total_content_width = slot_width * carousel.total_items as f32;
        let teleport_boundary = total_content_width / 2.0;
        let centering_offset = (container_width / 2.0) - (carousel.item_width / 2.0);

        let container_limit = container_width * 0.45;
        let feather = 50.0;

        // if carousel.target_scroll.abs() > total_content_width {
        //     carousel.target_scroll %= total_content_width;
        //     carousel.current_scroll %= total_content_width;
        // }
        // Smooth wrapping: keep both values in sync
        if carousel.target_scroll > total_content_width {
            carousel.target_scroll -= total_content_width;
            carousel.current_scroll -= total_content_width;
        } else if carousel.target_scroll < -total_content_width {
            carousel.target_scroll += total_content_width;
            carousel.current_scroll += total_content_width;
        }

        for (entity, item, mut node, material_handle, children) in items_query.iter_mut() {
            if item.carousel_id != carousel.id {
                continue;
            }

            // --- LOGIQUE DE POSITION ---
            let base_x = item.index as f32 * slot_width;
            let mut x_pos = (base_x + carousel.current_scroll) % total_content_width;
            if x_pos < 0.0 {
                x_pos += total_content_width;
            }
            if x_pos > teleport_boundary {
                x_pos -= total_content_width;
            }

            node.left = Val::Px(x_pos + centering_offset);
            node.position_type = PositionType::Absolute;

            // --- LOGIQUE VISUELLE ---
            if let Some(material) = materials.get_mut(material_handle) {
                // 1. Position brute pour le shader (effet Wipe)
                material.uniforms.edge_fade = x_pos;
                material.uniforms._padding.x = container_width;

                // 2. CALCUL DE VISIBILITÉ CORRIGÉ
                // On veut que la carte soit 100% opaque pendant presque tout son trajet.
                // Elle ne commence à "fader" (alpha global) que lorsqu'elle approche
                // du moment où elle va disparaître (teleport_boundary).

                let dist_abs = x_pos.abs();

                // On commence à baisser l'alpha global uniquement 50 pixels avant la fin
                let fade_start = teleport_boundary - carousel.item_width * 1.5; //feather * 2.0;
                let fade_end = teleport_boundary;

                let visibility = ((fade_end - dist_abs) / (fade_end - fade_start)).clamp(0.0, 1.0);
                // let children_visibility = ((fade_end - 2.0 * feather - dist_abs)
                //     / (fade_end - 2.0 * feather - fade_start))
                //     .clamp(0.0, 1.0);
                let half_container = container_width / 2.0;
                let children_fade_start = half_container - carousel.item_width * 1.5;
                let children_fade_end = half_container - carousel.item_width * 0.5;
                let children_visibility = ((children_fade_end - dist_abs)
                    / (children_fade_end - children_fade_start))
                    .clamp(0.0, 1.0);

                // On applique cette visibilité
                material.uniforms.visibility = visibility;

                // // 2. Calcul de l'opacité globale (effet Fade)
                // // On veut que la carte soit à 0.0 d'opacité à l'endroit précis de la téléportation
                // let dist_abs = x_pos.abs();

                // // La visibilité commence à baisser à 70% du chemin vers le bord
                // let fade_start = teleport_boundary * 0.7;
                // let visibility =
                //     ((teleport_boundary - dist_abs) / (teleport_boundary - fade_start)).clamp(0.0, 1.0);

                // 3. Application au Verre (Uniforms)
                // On multiplie les opacités de base par notre facteur de visibilité
                // (Ici 0.3 et 0.85 sont tes valeurs de base du FrostedGlassConfig)
                // material.uniforms.opacity_top = 0.3 * visibility;
                // material.uniforms.opacity_bottom = 0.85 * visibility;

                // material.uniforms.visibility = visibility;

                // Mise à jour récursive des enfants avec CarouselAlpha
                update_descendants_opacity(
                    entity,
                    children_visibility,
                    children_query,
                    &mut faders_query,
                );

                // // 4. Application au Contenu (Texte, Icônes)
                // if let Some(children) = children {
                //     for child in children.iter() {
                //         if let Ok(mut text_color) = text_query.get_mut(child) {
                //             text_color.0.set_alpha(children_visibility);
                //         }
                //         // if let Ok(mut bg_color) = ui_color_query.get_mut(child) {
                //         //     bg_color.0.set_alpha(visibility);
                //         // }
                //     }
                // }
            }
        }
    }
}

pub fn apply_carousel_snap(time: Res<Time>, mut query: Query<&mut Carousel>) {
    let dt = time.delta_secs();

    for mut carousel in query.iter_mut() {
        carousel.snap_timer += dt;

        // Si l'utilisateur n'a pas scrollé depuis 150ms
        if carousel.snap_timer > 0.25 {
            let slot_width = carousel.item_width + carousel.spacing;

            // MATHÉMATIQUES DU SNAP :
            // On calcule à quel "index" de slot correspond la target actuelle
            // On utilise .round() pour trouver le slot le plus proche (0.0, 1.0, 2.0...)
            let nearest_slot = (carousel.target_scroll / slot_width).round();

            // On définit la nouvelle cible exactement sur ce slot
            let snapped_target = nearest_slot * slot_width;

            // On met à jour la cible. Le système de LERP s'occupera
            // de faire glisser les cartes doucement vers cette position.
            carousel.target_scroll = snapped_target;
        }
    }
}

pub fn update_carousel_icons(
    mut wipe_materials: ResMut<Assets<WipeMaterial>>,
    mut icons_query: Query<(&ChildOf, &MaterialNode<WipeMaterial>)>,
    cards_query: Query<&MaterialNode<FrostedGlassMaterial>>,
    frosted_materials: Res<Assets<FrostedGlassMaterial>>,
) {
    for (child_of, icon_mat_handle) in icons_query.iter_mut() {
        // child_of.entity() nous donne l'entité parente (la Carte)
        let parent_entity = child_of.0;

        if let Ok(card_mat_handle) = cards_query.get(parent_entity) {
            // On récupère les données de calcul depuis le matériau de la carte
            if let (Some(card_mat), Some(icon_mat)) = (
                frosted_materials.get(card_mat_handle),
                wipe_materials.get_mut(icon_mat_handle),
            ) {
                // SYNCHRONISATION CRUCIALE :
                // On recopie les paramètres d'animation de la carte vers l'icône
                icon_mat.uniforms.edge_fade = card_mat.uniforms.edge_fade;
                icon_mat.uniforms.visibility = card_mat.uniforms.visibility;
                icon_mat.uniforms.screen_size = card_mat.uniforms.screen_size;

                // Note: icon_mat.uniforms.size doit être la taille de l'icône (ex: 64x64)
                // Elle est généralement fixée au spawn ou via un système de resize.
            }
        }
    }
}

/// Fonction utilitaire pour propager l'opacité aux enfants
fn update_descendants_opacity(
    parent: Entity,
    visibility: f32,
    children_query: Query<&Children>,
    faders_query: &mut Query<(
        &CarouselAlpha,
        Option<&mut TextColor>,
        Option<&mut BackgroundColor>,
        Option<&mut ImageNode>,
    )>,
) {
    if let Ok(children) = children_query.get(parent) {
        for child in children.iter() {
            if let Ok((alpha_conf, text, bg, img)) = faders_query.get_mut(child) {
                let final_alpha = alpha_conf.base_alpha * visibility;

                if let Some(mut t) = text {
                    t.0.set_alpha(final_alpha);
                }

                if let Some(mut b) = bg {
                    // Only animate BackgroundColor if it was intentionally opaque
                    // Color::NONE (0,0,0,0) should stay transparent
                    if alpha_conf.has_visible_background {
                        b.0.set_alpha(final_alpha);
                    }
                }

                if let Some(mut i) = img {
                    i.color.set_alpha(final_alpha);
                }
            }

            update_descendants_opacity(child, visibility, children_query, faders_query);
        }
    }
}
