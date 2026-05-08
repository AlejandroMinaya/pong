use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use rand::prelude::*;

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Opponent;

#[derive(Component)]
struct Paddle;

#[derive(Component)]
struct MainCamera;

#[derive(Component)]
struct Wall;

#[derive(Component)]
enum Goal {
    Home,
    Away,
}

#[derive(Message)]
struct GoalScored {
    affected_goal: Goal,
}

#[derive(Message)]
struct SwingBallHit {
    direction: Vec2,
}

#[derive(Component)]
struct Ball;

#[derive(Resource)]
struct BallConfig {
    speed: f32,
    size: f32,
    mass: f32,
    bounciness: f32,
    serve_delay: f32,
}

#[derive(Resource)]
struct PaddleConfig {
    height: f32,
    width: f32,
    padding: f32,
    strength: f32,
}

#[derive(Resource)]
struct PlayerConfig {
    speed: f32,
}

#[derive(Resource)]
struct OpponentConfig {
    reflex: f32,
}

#[derive(Resource, Default)]
struct Scoreboard {
    home_goal_id: Option<Entity>,
    away_goal_id: Option<Entity>,
    ball_goal_id: Option<Entity>,
    player_id: Option<Entity>,
    opponent_id: Option<Entity>,
    home: usize,
    away: usize,
    ball_in_field: bool,
    serve_timer: Timer,
}

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins)
        .insert_resource(BallConfig {
            speed: 150.0,
            size: 2.5,
            mass: 0.56,
            bounciness: 1.0,
            serve_delay: 1.0,
        })
        .insert_resource(PaddleConfig {
            height: 30.0,
            width: 5.0,
            padding: 5.0,
            strength: 10.0,
        })
        .insert_resource(PlayerConfig { speed: 5.0 })
        .insert_resource(OpponentConfig { reflex: 10.0 })
        .insert_resource(Scoreboard::default())
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        // .add_plugins(RapierDebugRenderPlugin::default())
        .add_message::<GoalScored>()
        .add_message::<SwingBallHit>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                move_paddle,
                move_opponent,
                triage_goal_events,
                tally_score,
                reset_ball,
                hit_ball,
                recolor_ball,
            ),
        )
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut scoreboard: ResMut<Scoreboard>,
    ball_config: Res<BallConfig>,
    paddle_config: Res<PaddleConfig>,
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
        .insert(Friction {
            coefficient: 0.,
            combine_rule: CoefficientCombineRule::Min,
        })
        .insert(Mesh2d(meshes.add(Rectangle::new(500., 5.))))
        .insert(MeshMaterial2d(materials.add(Color::WHITE)))
        .insert(Transform::from_xyz(0., -125., 0.));

    commands
        .spawn(Wall)
        .insert(RigidBody::Fixed)
        .insert(Collider::cuboid(250., 2.5))
        .insert(Friction {
            coefficient: 0.,
            combine_rule: CoefficientCombineRule::Min,
        })
        .insert(Mesh2d(meshes.add(Rectangle::new(500., 5.))))
        .insert(MeshMaterial2d(materials.add(Color::WHITE)))
        .insert(Transform::from_xyz(0., 125., 0.));

    scoreboard.home_goal_id = Some(
        commands
            .spawn(Goal::Home)
            .insert(Collider::cuboid(paddle_config.width / 2., 125.))
            .insert(Sensor)
            .insert(Transform::from_xyz(250., 0., 0.))
            .id(),
    );

    scoreboard.away_goal_id = Some(
        commands
            .spawn(Goal::Away)
            .insert(Collider::cuboid(paddle_config.width / 2., 125.))
            .insert(Sensor)
            .insert(Transform::from_xyz(-250., 0., 0.))
            .id(),
    );

    scoreboard.ball_goal_id = Some(
        commands
            .spawn(Ball)
            .insert(RigidBody::Dynamic)
            .insert(Velocity::zero())
            .insert(GravityScale(0.))
            .insert(Ccd::enabled())
            .insert(ExternalImpulse::default())
            .insert(Restitution {
                coefficient: ball_config.bounciness,
                combine_rule: CoefficientCombineRule::Max,
            })
            .insert(Collider::ball(ball_config.size))
            .insert(ColliderMassProperties::Mass(ball_config.mass))
            .insert(Mesh2d(meshes.add(Circle::new(ball_config.size))))
            .insert(MeshMaterial2d(materials.add(Color::WHITE)))
            .insert(ActiveEvents::COLLISION_EVENTS)
            .insert(Transform::from_xyz(0., 0., 0.))
            .id(),
    );
    scoreboard.ball_in_field = false;
    scoreboard.serve_timer = Timer::from_seconds(ball_config.serve_delay, TimerMode::Once);

    scoreboard.opponent_id = Some(
        commands
            .spawn(Opponent)
            .insert(Paddle)
            .insert(RigidBody::KinematicPositionBased)
            .insert(LockedAxes::TRANSLATION_LOCKED_X)
            .insert(Restitution {
                coefficient: 1.,
                combine_rule: CoefficientCombineRule::Max,
            })
            .insert(Collider::cuboid(
                paddle_config.width / 2.,
                paddle_config.height / 2.,
            ))
            .insert(Mesh2d(
                meshes.add(Rectangle::new(paddle_config.width, paddle_config.height)),
            ))
            .insert(MeshMaterial2d(materials.add(Color::WHITE)))
            .insert(Transform::from_xyz(paddle_config.padding - 250.0, 0., 0.))
            .id(),
    );

    scoreboard.player_id = Some(
        commands
            .spawn(Player)
            .insert(KinematicCharacterController::default())
            .insert(Paddle)
            .insert(RigidBody::KinematicPositionBased)
            .insert(LockedAxes::TRANSLATION_LOCKED_X)
            .insert(Restitution {
                coefficient: 0.,
                combine_rule: CoefficientCombineRule::Max,
            })
            .insert(Collider::cuboid(
                paddle_config.width / 2.,
                paddle_config.height / 2.,
            ))
            .insert(Mesh2d(
                meshes.add(Rectangle::new(paddle_config.width, paddle_config.height)),
            ))
            .insert(MeshMaterial2d(materials.add(Color::WHITE)))
            .insert(Transform::from_xyz(250.0 - paddle_config.padding, 0., 0.))
            .id(),
    );
}

