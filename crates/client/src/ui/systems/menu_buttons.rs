use bevy::{prelude::*};
// use bevy::{color::palettes::css::*, prelude::*};

use crate::ui::components::MenuButton;

const NORMAL_COLOR: Color = Color::srgb_u8(157, 136, 93);
const HOVER_COLOR: Color = Color::srgb_u8(197, 176, 133);
const CLICK_COLOR: Color = Color::srgb_u8(227, 206, 163);

pub fn handle_menu_button_interactions(
    mut query: Query<(&MenuButton, &mut ImageNode, &Interaction), Changed<Interaction>>,
) {
    for (button, mut image_node, interaction) in query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                image_node.color = CLICK_COLOR;
                info!("Menu button {} clicked!", button.button_id);
                // TODO: Handle button actions based on button_id
            }
            Interaction::Hovered => {
                image_node.color = HOVER_COLOR;
            }
            Interaction::None => {
                image_node.color = NORMAL_COLOR;
            }
        }
    }
}
