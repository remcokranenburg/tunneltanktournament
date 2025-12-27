use crate::components::{CameraPosition, Player};
use bevy::{
    camera::{ScalingMode, Viewport},
    prelude::*,
    window::{WindowResized, WindowTheme},
};
use bevy_ggrs::{LocalPlayers, prelude::*};
use bevy_matchbox::{MatchboxSocket, prelude::PeerId};
use fastrand::Rng;

const NUM_PLAYERS: usize = 2;
const MAP_WIDTH: usize = 1000;
const MAP_HEIGHT: usize = 500;

const COLOR_BLUE: Color = Color::srgb(0.173, 0.173, 1.0);
const COLOR_BLUE_DARK: Color = Color::srgb(0.0, 0.0, 0.714);
const COLOR_GREEN: Color = Color::srgb(0.0, 1.0, 0.0);
const COLOR_GREEN_DARK: Color = Color::srgb(0.0, 0.667, 0.0);
const COLOR_TERRAIN_LIGHT: Color = Color::srgb(0.765, 0.475, 0.188);
const COLOR_TERRAIN_DARK: Color = Color::srgb(0.729, 0.349, 0.016);
const COLOR_ROCK: Color = Color::srgb(0.604, 0.604, 0.604);
const COLOR_ENERGY: Color = Color::srgb(0.915, 0.922, 0.110);
const COLOR_SHIELD: Color = Color::srgb(0.157, 0.953, 0.953);
const COLOR_UI: Color = Color::srgb(0.396, 0.396, 0.396);

mod components;
mod input;

type Config = GgrsConfig<u8, PeerId>;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    fit_canvas_to_parent: true,
                    prevent_default_event_handling: false,
                    title: "Tunnel Tank Tournament".to_string(),
                    window_theme: Some(WindowTheme::Dark),
                    ..default()
                }),
                ..default()
            }),
            GgrsPlugin::<Config>::default(),
        ))
        .rollback_component_with_clone::<Transform>()
        .insert_resource(ClearColor(Color::srgb(0.0, 0.0, 0.0)))
        .add_systems(
            Startup,
            (
                spawn_camera,
                spawn_map,
                spawn_players,
                start_matchbox_socket,
            ),
        )
        .add_systems(Update, set_camera_viewports)
        .add_systems(FixedUpdate, (wait_for_players, camera_follow))
        .add_systems(ReadInputs, input::read_local_inputs)
        .add_systems(GgrsSchedule, move_players)
        .run();
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::AutoMax {
                max_width: 76.0,
                max_height: 76.0,
            },
            ..OrthographicProjection::default_2d()
        }),
        CameraPosition {
            pos: UVec2::new(0, 0),
        },
    ));

    commands.spawn((
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::AutoMax {
                max_width: 76.0,
                max_height: 76.0,
            },
            ..OrthographicProjection::default_2d()
        }),
        CameraPosition {
            pos: UVec2::new(1, 0),
        },
    ));
}

fn spawn_map(mut commands: Commands) {
    let mut rng = Rng::with_seed(42);

    for x in 0..MAP_WIDTH {
        for y in 0..MAP_HEIGHT {
            let color = if rng.bool() {
                COLOR_TERRAIN_LIGHT
            } else {
                COLOR_TERRAIN_DARK
            };

            commands.spawn((
                Transform::from_translation(Vec3::new(
                    x as f32 - MAP_WIDTH as f32 / 2.0,
                    y as f32 - MAP_HEIGHT as f32 / 2.0,
                    0.0,
                )),
                Sprite {
                    color,
                    custom_size: Some(Vec2::new(1.0, 1.0)),
                    ..default()
                },
            ));
        }
    }
}

fn set_camera_viewports(
    windows: Query<&Window>,
    mut window_resized_reader: MessageReader<WindowResized>,
    mut query: Query<(&CameraPosition, &mut Camera)>,
) {
    for window_resized in window_resized_reader.read() {
        let window = windows.get(window_resized.window).unwrap();
        let max_width = window.physical_width() / 2;
        let max_height = window.physical_height();
        let size = window.physical_size() / 2;

        for (camera_position, mut camera) in &mut query {
            camera.viewport = Some(Viewport {
                physical_position: camera_position.pos * size,
                physical_size: size,
                ..default()
            });
        }
    }
}

fn camera_follow(
    players: Query<(&Player, &Transform)>,
    mut cameras: Query<(&mut Transform, &CameraPosition), (With<Camera2d>, Without<Player>)>,
) {
    for (player, player_transform) in &players {
        for (mut transform, position) in &mut cameras {
            if position.pos.x as usize != player.id {
                // skip if the camera is for another player
                continue;
            }

            let pos = player_transform.translation;
            transform.translation.x = pos.x;
            transform.translation.y = pos.y;
        }
    }
}

fn spawn_players(mut commands: Commands) {
    commands
        .spawn((
            Player { id: 0 },
            Transform::from_translation(Vec3::new(-20., 0., 10.)),
            Sprite {
                color: COLOR_BLUE,
                custom_size: Some(Vec2::new(5., 7.)),
                ..default()
            },
        ))
        .add_rollback();

    commands
        .spawn((
            Player { id: 1 },
            Transform::from_translation(Vec3::new(20., 0., 10.)),
            Sprite {
                color: COLOR_GREEN,
                custom_size: Some(Vec2::new(5., 7.)),
                ..default()
            },
        ))
        .add_rollback();
}

fn move_players(
    mut players: Query<(&mut Transform, &Player)>,
    inputs: Res<PlayerInputs<Config>>,
    time: Res<Time>,
) {
    for (mut transform, player) in &mut players {
        let (input, _) = inputs[player.id];
        let direction = input::direction(input);

        if direction == Vec2::ZERO {
            continue;
        }

        let move_speed = 7.;
        let move_delta = direction * move_speed * time.delta_secs();

        let old_pos = transform.translation.xy();
        let limit = Vec2::new(MAP_WIDTH as f32 / 2.0 - 0.5, MAP_HEIGHT as f32 / 2.0 - 0.5);
        let new_pos = (old_pos + move_delta).clamp(-limit, limit);

        transform.translation.x = new_pos.x;
        transform.translation.y = new_pos.y;
    }
}

fn start_matchbox_socket(mut commands: Commands) {
    let room_url = "ws://home.burgsoft.nl:3536/tunneltanktournament?next=2";
    info!("Connecting to matchbox room at: {}", room_url);
    commands.insert_resource(MatchboxSocket::new_unreliable(room_url));
}

fn wait_for_players(mut commands: Commands, mut socket: ResMut<MatchboxSocket>) {
    if socket.get_channel(0).is_err() {
        return; // skip system: we've already started
    }

    socket.update_peers();
    let players = socket.players();

    if players.len() < NUM_PLAYERS {
        return; // wait for more players
    }

    info!("All players connected, starting game!");

    // create a GGRS P2P session
    let mut session_builder = SessionBuilder::<Config>::new()
        .with_num_players(players.len())
        .with_input_delay(2);

    for (i, player) in players.into_iter().enumerate() {
        session_builder = session_builder
            .add_player(player, i)
            .expect("failed to add player");
    }

    // move the channel out of the socket (required because GGRS takes ownership of it)
    let channel = socket.take_channel(0).unwrap();

    // start the GGRS session
    let ggrs_session = session_builder
        .start_p2p_session(channel)
        .expect("failed to start session");

    commands.insert_resource(Session::P2P(ggrs_session));
}