fn move_paddle(
    input: Res<ButtonInput<KeyCode>>,
    player_config: Res<PlayerConfig>,
    mut controller: Single<&mut KinematicCharacterController, With<Player>>,
) {
    let mut direction = Vec2::ZERO;

    if input.pressed(KeyCode::ArrowUp) {
        direction.y = 1. * player_config.speed;
    }
    if input.pressed(KeyCode::ArrowDown) {
        direction.y = -1. * player_config.speed;
    }
    controller.translation = Some(direction);
}

fn move_opponent(
    time: Res<Time>,
    opponent_config: Res<OpponentConfig>,
    ball_transform: Single<&Transform, (With<Ball>, Without<Opponent>)>,
    mut opponent_transform: Single<&mut Transform, (With<Opponent>, Without<Ball>)>,
) {
    let target = Vec3 {
        y: ball_transform.translation.y,
        x: opponent_transform.translation.x,
        z: opponent_transform.translation.z,
    };
    opponent_transform
        .translation
        .smooth_nudge(&target, opponent_config.reflex, time.delta_secs());
}

fn triage_goal_events(
    scoreboard: Res<Scoreboard>,
    mut collision_events: MessageReader<CollisionEvent>,
    mut goal_writer: MessageWriter<GoalScored>,
    mut swing_writer: MessageWriter<SwingBallHit>,
) {
    for collision_event in collision_events.read() {
        if let (
            CollisionEvent::Stopped(emitter, receiver, _),
            Some(home_goal),
            Some(away_goal),
            Some(ball),
            Some(player),
            Some(opponent),
        ) = (
            collision_event,
            scoreboard.home_goal_id,
            scoreboard.away_goal_id,
            scoreboard.ball_goal_id,
            scoreboard.player_id,
            scoreboard.opponent_id,
        ) && *receiver == ball
        {
            match *emitter {
                n if n == home_goal => {
                    goal_writer.write(GoalScored {
                        affected_goal: Goal::Home,
                    });
                }
                n if n == away_goal => {
                    goal_writer.write(GoalScored {
                        affected_goal: Goal::Away,
                    });
                }
                n if n == player => {
                    swing_writer.write(SwingBallHit {
                        direction: Vec2::new(-1., 0.),
                    });
                }
                n if n == opponent => {
                    swing_writer.write(SwingBallHit {
                        direction: Vec2::new(1., 0.),
                    });
                }
                _ => println!("Unhandled emitter: {:?}", emitter),
            }
        }
    }
}

fn tally_score(mut scoreboard: ResMut<Scoreboard>, mut goal_reader: MessageReader<GoalScored>) {
    for goal_event in goal_reader.read() {
        scoreboard.ball_in_field = false;
        match goal_event.affected_goal {
            Goal::Home => scoreboard.away += 1,
            Goal::Away => scoreboard.home += 1,
        }
        println!("Home: {} - Away: {}", scoreboard.home, scoreboard.away);
    }
}
fn reset_ball(
    ball_config: Res<BallConfig>,
    time: Res<Time>,
    mut scoreboard: ResMut<Scoreboard>,
    mut transform: Single<&mut Transform, With<Ball>>,
    mut velocity: Single<&mut Velocity, With<Ball>>,
    mut external_impulse: Single<&mut ExternalImpulse, With<Ball>>,
) {
    if scoreboard.ball_in_field {
        return;
    }

    transform.translation = Vec3::ZERO;
    transform.rotation = Quat::from_xyzw(0., 0., 0., 0.);
    velocity.linvel = Vec2::ZERO;
    velocity.angvel = 0.;

    if !scoreboard.serve_timer.is_finished() {
        scoreboard.serve_timer.tick(time.delta());
        return;
    }
    scoreboard.serve_timer.reset();

    let mut rng = rand::rng();
    let launch_impulse = if rng.random_bool(0.5) {
        Vec2::new(1., 0.)
    } else {
        Vec2::new(-1., 0.)
    } * ball_config.speed;
    external_impulse.impulse = launch_impulse;
    external_impulse.torque_impulse = 0.;

    scoreboard.ball_in_field = true;
}

fn hit_ball(
    paddle_config: Res<PaddleConfig>,
    mut hit_reader: MessageReader<SwingBallHit>,
    mut external_impulse: Single<&mut ExternalImpulse, With<Ball>>,
    mut velocity: Single<&mut Velocity, With<Ball>>,
) {
    let mut rng = rand::rng();
    for SwingBallHit { direction } in hit_reader.read() {
        external_impulse.impulse += direction * paddle_config.strength;
    }
    velocity.angvel = rng.random_range(-50.0..50.0)
}
fn recolor_ball(
    ball_material: Single<&MeshMaterial2d<ColorMaterial>, With<Ball>>,
    mut hit_reader: MessageReader<SwingBallHit>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    if let Some(ball_material_handle) = materials.get_mut(*ball_material) {
        ball_material_handle.color = Color::WHITE;
        for _ in hit_reader.read() {
            ball_material_handle.color = Color::from(Srgba::RED);
        }
    }
}
