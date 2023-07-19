use bevy::{prelude::*, sprite::collide_aabb::collide, utils::HashMap};
use rand::Rng;

const WALL_THICKNESS: f32 = 10.0;
// x coordinates
const LEFT_WALL: f32 = -455.;
const RIGHT_WALL: f32 = 455.;
// y coordinates
const BOTTOM_WALL: f32 = -305.;
const TOP_WALL: f32 = 305.;

const SCOREBOARD_FONT_SIZE: f32 = 40.0;
const SCOREBOARD_TEXT_PADDING: Val = Val::Px(5.0);

const GRID_SIZE: Vec3 = Vec3::new(30.0, 30.0, 0.0);
const GRID_WIDTH: f32 = 30.0;
const GRID_HEIGHT: f32 = 20.0;

const SNAKE_BODY_SIZE: Vec3 = Vec3::new(25.0, 25.0, 0.0);

const BACKGROUND_COLOR: Color = Color::rgb(0.9, 0.9, 0.9);
const WALL_COLOR: Color = Color::rgb(0.8, 0.8, 0.8);
const TEXT_COLOR: Color = Color::rgb(0.5, 0.5, 1.0);
const SCORE_COLOR: Color = Color::rgb(1.0, 0.5, 0.5);
const SNAKE_COLOR: Color = Color::rgb(0.7, 0.7, 0.7);
const FOOT_COLOR: Color = Color::ORANGE_RED;
const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Snake".into(),
                resizable: true,
                ..default()
            }),
            ..default()
        }))
        .add_state::<GameState>()
        .insert_resource(Scoreboard { score: 0 })
        .insert_resource(GameLevel::Simple)
        .insert_resource(SnakeBody {
            body: HashMap::new(),
            entities: Vec::new(),
        })
        .insert_resource(SnakeTailPosition {
            position: Vec3::ZERO,
        })
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        // Configure how frequently our gameplay systems are run
        .insert_resource(FixedTime::new_from_secs(1.))
        .add_event::<GameoverEvent>()
        .add_event::<GrowEvent>()
        .add_systems(Startup, setup)
        .add_systems(OnEnter(GameState::Menu), setup_menu)
        .add_systems(Update, menu.run_if(in_state(GameState::Menu)))
        .add_systems(OnExit(GameState::Menu), cleanup_menu)
        .add_systems(
            OnEnter(GameState::InGame),
            (setup_game, generate_foot.after(setup_game)),
        )
        .add_systems(
            FixedUpdate,
            (
                movement,
                check_for_collisions.after(movement),
                update_scoreboard.after(check_for_collisions),
                update_velocity.after(update_scoreboard),
                growth.after(check_for_collisions),
            )
                .run_if(in_state(GameState::InGame)),
        )
        .add_systems(
            Update,
            (change_direction, generate_foot).run_if(in_state(GameState::InGame)),
        )
        .add_systems(OnExit(GameState::InGame), cleanup_game)
        .run();
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Hash, States)]
enum GameState {
    #[default]
    Menu,
    InGame,
}

#[derive(Component)]
struct Menu;

#[derive(Component)]
enum MenuButtonAction {
    Simple,
    Regular,
    Hard,
}

#[derive(Component)]
struct SnakeHead;

#[derive(Component)]
struct SnakeBodyPart;

#[derive(Component)]
struct Foot;

#[derive(Component, Copy, Clone)]
struct Velocity(Vec3);

#[derive(Component)]
struct Collider;

#[derive(Event, Default)]
struct GrowEvent;

#[derive(Event, Default)]
struct GameoverEvent;

// This bundle is a collection of the components that define a "wall" in our game
#[derive(Bundle)]
struct WallBundle {
    // You can nest bundles inside of other bundles like this
    // Allowing you to compose their functionality
    sprite_bundle: SpriteBundle,
    collider: Collider,
}

/// Which side of the arena is this wall located on?
enum WallLocation {
    Left,
    Right,
    Bottom,
    Top,
}

impl WallLocation {
    fn position(&self) -> Vec2 {
        match self {
            WallLocation::Left => Vec2::new(LEFT_WALL, 0.),
            WallLocation::Right => Vec2::new(RIGHT_WALL, 0.),
            WallLocation::Bottom => Vec2::new(0., BOTTOM_WALL),
            WallLocation::Top => Vec2::new(0., TOP_WALL),
        }
    }

