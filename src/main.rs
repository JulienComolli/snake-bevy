use bevy::{prelude::*, time::Stopwatch};
use std::process::exit;

use rand::Rng;

const SNAKE_HEAD_COLOR: Color = Color::VIOLET;
const SNAKE_BODY_COLOR: Color = Color::VIOLET;
const FOOD_COLOR: Color = Color::CRIMSON;
const AREA_COLOR: Color = Color::SEA_GREEN;

const AREA_HEIGHT: i32 = 31;
const AREA_WIDTH: i32 = 53;

const SEGMENT_SIZE: i32 = 20;

const DEFAULT_STEP: u128 = 80; // In Ms
const MIN_STEP: u128 = 32;

#[derive(PartialEq, Eq, Copy, Clone)]
enum EnumDirection {
    UP = 0,
    DOWN = 1,
    LEFT = 2,
    RIGHT = 3,
}

#[derive(Component)]
struct SnakeHead;

#[derive(Component)]
struct SnakeBody;

#[derive(Component)]
struct Food;

#[derive(Component)]
struct Position {
    x: i32,
    y: i32,
}

#[derive(Component)]
struct Direction(EnumDirection);

#[derive(Component)]
struct NextDirection(EnumDirection);

#[derive(Component)]
struct Tile;

#[derive(Resource)]
struct GameState {
    just_ate: bool,
    must_grow: bool,
    last_move: Stopwatch,
    step: u128,
    length: u32,
}

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Snakeuh !".into(),
                    name: Some("Snakeuh !".into()),
                    resolution: (
                        (AREA_WIDTH * SEGMENT_SIZE) as f32,
                        (AREA_HEIGHT * SEGMENT_SIZE) as f32,
                    )
                        .into(),
                    ..default()
                }),
                ..default()
            }),
        )
        .insert_resource(ClearColor(AREA_COLOR))
        .insert_resource(GameState {
            just_ate: false,
            must_grow: false,
            last_move: Stopwatch::new(),
            step: DEFAULT_STEP,
            length: 1,
        })
        .add_systems(Startup, setup)
        .add_systems(Update, change_direction)
        .add_systems(Update, (check_death, check_eat))
        .add_systems(Update, (respawn_food, draw_food).chain())
        .add_systems(
            Update,
            (
                move_snake, //.run_if(on_timer(Duration::from_millis(STEP))),
                draw_snake,
            )
                .chain(), // Chaining need to avoid the body part from spawning
                          // then getting moved to the queue (weird flickering)
        )
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(
                    (SEGMENT_SIZE + 3) as f32,
                    (SEGMENT_SIZE + 3) as f32,
                )),
                color: SNAKE_HEAD_COLOR,
                ..Default::default()
            },
            transform: Transform::from_xyz(0., 0., 2.),
            ..default()
        },
        SnakeHead,
        Direction(EnumDirection::UP),
        NextDirection(EnumDirection::UP),
        Position { x: 0, y: 0 },
    ));

    spawn_food(commands, 1, 1);
}

fn spawn_food(mut commands: Commands, xpos: i32, ypos: i32) {
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(SEGMENT_SIZE as f32, SEGMENT_SIZE as f32)),
                color: FOOD_COLOR,
                ..Default::default()
            },
            transform: Transform::from_xyz(0., 0., 1.),
            ..default()
        },
        Food,
        Position { x: xpos, y: ypos },
    ));
}

fn spawn_body_part(mut commands: Commands, xpos: i32, ypos: i32) {
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(
                    (SEGMENT_SIZE - 1) as f32,
                    (SEGMENT_SIZE - 1) as f32,
                )),
                color: SNAKE_BODY_COLOR,
                ..Default::default()
            },
            transform: Transform::from_xyz(0., 0., 1.),
            ..default()
        },
        SnakeBody,
        Position { x: xpos, y: ypos },
    ));
}

fn change_direction(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    query_direction: Query<&Direction, With<SnakeHead>>,
    mut query_next_direction: Query<&mut NextDirection, With<SnakeHead>>,
) {
    let dir = query_direction.single();
    let mut next_dir = query_next_direction.single_mut();

    if dir.0 != EnumDirection::DOWN && keyboard_input.just_pressed(KeyCode::ArrowUp) {
        next_dir.0 = EnumDirection::UP;
    }
    if dir.0 != EnumDirection::UP && keyboard_input.just_pressed(KeyCode::ArrowDown) {
        next_dir.0 = EnumDirection::DOWN;
    }
    if dir.0 != EnumDirection::RIGHT && keyboard_input.just_pressed(KeyCode::ArrowLeft) {
        next_dir.0 = EnumDirection::LEFT;
    }
    if dir.0 != EnumDirection::LEFT && keyboard_input.just_pressed(KeyCode::ArrowRight) {
        next_dir.0 = EnumDirection::RIGHT;
    }

    // TODO: migrate this in another function
    if keyboard_input.pressed(KeyCode::Escape) {
        exit(0);
    }
}

