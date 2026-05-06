use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

#[derive(Component)]
struct Player;

#[derive(Component)]
struct MainCamera;

#[derive(Component)]
struct Wall;

#[derive(Component)]
struct Ball;

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        .add_plugins(RapierDebugRenderPlugin::default())
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands
        .spawn(MainCamera)
        .insert(Camera2d)
        .insert(Camera {
            clear_color: ClearColorConfig::Custom(Color::BLACK),
            ..default()
        })
        .insert(Projection::Orthographic(OrthographicProjection {
            scaling_mode: bevy::camera::ScalingMode::Fixed {
                width: 500.,
                height: 250.,
            },
            ..OrthographicProjection::default_2d()
        }));

    commands
        .spawn(Wall)
        .insert(RigidBody::Fixed)
        .insert(Collider::cuboid(250., 2.5))
        .insert(Mesh2d(meshes.add(Rectangle::new(500., 5.))))
        .insert(MeshMaterial2d(materials.add(Color::WHITE)))
        .insert(Transform::from_xyz(0., -125., 0.));

    commands
        .spawn(Wall)
        .insert(RigidBody::Fixed)
        .insert(Collider::cuboid(250., 2.5))
        .insert(Mesh2d(meshes.add(Rectangle::new(500., 5.))))
        .insert(MeshMaterial2d(materials.add(Color::WHITE)))
        .insert(Transform::from_xyz(0., 125., 0.));

    commands
        .spawn(Ball)
        .insert(RigidBody::Dynamic)
        .insert(GravityScale::default())
        .insert(Collider::ball(2.5))
        .insert(Mesh2d(meshes.add(Circle::new(2.5))))
        .insert(MeshMaterial2d(materials.add(Color::WHITE)))
        .insert(Transform::from_xyz(0., 0., 0.));
}