    fn size(&self) -> Vec2 {
        let arena_height = TOP_WALL - BOTTOM_WALL;
        let arena_width = RIGHT_WALL - LEFT_WALL;
        // Make sure we haven't messed up our constants
        assert!(arena_height > 0.0);
        assert!(arena_width > 0.0);

        match self {
            WallLocation::Left | WallLocation::Right => {
                Vec2::new(WALL_THICKNESS, arena_height + WALL_THICKNESS)
            }
            WallLocation::Bottom | WallLocation::Top => {
                Vec2::new(arena_width + WALL_THICKNESS, WALL_THICKNESS)
            }
        }
    }
}

impl WallBundle {
    // This "builder method" allows us to reuse logic across our wall entities,
    // making our code easier to read and less prone to bugs when we change the logic
    fn new(location: WallLocation) -> WallBundle {
        WallBundle {
            sprite_bundle: SpriteBundle {
                transform: Transform {
                    // We need to convert our Vec2 into a Vec3, by giving it a z-coordinate
                    // This is used to determine the order of our sprites
                    translation: location.position().extend(0.0),
                    scale: location.size().extend(1.0),
                    ..default()
                },
                sprite: Sprite {
                    color: WALL_COLOR,
                    ..default()
                },
                ..default()
            },
            collider: Collider,
        }
    }
}

#[derive(Resource)]
struct Scoreboard {
    score: usize,
}

#[derive(Resource)]
enum GameLevel {
    Simple,
    Regular,
    Hard,
}

#[derive(Resource)]
struct SnakeBody {
    body: HashMap<(i32, i32), bool>,
    entities: Vec<Entity>,
}

#[derive(Resource)]
struct SnakeTailPosition {
    position: Vec3,
}

// Add the game's entities to our world.
fn setup(mut commands: Commands) {
    // Camera.
    commands.spawn(Camera2dBundle::default());

    // Scoreboard
    commands.spawn(
        TextBundle::from_sections([
            TextSection::new(
                "Score: ",
                TextStyle {
                    font_size: SCOREBOARD_FONT_SIZE,
                    color: TEXT_COLOR,
                    ..default()
                },
            ),
            TextSection::from_style(TextStyle {
                font_size: SCOREBOARD_FONT_SIZE,
                color: SCORE_COLOR,
                ..default()
            }),
        ])
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: SCOREBOARD_TEXT_PADDING,
            left: SCOREBOARD_TEXT_PADDING,
            ..default()
        }),
    );

    // Walls
    commands.spawn(WallBundle::new(WallLocation::Left));
    commands.spawn(WallBundle::new(WallLocation::Right));
    commands.spawn(WallBundle::new(WallLocation::Bottom));
    commands.spawn(WallBundle::new(WallLocation::Top));
}

