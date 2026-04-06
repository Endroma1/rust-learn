use bevy::{
    math::bounding::{Aabb2d, BoundingCircle, BoundingVolume, IntersectsVolume},
    prelude::*,
};
const BALL_RADIUS: f32 = 10.;
const BORDER_WIDTH: f32 = 500.;
const BORDER_HEIGHT: f32 = 500.;
const BORDER_PADDING: f32 = 10.;
const BALL_VELOCITY: Vec2 = Vec2::new(0., 0.);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(Color::srgb(0.9, 0.9, 0.9)))
        .add_systems(Startup, (spawn, set_fixed_time).chain())
        .add_systems(
            FixedUpdate,
            (
                apply_acceleration,
                apply_velocity_ball,
                check_ball_collision,
                check_collision,
            )
                .chain(),
        )
        .run();
}

#[derive(Component)]
#[require(Gravity)]
struct Ball;

impl Ball {
    fn new(
        start_position: Vec2,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<ColorMaterial>>,
    ) -> (
        Ball,
        Mesh2d,
        Transform,
        MeshMaterial2d<ColorMaterial>,
        Velocity,
        Acceleration,
        Mass,
    ) {
        (
            Ball,
            Mesh2d(meshes.add(Circle::default())),
            Transform::from_translation(start_position.extend(0.))
                .with_scale(Vec2::splat(BALL_RADIUS * 2.).extend(1.)),
            MeshMaterial2d(materials.add(Color::WHITE)),
            Velocity(Vec2::from_array(BALL_VELOCITY.into())),
            Acceleration(Vec2::new(0., -100.)),
            Mass(10),
        )
    }
}

#[derive(Component, Deref, DerefMut)]
struct Velocity(Vec2);

#[derive(Component, Default)]
struct Gravity;

#[derive(Component, Deref, DerefMut, Default)]
struct Acceleration(Vec2);

#[derive(Component, Default)]
struct Collider;

#[derive(Component, Default)]
struct Mass(u32);

#[derive(Component)]
#[require(Collider, Sprite, Transform)]
struct Wall;
impl Wall {
    fn new(location: WallLocation) -> (Wall, Sprite, Transform) {
        (
            Wall,
            Sprite::from_color(Color::WHITE, Vec2::ONE),
            Transform {
                translation: location.position().extend(0.0),
                scale: location.size().extend(1.0),
                ..default()
            },
        )
    }
}

enum WallLocation {
    Left,
    Right,
    Bottom,
    Top,
}
impl WallLocation {
    fn position(&self) -> Vec2 {
        match self {
            WallLocation::Left => Vec2::new(-BORDER_WIDTH / 2., 0.),
            WallLocation::Right => Vec2::new(BORDER_WIDTH / 2., 0.),
            WallLocation::Top => Vec2::new(0., BORDER_HEIGHT / 2.),
            WallLocation::Bottom => Vec2::new(0., -BORDER_HEIGHT / 2.),
        }
    }
    fn size(&self) -> Vec2 {
        match self {
            WallLocation::Left | WallLocation::Right => {
                Vec2::new(BORDER_PADDING, BORDER_HEIGHT + BORDER_PADDING)
            }
            WallLocation::Top | WallLocation::Bottom => {
                Vec2::new(BORDER_WIDTH + BORDER_PADDING, BORDER_PADDING)
            }
        }
    }
}

fn set_fixed_time(mut time: ResMut<Time<Fixed>>) {
    time.set_timestep_hz(128.);
}

fn spawn(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2d);

    commands.spawn(Ball::new(
        Vec2::new(100., 100.),
        &mut meshes,
        &mut materials,
    ));
    commands.spawn(Ball::new(
        Vec2::new(-100., 100.),
        &mut meshes,
        &mut materials,
    ));
    commands.spawn(Ball::new(
        Vec2::new(-100., 60.),
        &mut meshes,
        &mut materials,
    ));

    commands.spawn(Wall::new(WallLocation::Bottom));
    commands.spawn(Wall::new(WallLocation::Right));
    commands.spawn(Wall::new(WallLocation::Left));
    commands.spawn(Wall::new(WallLocation::Top));
}

