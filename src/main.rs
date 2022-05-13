use bevy::{
    core::FixedTimestep,
    prelude::*
};
use rand::Rng;

const GAME_AREA_WIDTH_UNITS: i32 = 40;
const GAME_AREA_HEIGHT_UNITS: i32 = 30;
const GAME_AREA_STEP: f32 = 20.0;
const GAME_AREA_WIDTH: f32 = (GAME_AREA_WIDTH_UNITS+1) as f32 * GAME_AREA_STEP;
const GAME_AREA_HEIGHT: f32 = (GAME_AREA_HEIGHT_UNITS+1) as f32 * GAME_AREA_STEP;
const TIME_STEP: f32 = 5.0 / 60.0;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
enum GameState {
    Playing,
    GameOver,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_resource::<Snake>()
        .init_resource::<EatableSegment>()
        .insert_resource(ClearColor(Color::rgb(0.2, 0.2, 0.2)))
        .add_startup_system(base_setup)
        .add_system_set(SystemSet::on_enter(GameState::Playing).with_system(setup))
        .add_event::<CollisionEvent>()
        .add_event::<EatEvent>()
        .add_state(GameState::Playing)
        .add_system_set(
            SystemSet::new().with_run_criteria(FixedTimestep::step(TIME_STEP as f64))
            .with_system(move_snake)
            .with_system(check_collision.after(move_snake))
            .with_system(game_over.after(check_collision))
            .with_system(grow.after(check_collision))
        )
        .add_system(keyboard_input)
        .add_system_set(SystemSet::on_enter(GameState::GameOver).with_system(teardown))
        .run();
}

#[derive(PartialEq, Copy, Clone)]
enum Direction {
    Up,
    Down,
    Left,
    Right
}

impl Default for Direction {
    fn default() -> Self { Direction::Up }
}

struct SnakeSegment {
    x: i32,
    y: i32,
    visual: Entity
}

#[derive(Default)]
struct Snake {
    direction: Direction,
    segments: Vec<SnakeSegment>,
    grow: bool
}

struct EatableSegment {
    x: i32,
    y: i32,
    segment: Option<Entity>
}

impl Default for EatableSegment {
    fn default() -> Self { 
        EatableSegment {
            x: 0,
            y: 0,
            segment: None
        }
     }
}

#[derive(Default)]
struct CollisionEvent;

#[derive(Default)]
struct EatEvent;

fn base_setup(mut commands: Commands, mut snake: ResMut<Snake>) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.spawn_bundle(UiCameraBundle::default());
    snake.grow = false;

    commands.spawn_bundle( 
                SpriteBundle {
                            sprite: Sprite {
                                color: Color::rgb(0.1, 0.1, 0.1),
                                custom_size: Some(Vec2::new(GAME_AREA_WIDTH, GAME_AREA_HEIGHT)),
                                ..default()
                            },
                            transform: Transform {
                                translation: Vec3::new(0.0, 0.0, 0.0),
                                scale: Vec3::new(1.0, 1.0, 1.0),
                                ..default()
                            },
                            ..default()
                        });
}

fn setup(mut commands: Commands, mut snake: ResMut<Snake>, eatable_segment: ResMut<EatableSegment>) {
    snake.grow = false;
    snake.segments.push(
        SnakeSegment {
            x: 0,
            y: 0,
            visual: commands.spawn_bundle( 
                        SpriteBundle {
                            sprite: Sprite {
                                color: Color::rgb(0.8, 0.8, 0.8),
                                custom_size: Some(Vec2::new(GAME_AREA_STEP, GAME_AREA_STEP)),
                                ..default()
                            },
                            transform: Transform {
                                translation: Vec3::new(0.0, 0.0, 0.0),
                                scale: Vec3::new(1.0, 1.0, 1.0),
                                ..default()
                            },
                            ..default()
                        }).id()

        });
       create_eatable_segment(commands, snake, eatable_segment);
}

fn create_eatable_segment(mut commands: Commands, snake: ResMut<Snake>, mut eatable_segment: ResMut<EatableSegment>) {
    let mut rng = rand::thread_rng();
    loop {
        let ux = rng.gen_range(-(GAME_AREA_WIDTH_UNITS as i32) /2..(GAME_AREA_WIDTH_UNITS/2) as i32);
        let uy = rng.gen_range(-(GAME_AREA_HEIGHT_UNITS as i32) /2..(GAME_AREA_HEIGHT_UNITS/2) as i32);
        let mut is_ok_generated = true;
        for segment in snake.segments.iter() {
            if segment.x == ux || segment.y == uy {
                is_ok_generated = false;
                break;
            }
        }
        if is_ok_generated {
            eatable_segment.x = ux;
            eatable_segment.y = uy;
            eatable_segment.segment = Some(
                    commands.spawn_bundle( 
                        SpriteBundle {
                            sprite: Sprite {
                                color: Color::rgb(0.7, 0.7, 0.7),
                                custom_size: Some(Vec2::new(GAME_AREA_STEP, GAME_AREA_STEP)),
                                ..default()
                            },
                            transform: Transform {
                                translation: Vec3::new(ux as f32 * GAME_AREA_STEP, uy as f32 * GAME_AREA_STEP, 0.0),
                                scale: Vec3::new(1.0, 1.0, 1.0),
                                ..default()
                            },
                            ..default()
                        }).id()
                );
                return;
            }
    }
}

