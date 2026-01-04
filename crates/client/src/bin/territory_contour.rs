use std::collections::HashSet;

use bevy::{pbr::PbrPlugin, prelude::*, window::PresentMode};
use hexx::*;

mod camera;
mod materials;
mod territory;
mod ui;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb_u8(0, 15, 30)))
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Territory tests".to_string(),
                        resolution: (1280, 720).into(),
                        present_mode: PresentMode::AutoNoVsync,
                        ..default()
                    }),
                    ..default()
                })
                .disable::<PbrPlugin>()
                .set(AssetPlugin {
                    file_path: "../../assets".to_string(),
                    ..default()
                }),
        )
        .add_plugins((
            camera::CameraPlugin,
            territory::TerritoryPlugin,
            ui::UiPlugin,
        ))
        .run();

    // debug_corners(&grid_config.layout);

    // let territory_cells = [
    //     Hex::new(42, 41),
    //     Hex::new(43, 40),
    //     Hex::new(43, 41),
    //     Hex::new(44, 39),
    //     Hex::new(44, 40),
    //     Hex::new(44, 41),
    //     Hex::new(45, 38),
    //     Hex::new(45, 39),
    //     Hex::new(45, 40),
    //     Hex::new(45, 41),
    //     Hex::new(46, 37),
    //     Hex::new(46, 38),
    //     Hex::new(46, 39),
    //     Hex::new(46, 40),
    //     Hex::new(46, 41),
    //     Hex::new(47, 36),
    //     Hex::new(47, 37),
    //     Hex::new(47, 38),
    //     Hex::new(47, 39),
    //     Hex::new(47, 40),
    //     Hex::new(47, 41),
    //     Hex::new(48, 35),
    //     Hex::new(48, 36),
    //     Hex::new(48, 37),
    //     Hex::new(48, 38),
    //     Hex::new(48, 39),
    //     Hex::new(48, 40),
    //     Hex::new(48, 41),
    //     Hex::new(48, 42),
    //     Hex::new(49, 34),
    //     Hex::new(49, 35),
    //     Hex::new(49, 36),
    //     Hex::new(49, 37),
    //     Hex::new(49, 38),
    //     Hex::new(49, 39),
    //     Hex::new(49, 40),
    //     Hex::new(49, 41),
    //     Hex::new(49, 42),
    //     Hex::new(50, 33),
    //     Hex::new(50, 34),
    //     Hex::new(50, 35),
    //     Hex::new(50, 36),
    //     Hex::new(50, 37),
    //     Hex::new(50, 38),
    //     Hex::new(50, 39),
    //     Hex::new(50, 40),
    //     Hex::new(50, 41),
    //     Hex::new(50, 42),
    //     Hex::new(51, 32),
    //     Hex::new(51, 33),
    //     Hex::new(51, 34),
    //     Hex::new(51, 35),
    //     Hex::new(51, 36),
    //     Hex::new(51, 37),
    //     Hex::new(51, 38),
    //     Hex::new(51, 39),
    //     Hex::new(51, 40),
    //     Hex::new(51, 41),
    //     Hex::new(51, 42),
    //     Hex::new(51, 43),
    //     Hex::new(52, 31),
    //     Hex::new(52, 32),
    //     Hex::new(52, 33),
    //     Hex::new(52, 34),
    //     Hex::new(52, 35),
    //     Hex::new(52, 36),
    //     Hex::new(52, 37),
    //     Hex::new(52, 38),
    //     Hex::new(52, 39),
    //     Hex::new(52, 40),
    //     Hex::new(52, 41),
    //     Hex::new(52, 42),
    //     Hex::new(52, 43),
    //     Hex::new(53, 32),
    //     Hex::new(53, 33),
    //     Hex::new(53, 34),
    //     Hex::new(53, 35),
    //     Hex::new(53, 36),
    //     Hex::new(53, 37),
    //     Hex::new(53, 38),
    //     Hex::new(53, 39),
    //     Hex::new(53, 40),
    //     Hex::new(53, 41),
    //     Hex::new(53, 42),
    //     Hex::new(54, 33),
    //     Hex::new(54, 34),
    //     Hex::new(54, 35),
    //     Hex::new(54, 36),
    //     Hex::new(54, 37),
    //     Hex::new(54, 38),
    //     Hex::new(54, 39),
    //     Hex::new(54, 40),
    //     Hex::new(55, 34),
    //     Hex::new(55, 35),
    //     Hex::new(55, 36),
    //     Hex::new(55, 37),
    //     Hex::new(55, 38),
    //     Hex::new(55, 39),
    //     Hex::new(56, 34),
    //     Hex::new(56, 35),
    //     Hex::new(56, 36),
    //     Hex::new(56, 37),
    //     Hex::new(56, 38),
    //     Hex::new(57, 35),
    //     Hex::new(57, 36),
    // ];

    // let contour_cells = [
    //     Hex::new(5, 3),
    //     Hex::new(6, 3),
    //     Hex::new(7, 3),
    //     Hex::new(8, 3),
    //     Hex::new(9, 3),
    //     Hex::new(10, 3),
    //     Hex::new(11, 3),
    //     Hex::new(11, 4),
    //     Hex::new(12, 4),
    //     Hex::new(13, 4),
    //     Hex::new(14, 4),
    //     Hex::new(14, 5),
    //     Hex::new(15, 5),
    //     Hex::new(16, 4),
    //     Hex::new(16, 3),
    //     Hex::new(17, 2),
    //     Hex::new(18, 1),
    //     Hex::new(19, 0),
    //     Hex::new(19, -1),
    //     Hex::new(20, -2),
    //     Hex::new(20, -3),
    //     Hex::new(19, -3),
    //     Hex::new(19, -4),
    //     Hex::new(18, -4),
    //     Hex::new(17, -4),
    //     Hex::new(17, -5),
    //     Hex::new(16, -5),
    //     Hex::new(16, -6),
    //     Hex::new(15, -6),
    //     Hex::new(15, -7),
    //     Hex::new(14, -6),
    //     Hex::new(13, -5),
    //     Hex::new(12, -4),
    //     Hex::new(11, -3),
    //     Hex::new(10, -2),
    //     Hex::new(9, -1),
    //     Hex::new(8, 0),
    //     Hex::new(7, 1),
    //     Hex::new(6, 2),
    // ];

    // let expected_contour = [
    //     Hex::new(42, 41), // était 5, 3
    //     Hex::new(43, 41), // était 6, 3
    //     Hex::new(44, 41), // était 7, 3
    //     Hex::new(45, 41), // était 8, 3
    //     Hex::new(46, 41), // était 9, 3
    //     Hex::new(47, 41), // était 10, 3
    //     Hex::new(48, 41), // était 11, 3
    //     Hex::new(48, 42), // était 11, 4
    //     Hex::new(49, 42), // était 12, 4
    //     Hex::new(50, 42), // était 13, 4
    //     Hex::new(51, 42), // était 14, 4
    //     Hex::new(51, 43), // était 14, 5
    //     Hex::new(52, 43), // était 15, 5
    //     Hex::new(53, 42), // était 16, 4
    //     Hex::new(53, 41), // était 16, 3
    //     Hex::new(54, 40), // était 17, 2
    //     Hex::new(55, 39), // était 18, 1
    //     Hex::new(56, 38), // était 19, 0
    //     Hex::new(56, 37), // était 19, -1
    //     Hex::new(57, 36), // était 20, -2
    //     Hex::new(57, 35), // était 20, -3
    //     Hex::new(56, 35), // était 19, -3
    //     Hex::new(56, 34), // était 19, -4
    //     Hex::new(55, 34), // était 18, -4
    //     Hex::new(54, 34), // était 17, -4
    //     Hex::new(54, 33), // était 17, -5
    //     Hex::new(53, 33), // était 16, -5
    //     Hex::new(53, 32), // était 16, -6
    //     Hex::new(52, 32), // était 15, -6
    //     Hex::new(52, 31), // était 15, -7
    //     Hex::new(51, 32), // était 14, -6
    //     Hex::new(50, 33), // était 13, -5
    //     Hex::new(49, 34), // était 12, -4
    //     Hex::new(48, 35), // était 11, -3
    //     Hex::new(47, 36), // était 10, -2
    //     Hex::new(46, 37), // était 9, -1
    //     Hex::new(45, 38), // était 8, 0
    //     Hex::new(44, 39), // était 7, 1
    //     Hex::new(43, 40), // était 6, 2
    // ];

    // let territory: HashSet<Hex> = territory_cells.into_iter().collect();
    // let ordered_border = territory::trace_border_hexes(&territory);
    // let contour_points = territory::build_contour_points(&grid_config.layout, &territory, &ordered_border);

    // for (i, point) in contour_points.iter().enumerate() {

    //     println!("{:2}: {:?}", i, point);
    // }
}

fn debug_corners(layout: &HexLayout) {
    let hex = Hex::ZERO;
    let center = layout.hex_to_world_pos(hex);
    let corners = layout.hex_corners(hex);

    println!("Center: {:?}", center);
    for (i, corner) in corners.iter().enumerate() {
        let delta = *corner - center;
        let angle = delta.y.atan2(delta.x).to_degrees();
        println!("Corner {}: {:?} (angle: {:.1}°)", i, corner, angle);
    }
}