fn setup_menu(mut commands: Commands) {
    let button_style = Style {
        width: Val::Px(150.0),
        height: Val::Px(65.0),
        margin: UiRect::all(Val::Px(20.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };

    commands
        .spawn(NodeBundle {
            style: Style {
                // center button
                width: Val::Percent(100.),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            ..default()
        })
        .insert(Menu)
        .with_children(|parent| {
            for (action, text) in [
                (MenuButtonAction::Simple, "Simple"),
                (MenuButtonAction::Regular, "Regular"),
                (MenuButtonAction::Hard, "Hard"),
            ] {
                parent
                    .spawn((
                        ButtonBundle {
                            style: button_style.clone(),
                            background_color: NORMAL_BUTTON.into(),
                            ..default()
                        },
                        action,
                    ))
                    .with_children(|parent| {
                        parent.spawn(TextBundle::from_section(
                            text,
                            TextStyle {
                                font_size: 40.0,
                                color: Color::rgb(0.9, 0.9, 0.9),
                                ..default()
                            },
                        ));
                    });
            }
        });
}

fn menu(
    mut level: ResMut<GameLevel>,
    mut next_state: ResMut<NextState<GameState>>,
    mut interaction_query: Query<
        (&Interaction, &MenuButtonAction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, action, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();
                next_state.set(GameState::InGame);

                match action {
                    MenuButtonAction::Simple => *level = GameLevel::Simple,
                    MenuButtonAction::Regular => *level = GameLevel::Regular,
                    MenuButtonAction::Hard => *level = GameLevel::Hard,
                }
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
            }
        }
    }
}

fn cleanup_menu(mut commands: Commands, menu: Query<Entity, With<Menu>>) {
    let entity = menu.single();
    commands.entity(entity).despawn_recursive();
}

fn setup_game(
    mut commands: Commands,
    mut snake_body: ResMut<SnakeBody>,
    mut scoreboard: ResMut<Scoreboard>,
    mut snake_tail_position: ResMut<SnakeTailPosition>,
) {
    // reset resource data.
    scoreboard.score = 0;
    snake_body.body.clear();
    snake_body.entities.clear();

    // Snake
    let head_position = get_grid_position(1, 0);
    let tail_position = get_grid_position(0, 0);
    let head = commands
        .spawn(SpriteBundle {
            transform: Transform {
                scale: SNAKE_BODY_SIZE,
                translation: head_position,
                ..default()
            },
            sprite: Sprite {
                color: Color::WHITE,
                ..default()
            },
            ..default()
        })
        .insert(SnakeHead)
        .insert(SnakeBodyPart)
        .insert(Velocity(Vec3::new(1., 0., 0.)))
        .id();
    let tail = spawn_snake_body(&mut commands, tail_position);

    snake_body.body.insert((1, 0), true);
    snake_body.body.insert((0, 0), true);
    snake_body.entities.push(head);
    snake_body.entities.push(tail);

    snake_tail_position.position = tail_position;
}

fn change_direction(
    mut head: Query<&mut Velocity, With<SnakeHead>>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    let mut head_velocity = head.single_mut();

    let mut velocity = head_velocity.0.clone();
    if keyboard_input.pressed(KeyCode::Left) {
        velocity.x = -1.;
        velocity.y = 0.
    }
    if keyboard_input.pressed(KeyCode::Right) {
        velocity.x = 1.;
        velocity.y = 0.
    }
    if keyboard_input.pressed(KeyCode::Up) {
        velocity.x = 0.;
        velocity.y = 1.;
    }
    if keyboard_input.pressed(KeyCode::Down) {
        velocity.x = 0.;
        velocity.y = -1.
    }

    head_velocity.0 = velocity;
}

fn movement(
    mut actual_velocity: Local<Vec3>,
    mut snake_body: ResMut<SnakeBody>,
    mut snake_tail_position: ResMut<SnakeTailPosition>,
    mut snake_head: Query<&Velocity, With<SnakeHead>>,
    mut snae_body_parts: Query<&mut Transform, With<SnakeBodyPart>>,
) {
    // Caculate snake new position.
    let velocity = snake_head.single_mut();
    let head_entity = snake_body.entities[0];
    let head_transform = snae_body_parts.get_mut(head_entity).unwrap();

    if -velocity.0 != *actual_velocity {
        *actual_velocity = velocity.0;
    }

    let mut new_part_position = head_transform.translation + *actual_velocity * GRID_SIZE;

    snake_body
        .body
        .insert(get_grid_number(new_part_position), true);

    // Update snake body part position.
    for entity in snake_body.entities.iter() {
        if let Ok(mut transform) = snae_body_parts.get_mut(entity.clone()) {
            let prev_part_position = transform.translation;
            transform.translation = new_part_position;
            new_part_position = prev_part_position;

            snake_tail_position.position = new_part_position;
        }
    }

    snake_body
        .body
        .insert(get_grid_number(new_part_position), false);

    // println!("snake body part: {:?}", snake_body.body);
}

fn generate_foot(mut commands: Commands, snake_body: Res<SnakeBody>, query: Query<&Foot>) {
    if snake_body.entities.len() as f32 == GRID_WIDTH * GRID_HEIGHT {
        return;
    }
    // if there is no foot in the game, then generate one.
    if query.is_empty() {
        let mut rng = rand::thread_rng();
        let mut grid_position = (
            rng.gen_range(0..GRID_WIDTH as i32),
            rng.gen_range(0..GRID_HEIGHT as i32),
        );
        while snake_body.body.get(&grid_position).is_some_and(|t| *t) {
            grid_position = (
                rng.gen_range(0..GRID_WIDTH as i32),
                rng.gen_range(0..GRID_HEIGHT as i32),
            );
        }

        commands
            .spawn(SpriteBundle {
                transform: Transform {
                    translation: get_grid_position(
                        rng.gen_range(0..GRID_WIDTH as i32),
                        rng.gen_range(0..GRID_HEIGHT as i32),
                    ),
                    scale: SNAKE_BODY_SIZE,
                    ..default()
                },
                sprite: Sprite {
                    color: FOOT_COLOR,
                    ..default()
                },
                ..default()
            })
            .insert(Foot)
            .insert(Collider);
    }
}

fn update_scoreboard(scoreboard: Res<Scoreboard>, mut query: Query<&mut Text>) {
    let mut text = query.single_mut();
    text.sections[1].value = scoreboard.score.to_string();
}

fn update_velocity(
    scoreboard: Res<Scoreboard>,
    level: Res<GameLevel>,
    mut fixed_time: ResMut<FixedTime>,
) {
    let base_velocity: f32;
    match *level {
        GameLevel::Simple => base_velocity = 2.,
        GameLevel::Regular => base_velocity = 20.,
        GameLevel::Hard => base_velocity = 200.,
    }
    let period = (scoreboard.score as f32 + base_velocity).log(600.);
    *fixed_time = FixedTime::new_from_secs(1. - period);
}

fn growth(
    mut commands: Commands,
    mut snake_body: ResMut<SnakeBody>,
    snake_tail_position: ResMut<SnakeTailPosition>,
    mut event: EventReader<GrowEvent>,
) {
    for _ in event.iter() {
        let new_snake_tail_entity = spawn_snake_body(&mut commands, snake_tail_position.position);

        snake_body
            .body
            .insert(get_grid_number(snake_tail_position.position), true);
        snake_body.entities.push(new_snake_tail_entity);
    }
}

fn cleanup_game(
    mut commands: Commands,
    snake: Query<Entity, With<SnakeBodyPart>>,
    foot: Query<Entity, With<Foot>>,
    mut event: EventReader<GameoverEvent>,
) {
    for _ in event.iter() {
        for entity in snake.iter() {
            commands.entity(entity).despawn_recursive();
        }

        for entity in foot.iter() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn check_for_collisions(
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
    mut scoreboard: ResMut<Scoreboard>,
    mut snake_head_query: Query<(&Transform, &mut Velocity), With<SnakeHead>>,
    collider_query: Query<(Entity, &Transform, Option<&Foot>), With<Collider>>,
    mut gameover_events: EventWriter<GameoverEvent>,
    mut grow_events: EventWriter<GrowEvent>,
) {
    let (snake_head_transform, mut snake_head_velocity) = snake_head_query.single_mut();
    let snake_head_size = snake_head_transform.scale.truncate();

    // check snake head with walls, snake body and foot.
    for (collider_entity, transform, foot) in collider_query.iter() {
        let collision = collide(
            snake_head_transform.translation,
            snake_head_size,
            transform.translation,
            transform.scale.truncate(),
        );

        if let Some(_) = collision {
            // Eat foot.
            if foot.is_some() {
                scoreboard.score += 1;
                commands.entity(collider_entity).despawn();

                grow_events.send_default();
            } else {
                snake_head_velocity.0 = Vec3::ZERO;
                next_state.set(GameState::Menu);

                gameover_events.send_default();
            }
        }
    }
}

fn spawn_snake_body(commands: &mut Commands, position: Vec3) -> Entity {
    let entity = commands
        .spawn(SpriteBundle {
            transform: Transform {
                scale: SNAKE_BODY_SIZE,
                translation: position,
                ..default()
            },
            sprite: Sprite {
                color: SNAKE_COLOR,
                ..default()
            },
            ..default()
        })
        .insert(SnakeBodyPart)
        .insert(Collider)
        .id();
    entity
}

fn get_grid_position(x: i32, y: i32) -> Vec3 {
    Vec3::new(
        LEFT_WALL + (x as f32 + 0.5) * GRID_SIZE.x + WALL_THICKNESS / 2.,
        TOP_WALL - (y as f32 + 0.5) * GRID_SIZE.y - WALL_THICKNESS / 2.,
        0.,
    )
}

fn get_grid_number(position: Vec3) -> (i32, i32) {
    (
        ((position.x - LEFT_WALL - WALL_THICKNESS / 2.) / GRID_SIZE.x - 0.5) as i32,
        ((TOP_WALL - position.y - WALL_THICKNESS / 2.) / GRID_SIZE.y - 0.5) as i32,
    )
}
