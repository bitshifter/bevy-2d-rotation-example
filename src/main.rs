use bevy::{core::FixedTimestep, prelude::*};

const TIME_STEP: f32 = 1.0 / 60.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(TIME_STEP as f64))
                .with_system(player_movement_system)
                .with_system(enemy_movement_system),
        )
        .add_system(bevy::input::system::exit_on_esc_system)
        .run();
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct ShipConfig {
    movement_speed: f32,
    rotation_speed: f32,
}

fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // Add the game's entities to our world

    let ship_handle = asset_server.load("textures/simplespace/ship_C.png");
    let enemy_handle = asset_server.load("textures/simplespace/enemy_A.png");

    // cameras
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.spawn_bundle(UiCameraBundle::default());

    // ship
    commands
        .spawn_bundle(SpriteBundle {
            material: materials.add(ship_handle.into()),
            ..Default::default()
        })
        .insert(ShipConfig {
            movement_speed: 500.0,
            rotation_speed: f32::to_radians(360.0),
        })
        .insert(Player);

    // enemy
    commands
        .spawn_bundle(SpriteBundle {
            material: materials.add(enemy_handle.into()),
            ..Default::default()
        })
    .insert(ShipConfig {
        movement_speed: 0.0,
        rotation_speed: f32::to_radians(180.0)
    });

    // Add walls
    let wall_material = materials.add(Color::rgb(0.8, 0.8, 0.8).into());
    let wall_thickness = 10.0;
    let bounds = Vec2::new(900.0, 600.0);

    // left
    commands
        .spawn_bundle(SpriteBundle {
            material: wall_material.clone(),
            transform: Transform::from_xyz(-bounds.x / 2.0, 0.0, 0.0),
            sprite: Sprite::new(Vec2::new(wall_thickness, bounds.y + wall_thickness)),
            ..Default::default()
        });
    // right
    commands
        .spawn_bundle(SpriteBundle {
            material: wall_material.clone(),
            transform: Transform::from_xyz(bounds.x / 2.0, 0.0, 0.0),
            sprite: Sprite::new(Vec2::new(wall_thickness, bounds.y + wall_thickness)),
            ..Default::default()
        });
    // bottom
    commands
        .spawn_bundle(SpriteBundle {
            material: wall_material.clone(),
            transform: Transform::from_xyz(0.0, -bounds.y / 2.0, 0.0),
            sprite: Sprite::new(Vec2::new(bounds.x + wall_thickness, wall_thickness)),
            ..Default::default()
        });
    // top
    commands
        .spawn_bundle(SpriteBundle {
            material: wall_material,
            transform: Transform::from_xyz(0.0, bounds.y / 2.0, 0.0),
            sprite: Sprite::new(Vec2::new(bounds.x + wall_thickness, wall_thickness)),
            ..Default::default()
        });

}

fn player_movement_system(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&ShipConfig, &mut Transform), With<Player>>,
) {
    let (ship, mut transform) = query.single_mut();

    let mut rotation_direction = 0.0;
    let mut movement_direction = 0.0;

    if keyboard_input.pressed(KeyCode::Left) {
        rotation_direction += 1.0;
    }

    if keyboard_input.pressed(KeyCode::Right) {
        rotation_direction -= 1.0;
    }

    if keyboard_input.pressed(KeyCode::Up) {
        movement_direction += 1.0;
    }

    // create the change in rotation around the Z axis (pointing through the 2d plane of the screen)
    let rotation_delta =
        Quat::from_rotation_z(rotation_direction * ship.rotation_speed * TIME_STEP);
    // update the ship rotation with our rotation delta
    transform.rotation *= rotation_delta;

    // create the change in translation using the new rotation directon
    let translation_delta =
        transform.rotation * Vec3::Y * movement_direction * ship.movement_speed * TIME_STEP;
    // update the ship translation with our new translation delta
    transform.translation += translation_delta;

    // bound the ship within the walls
    let extents = Vec3::new(450.0, 300.0, 0.0);
    transform.translation = transform.translation.min(extents).max(-extents);
}

fn enemy_movement_system(
    mut query: Query<(&ShipConfig, &mut Transform), Without<Player>>,
    player_query: Query<&Transform, With<Player>>
) {
    let _player_transform = player_query.single();

    for (_enemy_ship, mut _enemy_transform) in query.iter_mut() {
        // TODO
    }
}