fn move_snake(mut commands: Commands, mut snake: ResMut<Snake>, mut query: Query<&mut Transform>, state: Res<State<GameState>>) {

    if *state.current() != GameState::Playing {
        return;
    }

    let direction = snake.direction;

    let mut previus_x: i32 = 0;
    let mut previus_y: i32 = 0;
    for (i, mut segment) in snake.segments.iter_mut().enumerate() {
        if i == 0 {
            previus_x = segment.x;
            previus_y = segment.y;    
            match direction {
                Direction::Up => segment.y += 1,
                Direction::Down => segment.y -= 1,
                Direction::Left => segment.x -= 1,
                Direction::Right => segment.x += 1
            }
            
            if segment.x > GAME_AREA_WIDTH_UNITS/2 {
                segment.x = - GAME_AREA_WIDTH_UNITS/2;
            }else if segment.x < -GAME_AREA_WIDTH_UNITS/2 {
                segment.x = GAME_AREA_WIDTH_UNITS/2;
            }else if segment.y > GAME_AREA_HEIGHT_UNITS/2 {
                segment.y = - GAME_AREA_HEIGHT_UNITS/2;
            }else if segment.y < -GAME_AREA_HEIGHT_UNITS/2 {
                segment.y = GAME_AREA_HEIGHT_UNITS/2;
            }

        }else {
            let tmp_x = segment.x;
            let tmp_y = segment.y;
            segment.x = previus_x;
            segment.y = previus_y;
            previus_x = tmp_x;
            previus_y = tmp_y;
        }
        query.get_mut(segment.visual).unwrap().translation.x = segment.x as f32 * GAME_AREA_STEP;
        query.get_mut(segment.visual).unwrap().translation.y = segment.y as f32 * GAME_AREA_STEP;
    }

    if snake.grow {
        snake.grow = false;

        snake.segments.push(
            SnakeSegment {
                x: previus_x,
                y: previus_y,
                visual: commands.spawn_bundle( 
                    SpriteBundle {
                        sprite: Sprite {
                            color: Color::rgb(0.8, 0.8, 0.8),
                            custom_size: Some(Vec2::new(GAME_AREA_STEP, GAME_AREA_STEP)),
                            ..default()
                        },
                        transform: Transform {
                            translation: Vec3::new(previus_x as f32 * GAME_AREA_STEP, previus_y as f32 * GAME_AREA_STEP, 0.0),
                            scale: Vec3::new(1.0, 1.0, 1.0),
                            ..default()
                        },
                        ..default()
                    }).id()
            }
        );
    }
}

fn keyboard_input(mut snake: ResMut<Snake>, key: Res<Input<KeyCode>>) {
    if key.pressed(KeyCode::Up) {
        snake.direction = Direction::Up;
    }else if key.just_pressed(KeyCode::Down) {
        snake.direction = Direction::Down;
    }else if key.just_pressed(KeyCode::Left) {
        snake.direction = Direction::Left;
    }else if key.just_pressed(KeyCode::Right) {
        snake.direction = Direction::Right;
    }
}

fn check_collision(snake: Res<Snake>, 
    eatable_segment: Res<EatableSegment>, 
    mut collision_events: EventWriter<CollisionEvent>,
    state: ResMut<State<GameState>>,
    mut eat_events: EventWriter<EatEvent>) {

        if *state.current() != GameState::Playing {
            return;
        }

        for segment_to_check in snake.segments.iter() {
            for segment in snake.segments.iter() {
                if segment.visual == segment_to_check.visual {
                    continue;
                }

                if segment.x == segment_to_check.x && segment.y == segment_to_check.y{
                    collision_events.send_default();
                    return;
                }
            }
            if eatable_segment.x == segment_to_check.x && eatable_segment.y == segment_to_check.y {
                eat_events.send_default();
            }
        }
}

fn game_over(mut collision_events: EventReader<CollisionEvent>, mut state: ResMut<State<GameState>>) {
    if collision_events.iter().count() > 0 {
        state.set(GameState::GameOver).unwrap();
    }
}

fn grow(mut eat_events: EventReader<EatEvent>, mut commands: Commands, mut snake: ResMut<Snake>, eatable_segment: ResMut<EatableSegment>) {
    if eat_events.iter().count() > 0 {
        snake.grow = true;
        commands.entity(eatable_segment.segment.unwrap()).despawn();
        create_eatable_segment(commands, snake, eatable_segment);
    }
}

fn teardown(mut commands: Commands, mut snake: ResMut<Snake>, mut eatable_segment: ResMut<EatableSegment>, mut state: ResMut<State<GameState>>) {
    if eatable_segment.segment.is_some() {
        commands.entity(eatable_segment.segment.unwrap()).despawn();
        eatable_segment.segment = None;
    }
    for segment in snake.segments.iter() {
        commands.entity(segment.visual).despawn();
    }
    snake.segments.clear();
    state.set(GameState::Playing).unwrap();
}