fn move_snake(
    mut query: Query<(&mut Position, &mut Direction, &NextDirection), With<SnakeHead>>,
    mut query_bodies: Query<&mut Position, (With<SnakeBody>, Without<SnakeHead>)>,
    commands: Commands,
    mut game_state: ResMut<GameState>,
    time: Res<Time>,
) {
    game_state.last_move.tick(time.delta());
    println!("{}", game_state.last_move.elapsed().as_millis());

    if game_state.last_move.elapsed().as_millis() > game_state.step {
        game_state.last_move.reset();
    } else {
        return;
    }

    let mut snake_head = query.single_mut();

    let mut previous_position = Position {
        x: snake_head.0.x,
        y: snake_head.0.y,
    };

    snake_head.1 .0 = snake_head.2 .0;

    if EnumDirection::UP == snake_head.1 .0 {
        snake_head.0.y += 1;
        if snake_head.0.y > AREA_HEIGHT / 2 {
            snake_head.0.y = -(AREA_HEIGHT / 2);
        }
    }
    if EnumDirection::DOWN == snake_head.1 .0 {
        snake_head.0.y -= 1;
        if snake_head.0.y < -(AREA_HEIGHT / 2) {
            snake_head.0.y = AREA_HEIGHT / 2;
        }
    }
    if EnumDirection::LEFT == snake_head.1 .0 {
        snake_head.0.x -= 1;
        if snake_head.0.x < -(AREA_WIDTH / 2) {
            snake_head.0.x = AREA_WIDTH / 2;
        }
    }
    if EnumDirection::RIGHT == snake_head.1 .0 {
        snake_head.0.x += 1;
        if snake_head.0.x > AREA_WIDTH / 2 {
            snake_head.0.x = -(AREA_WIDTH / 2);
        }
    }

    for mut b in query_bodies.iter_mut() {
        let tx = b.x;
        let ty = b.y;
        b.x = previous_position.x;
        b.y = previous_position.y;
        previous_position.x = tx;
        previous_position.y = ty;
    }

    if game_state.must_grow {
        spawn_body_part(commands, previous_position.x, previous_position.y);
        game_state.must_grow = false;
    }
}

fn draw_snake(
    mut query: Query<(&mut Transform, &Position, &Direction), With<SnakeHead>>,
    mut query_bodies: Query<(&mut Transform, &Position), (With<SnakeBody>, Without<SnakeHead>)>,
) {
    let mut snake_head = query.single_mut();
    snake_head.0.translation.x = (snake_head.1.x * SEGMENT_SIZE) as f32;
    snake_head.0.translation.y = (snake_head.1.y * SEGMENT_SIZE) as f32;

    for mut body_part in query_bodies.iter_mut() {
        body_part.0.translation.x = (body_part.1.x * SEGMENT_SIZE) as f32;
        body_part.0.translation.y = (body_part.1.y * SEGMENT_SIZE) as f32;
    }
}

fn draw_food(mut query: Query<(&mut Transform, &Position), With<Food>>) {
    for mut food in query.iter_mut() {
        food.0.translation.x = (food.1.x * SEGMENT_SIZE) as f32;
        food.0.translation.y = (food.1.y * SEGMENT_SIZE) as f32;
    }
}

fn respawn_food(mut game_state: ResMut<GameState>, commands: Commands) {
    if game_state.just_ate {
        let mut rng = rand::thread_rng();
        let xpos = rng.gen_range(-(AREA_WIDTH / 2 - 1)..(AREA_WIDTH / 2 - 1)) as i32;
        let ypos = rng.gen_range(-(AREA_HEIGHT / 2 - 1)..(AREA_HEIGHT / 2 - 1)) as i32;
        spawn_food(commands, xpos, ypos);
        game_state.just_ate = false;
    }
}

fn check_eat(
    query_head: Query<&Position, With<SnakeHead>>,
    query_foods: Query<(Entity, &Position), With<Food>>,
    mut game_state: ResMut<GameState>,
    mut commands: Commands,
) {
    let head = query_head.get_single().unwrap();

    for (food, pos) in query_foods.iter() {
        if pos.x == head.x && pos.y == head.y {
            game_state.just_ate = true;
            game_state.must_grow = true;
            game_state.length = game_state.length + 1;
            if game_state.step > MIN_STEP {
                game_state.step -= 2;
            }
            commands.entity(food).despawn();
        }
    }
}

fn check_death(
    query_head: Query<&Position, With<SnakeHead>>,
    query_bodies: Query<&Position, With<SnakeBody>>,
) {
    let head = query_head.get_single().unwrap();

    for body_part in query_bodies.iter() {
        if body_part.x == head.x && body_part.y == head.y {
            exit(0);
        }
    }
}