fn apply_velocity_ball(mut query: Query<(&mut Transform, &Velocity), With<Ball>>, time: Res<Time>) {
    for (mut t, v) in &mut query {
        t.translation.x += v.0.x * time.delta_secs();
        t.translation.y += v.0.y * time.delta_secs();
    }
}

fn apply_acceleration(mut query: Query<(&mut Velocity, &Acceleration)>, time: Res<Time>) {
    for (mut v, a) in query.iter_mut() {
        v.0 += a.0 * time.delta_secs()
    }
}

/// Checks collision between a moving and static object
fn check_collision(
    mut ball_query: Query<(&mut Velocity, &Transform), With<Ball>>,
    collider_query: Query<&Transform, With<Collider>>,
) {
    for (i, (mut ball_velocity, ball_transform)) in ball_query.iter_mut().enumerate() {
        for collider_transform in &collider_query {
            let collision = box_collision(
                BoundingCircle::new(ball_transform.translation.truncate(), BALL_RADIUS),
                Aabb2d::new(
                    collider_transform.translation.truncate(),
                    collider_transform.scale.truncate() / 2.,
                ),
            );

            if let Some(collision) = collision {
                let mut reflect_x = false;
                let mut reflect_y = false;

                match collision {
                    Collision::Left => reflect_x = ball_velocity.x > 0.0,
                    Collision::Right => reflect_x = ball_velocity.x < 0.0,
                    Collision::Up => reflect_y = ball_velocity.y < 0.0,
                    Collision::Down => reflect_y = ball_velocity.y > 0.0,
                }

                if reflect_x {
                    ball_velocity.x = -ball_velocity.x
                }
                if reflect_y {
                    ball_velocity.y = -ball_velocity.y
                }
            }
        }
    }
}
fn check_ball_collision(
    mut ball1_query: Query<(&mut Velocity, &Transform), With<Ball>>,
    ball2_query: Query<&Transform, With<Ball>>,
) {
    for (i, (mut ball1_velocity, ball1_transform)) in ball1_query.iter_mut().enumerate() {
        for (l, ball2_transform) in ball2_query.iter().enumerate() {
            if i == l {
                continue;
            }

            let collision = ball_collision(
                BoundingCircle::new(ball1_transform.translation.truncate(), BALL_RADIUS),
                BoundingCircle::new(ball2_transform.translation.truncate(), BALL_RADIUS),
            );
            if let Some(collision) = collision {
                debug!("Bounced ball {}", i);
                let mut reflect_x = false;
                let mut reflect_y = false;

                match collision {
                    Collision::Left => reflect_x = ball1_velocity.x > 0.0,
                    Collision::Right => reflect_x = ball1_velocity.x < 0.0,
                    Collision::Up => reflect_y = ball1_velocity.y < 0.0,
                    Collision::Down => reflect_y = ball1_velocity.y > 0.0,
                }

                if reflect_x {
                    ball1_velocity.x = -ball1_velocity.x;
                }
                if reflect_y {
                    ball1_velocity.y = -ball1_velocity.y;
                }
            }
        }
    }
}

#[derive(Debug)]
enum Collision {
    Up,
    Down,
    Left,
    Right,
}
fn box_collision(ball: BoundingCircle, bounding_box: Aabb2d) -> Option<Collision> {
    if !ball.intersects(&bounding_box) {
        return None;
    }

    let closest = bounding_box.closest_point(ball.center());
    let offset = ball.center() - closest;
    let side = if offset.x.abs() > offset.y.abs() {
        if offset.x < 0. {
            Collision::Left
        } else {
            Collision::Right
        }
    } else if offset.y <= 0. {
        Collision::Down
    } else {
        Collision::Up
    };
    Some(side)
}
fn ball_collision(ball1: BoundingCircle, ball2: BoundingCircle) -> Option<Collision> {
    if !ball1.intersects(&ball2) {
        return None;
    }
    let closest = ball2.closest_point(ball1.center());
    let offset = ball1.center() - closest;

    let side = if offset.x.abs() > offset.y.abs() {
        if offset.x < 0. {
            Collision::Left
        } else {
            Collision::Right
        }
    } else if offset.y <= 0. {
        Collision::Down
    } else {
        Collision::Up
    };
    Some(side)
}
