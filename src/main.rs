use bevy::{math::VectorSpace, prelude::*};
use bevy_rapier2d::prelude::*;
use rand::prelude::*;

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Paddle;

#[derive(Component)]
struct MainCamera;

#[derive(Component)]
struct Wall;

#[derive(Component)]
struct Goal;

#[derive(Component)]
struct Ball;

#[derive(Resource)]
struct BallConfig {
    speed: f32,
    size: f32,
    mass: f32,
    bounciness: f32,
}

#[derive(Resource)]
struct PaddleConfig {
    height: f32,
    width: f32,
    padding: f32,
}

#[derive(Resource)]
struct PlayerConfig {
    speed: f32,
}

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins)
        .insert_resource(BallConfig {
            speed: 150.0,
            size: 2.5,
            mass: 0.56,
            bounciness: 1.0,
        })
        .insert_resource(PaddleConfig {
            height: 20.0,
            width: 5.0,
            padding: 5.0,
        })
        .insert_resource(PlayerConfig { speed: 5.0 })
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        .add_plugins(RapierDebugRenderPlugin::default())
        .add_systems(Startup, setup)
        .add_systems(Update, (move_paddle, respawn_ball))
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    ball_config: Res<BallConfig>,
    paddle_config: Res<PaddleConfig>,
) {
    let mut rng = rand::rng();
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
        .spawn(Goal)
        .insert(Collider::cuboid(paddle_config.width / 2., 125.))
        .insert(Sensor)
        .insert(ActiveEvents::COLLISION_EVENTS)
        .insert(Transform::from_xyz(250., 0., 0.));

    commands
        .spawn(Goal)
        .insert(Collider::cuboid(paddle_config.width / 2., 125.))
        .insert(Sensor)
        .insert(ActiveEvents::COLLISION_EVENTS)
        .insert(Transform::from_xyz(-250., 0., 0.));

    let launch_impulse = if rng.random_bool(0.5) {
        Vec2::new(1., 0.)
    } else {
        Vec2::new(-1., 0.)
    } * ball_config.speed;
    commands
        .spawn(Ball)
        .insert(RigidBody::Dynamic)
        .insert(GravityScale(0.))
        .insert(Ccd::enabled())
        .insert(ExternalImpulse {
            impulse: launch_impulse,
            torque_impulse: 0.,
        })
        .insert(Restitution {
            coefficient: ball_config.bounciness,
            combine_rule: CoefficientCombineRule::Max,
        })
        .insert(Collider::ball(ball_config.size))
        .insert(ColliderMassProperties::Mass(ball_config.mass))
        .insert(Mesh2d(meshes.add(Circle::new(ball_config.size))))
        .insert(MeshMaterial2d(materials.add(Color::WHITE)))
        .insert(Transform::from_xyz(0., 0., 0.));

    commands
        .spawn(Paddle)
        .insert(RigidBody::KinematicPositionBased)
        .insert(Collider::cuboid(
            paddle_config.width / 2.,
            paddle_config.height / 2.,
        ))
        .insert(Mesh2d(
            meshes.add(Rectangle::new(paddle_config.width, paddle_config.height)),
        ))
        .insert(MeshMaterial2d(materials.add(Color::WHITE)))
        .insert(Transform::from_xyz(paddle_config.padding - 250.0, 0., 0.));

    commands
        .spawn(Player)
        .insert(KinematicCharacterController::default())
        .insert(Paddle)
        .insert(RigidBody::KinematicPositionBased)
        .insert(Collider::cuboid(
            paddle_config.width / 2.,
            paddle_config.height / 2.,
        ))
        .insert(Mesh2d(
            meshes.add(Rectangle::new(paddle_config.width, paddle_config.height)),
        ))
        .insert(MeshMaterial2d(materials.add(Color::WHITE)))
        .insert(Transform::from_xyz(250.0 - paddle_config.padding, 0., 0.));
}

fn move_paddle(
    input: Res<ButtonInput<KeyCode>>,
    player_config: Res<PlayerConfig>,
    mut controller: Single<&mut KinematicCharacterController, With<Player>>,
) {
    let mut direction = Vec2::ZERO;

    if input.pressed(KeyCode::ArrowUp) {
        direction.y = 1.;
    }
    if input.pressed(KeyCode::ArrowDown) {
        direction.y = -1.;
    }
    controller.translation = Some(direction * player_config.speed);
}

fn respawn_ball(
    mut ball_transform: Single<&mut Transform, With<Ball>>,
    mut collision_events: MessageReader<CollisionEvent>,
) {
    for collision_event in collision_events.read() {
        if let CollisionEvent::Stopped(_, _, _) = collision_event {
            ball_transform.translation = Vec3::ZERO;
        }
    }
}
