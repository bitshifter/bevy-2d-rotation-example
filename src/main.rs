use bevy::{
    core::FixedTimestep,
    math::{const_vec2, Vec3Swizzles},
    prelude::*,
};

const TIME_STEP: f32 = 1.0 / 60.0;
const BOUNDS: Vec2 = const_vec2!([1200.0, 640.0]);

trait QuaternionEx {
    fn from_rotation_arc_2d(from: Vec2, to: Vec2) -> Quat;
}

impl QuaternionEx for Quat {
    // Adapted from `Quat::from_rotation_arc` for 2D case with clamp
    fn from_rotation_arc_2d(from: Vec2, to: Vec2) -> Quat {
        const ONE_MINUS_EPSILON: f32 = 1.0 - 2.0 * core::f32::EPSILON;
        let dot = from.dot(to);
        if dot > ONE_MINUS_EPSILON {
            // 0° singulary: from ≈ to
            Quat::IDENTITY
        } else if dot < -ONE_MINUS_EPSILON {
            // 180° singulary: from ≈ -to
            const COS_FRAC_PI_2: f32 = 0.0;
            const SIN_FRAC_PI_2: f32 = 1.0;
            // rotation around z by PI radians
            Quat::from_xyzw(0.0, 0.0, SIN_FRAC_PI_2, COS_FRAC_PI_2)
        } else {
            // vector3 cross where z=0
            let z = from.x * to.y - to.x * from.y;
            let w = 1.0 + dot;
            // calculate length with x=0 and y=0 to normalize
            let len_rcp = 1.0 / (z * z + w * w).sqrt();
            Quat::from_xyzw(0.0, 0.0, z * len_rcp, w * len_rcp)
        }
    }
}

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
struct Enemy;

#[derive(Component)]
struct Player {
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
            transform: Transform::from_xyz(0.0, 40.0 - BOUNDS.y / 2.0, 0.0),
            ..Default::default()
        })
        .insert(Player {
            movement_speed: 500.0,
            rotation_speed: f32::to_radians(360.0),
        });

    // enemy
    commands
        .spawn_bundle(SpriteBundle {
            material: materials.add(enemy_handle.into()),
            ..Default::default()
        })
        .insert(Enemy);

    // Add walls
    let wall_material = materials.add(Color::rgb(0.8, 0.8, 0.8).into());
    let wall_thickness = 10.0;

    // left
    commands.spawn_bundle(SpriteBundle {
        material: wall_material.clone(),
        transform: Transform::from_xyz(-BOUNDS.x / 2.0, 0.0, 0.0),
        sprite: Sprite::new(Vec2::new(wall_thickness, BOUNDS.y + wall_thickness)),
        ..Default::default()
    });
    // right
    commands.spawn_bundle(SpriteBundle {
        material: wall_material.clone(),
        transform: Transform::from_xyz(BOUNDS.x / 2.0, 0.0, 0.0),
        sprite: Sprite::new(Vec2::new(wall_thickness, BOUNDS.y + wall_thickness)),
        ..Default::default()
    });
    // bottom
    commands.spawn_bundle(SpriteBundle {
        material: wall_material.clone(),
        transform: Transform::from_xyz(0.0, -BOUNDS.y / 2.0, 0.0),
        sprite: Sprite::new(Vec2::new(BOUNDS.x + wall_thickness, wall_thickness)),
        ..Default::default()
    });
    // top
    commands.spawn_bundle(SpriteBundle {
        material: wall_material,
        transform: Transform::from_xyz(0.0, BOUNDS.y / 2.0, 0.0),
        sprite: Sprite::new(Vec2::new(BOUNDS.x + wall_thickness, wall_thickness)),
        ..Default::default()
    });
}

fn player_movement_system(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&Player, &mut Transform)>,
) {
    let (ship, mut transform) = query.single_mut();

    let mut rotation_factor = 0.0;
    let mut movement_factor = 0.0;

    if keyboard_input.pressed(KeyCode::Left) {
        rotation_factor += 1.0;
    }

    if keyboard_input.pressed(KeyCode::Right) {
        rotation_factor -= 1.0;
    }

    if keyboard_input.pressed(KeyCode::Up) {
        movement_factor += 1.0;
    }

    // create the change in rotation around the Z axis (pointing through the 2d plane of the screen)
    let rotation_delta = Quat::from_rotation_z(rotation_factor * ship.rotation_speed * TIME_STEP);
    // update the ship rotation with our rotation delta
    transform.rotation *= rotation_delta;

    // get the ship's forward vector by apply the current rotation to the ships initial facing vector
    let movement_direction = transform.rotation * Vec3::Y;
    // get the distance the ship will move based on direction, the ship's movement speed and delta
    // time
    let movement_distance = movement_factor * ship.movement_speed * TIME_STEP;
    // create the change in translation using the new movement direction and distance
    let translation_delta = movement_direction * movement_distance;
    // update the ship translation with our new translation delta
    transform.translation += translation_delta;

    // bound the ship within the walls
    let extents = Vec3::from((BOUNDS / 2.0, 0.0));
    transform.translation = transform.translation.min(extents).max(-extents);
}

fn enemy_movement_system(
    mut query: Query<(&Enemy, &mut Transform), Without<Player>>,
    player_query: Query<&Transform, With<Player>>,
) {
    let player_transform = player_query.single();

    for (_, mut enemy_transform) in query.iter_mut() {
        let enemy_forward = (enemy_transform.rotation * Vec3::Y).xy();
        let player_direction =
            (player_transform.translation.xy() - enemy_transform.translation.xy()).normalize();

        // get the quaternion to rotate from the current enemy facing direction to the direction
        // facing the player
        let rotate_to_player = Quat::from_rotation_arc_2d(enemy_forward, player_direction);

        // rotate the enemy to face the player
        enemy_transform.rotation *= rotate_to_player;
    }
}
