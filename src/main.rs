use crate::{
    args::Args,
    components::{Bullet, BulletReady, CameraPosition, MoveDir, Player, checksum_transform},
    input::fire,
};
use bevy::{
    asset::AssetMetaCheck,
    camera::{ScalingMode, Viewport},
    prelude::*,
    render::sync_world::SyncToRenderWorld,
    window::WindowTheme,
};
use bevy_asset_loader::prelude::*;
use bevy_ecs_tilemap::{FrustumCulling, prelude::*};
use bevy_ggrs::{ggrs::DesyncDetection, prelude::*};
use bevy_matchbox::{MatchboxSocket, prelude::PeerId};
use clap::Parser;
use fastrand::Rng;

const NUM_PLAYERS: usize = 2;
const TERRAIN_WIDTH: u32 = 1000;
const TERRAIN_HEIGHT: u32 = 500;

enum TerrainTextureIndex {
    Dark,
    Light,
}

impl Into<TileTextureIndex> for TerrainTextureIndex {
    fn into(self) -> TileTextureIndex {
        TileTextureIndex(self as u32)
    }
}

// const COLOR_BLUE: Color = Color::srgb(0.173, 0.173, 1.0);
// const COLOR_BLUE_DARK: Color = Color::srgb(0.0, 0.0, 0.714);
// const COLOR_GREEN: Color = Color::srgb(0.0, 1.0, 0.0);
// const COLOR_GREEN_DARK: Color = Color::srgb(0.0, 0.667, 0.0);
// const COLOR_TERRAIN_LIGHT: Color = Color::srgb(0.765, 0.475, 0.188);
// const COLOR_TERRAIN_DARK: Color = Color::srgb(0.729, 0.349, 0.016);
// const COLOR_ROCK: Color = Color::srgb(0.604, 0.604, 0.604);
// const COLOR_ENERGY: Color = Color::srgb(0.915, 0.922, 0.110);
// const COLOR_SHIELD: Color = Color::srgb(0.157, 0.953, 0.953);
// const COLOR_UI: Color = Color::srgb(0.396, 0.396, 0.396);
const COLOR_BACKGROUND: Color = Color::srgb(0.0, 0.0, 0.179);

const SPEED_MOVE_STANDARD: f32 = 7.0;
const SPEED_BULLET: f32 = 48.0;

const PLAYER_RADIUS: f32 = 2.5;
const BULLET_RADIUS: f32 = 0.5;

mod args;
mod components;
mod input;

type Config = GgrsConfig<u8, PeerId>;

#[derive(AssetCollection, Resource)]
struct ImageAssets {
    #[asset(path = "bullet.png")]
    bullet: Handle<Image>,
    #[asset(path = "tankblue.png")]
    tank_blue: Handle<Image>,
    #[asset(path = "tankgreen.png")]
    tank_green: Handle<Image>,
    #[asset(path = "terrain.png")]
    terrain: Handle<Image>,
}

#[derive(States, Debug, Hash, PartialEq, Eq, Clone, Default)]
enum GameState {
    #[default]
    AssetLoading,
    Matchmaking,
    InGame,
}

fn main() {
    let args = Args::parse();

    eprintln!("{args:#?}");

    App::new()
        .add_plugins((
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        fit_canvas_to_parent: true,
                        prevent_default_event_handling: false,
                        title: "Tunnel Tank Tournament".to_string(),
                        window_theme: Some(WindowTheme::Dark),
                        ..default()
                    }),
                    ..default()
                })
                .set(AssetPlugin {
                    meta_check: AssetMetaCheck::Never,
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
            GgrsPlugin::<Config>::default(),
            TilemapPlugin,
        ))
        .init_state::<GameState>()
        .add_loading_state(
            LoadingState::new(GameState::AssetLoading)
                .load_collection::<ImageAssets>()
                .continue_to_state(GameState::Matchmaking),
        )
        .rollback_component_with_clone::<Transform>()
        .rollback_component_with_clone::<Sprite>()
        .rollback_component_with_copy::<Player>()
        .rollback_component_with_copy::<Bullet>()
        .rollback_component_with_copy::<BulletReady>()
        .rollback_component_with_copy::<MoveDir>()
        // Tilemap bundle components
        .rollback_component_with_copy::<TilemapGridSize>()
        .rollback_component_with_copy::<TilemapType>()
        .rollback_component_with_copy::<TilemapSize>()
        .rollback_component_with_copy::<TilemapSpacing>()
        .rollback_component_with_clone::<TileStorage>()
        .rollback_component_with_clone::<TilemapTexture>()
        .rollback_component_with_copy::<TilemapTileSize>()
        .rollback_component_with_copy::<Transform>()
        .rollback_component_with_copy::<GlobalTransform>()
        .rollback_component_with_copy::<TilemapRenderSettings>()
        .rollback_component_with_copy::<Visibility>()
        .rollback_component_with_copy::<InheritedVisibility>()
        .rollback_component_with_copy::<ViewVisibility>()
        .rollback_component_with_copy::<FrustumCulling>()
        .rollback_component_with_clone::<MaterialTilemapHandle<StandardTilemapMaterial>>()
        .rollback_component_with_copy::<SyncToRenderWorld>()
        .rollback_component_with_copy::<TilemapAnchor>()
        // Tile components
        .rollback_component_with_copy::<TilePos>()
        .rollback_component_with_copy::<TileTextureIndex>()
        .rollback_component_with_copy::<TilemapId>()
        .rollback_component_with_copy::<TileVisible>()
        .rollback_component_with_copy::<TileFlip>()
        .rollback_component_with_copy::<TileColor>()
        .rollback_component_with_copy::<TilePosOld>()
        .checksum_component::<Transform>(checksum_transform)
        .insert_resource(args)
        .insert_resource(ClearColor(COLOR_BACKGROUND))
        .add_systems(
            OnEnter(GameState::Matchmaking),
            (
                spawn_camera,
                spawn_terrain,
                start_matchbox_socket.run_if(p2p_mode),
            )
                .chain(),
        )
        .add_systems(OnEnter(GameState::InGame), spawn_players)
        .add_systems(Update, (set_camera_viewports, camera_follow))
        .add_systems(
            FixedUpdate,
            (
                wait_for_players.run_if(p2p_mode),
                start_synctest_session.run_if(synctest_mode),
                input::read_unsynced_inputs,
            )
                .run_if(in_state(GameState::Matchmaking)),
        )
        .add_systems(
            FixedUpdate,
            handle_ggrs_events.run_if(in_state(GameState::InGame)),
        )
        .add_systems(ReadInputs, input::read_local_inputs)
        .add_systems(
            GgrsSchedule,
            (
                move_players,
                reload_bullet,
                fire_bullets,
                move_bullet,
                destroy_players,
                destroy_terrain,
            )
                .chain(),
        )
        .run();
}

fn synctest_mode(args: Res<Args>) -> bool {
    args.synctest
}

fn p2p_mode(args: Res<Args>) -> bool {
    !args.synctest
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera {
            order: 0,
            clear_color: ClearColorConfig::Custom(Color::BLACK),
            ..default()
        },
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
        Camera {
            order: 1,
            clear_color: ClearColorConfig::Custom(Color::BLACK),
            ..default()
        },
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

fn spawn_terrain(mut commands: Commands, images: Res<ImageAssets>) {
    let mut rng = Rng::with_seed(42);

    let map_size = TilemapSize {
        x: TERRAIN_WIDTH,
        y: TERRAIN_HEIGHT,
    };

    let tilemap_entity = commands.spawn_empty().id();
    let mut tile_storage = TileStorage::empty(map_size);

    for x in 0..TERRAIN_WIDTH {
        for y in 0..TERRAIN_HEIGHT {
            let index = if rng.bool() {
                TerrainTextureIndex::Light
            } else {
                TerrainTextureIndex::Dark
            };

            let tile_pos = TilePos { x, y };

            let tile_entity = commands
                .spawn(TileBundle {
                    texture_index: index.into(),
                    position: tile_pos,
                    tilemap_id: TilemapId(tilemap_entity),
                    ..default()
                })
                .id();

            tile_storage.set(&tile_pos, tile_entity);
        }
    }

    let tile_size = TilemapTileSize { x: 1.0, y: 1.0 };
    let grid_size = tile_size.into();
    let map_type = TilemapType::Square;

    commands
        .entity(tilemap_entity)
        .insert(TilemapBundle {
            grid_size,
            map_type,
            size: map_size,
            storage: tile_storage,
            texture: TilemapTexture::Single(images.terrain.clone()),
            tile_size,
            anchor: TilemapAnchor::Center,
            ..default()
        })
        .add_rollback();
}

fn set_camera_viewports(windows: Query<&Window>, mut query: Query<(&CameraPosition, &mut Camera)>) {
    for window in &windows {
        let viewport_size = window.physical_size().x / 2 - 15;
        let size = UVec2::splat(viewport_size);
        let offset_y = (window.physical_size().y - viewport_size) / 2 + 10;

        for (camera_position, mut camera) in &mut query {
            let offset = UVec2::new(10 * (camera_position.pos.x + 1), offset_y);

            camera.viewport = Some(Viewport {
                physical_position: camera_position.pos * size + offset,
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

fn spawn_players(mut commands: Commands, images: Res<ImageAssets>) {
    commands
        .spawn((
            Player { id: 0 },
            BulletReady(true),
            Transform::from_translation(Vec3::new(-40., 0., 10.)),
            Sprite {
                image: images.tank_blue.clone(),
                custom_size: Some(Vec2::new(5.0, 7.0)),
                ..default()
            },
            MoveDir(Vec2::Y),
        ))
        .add_rollback();

    commands
        .spawn((
            Player { id: 1 },
            BulletReady(true),
            Transform::from_translation(Vec3::new(40., 0., 10.)),
            Sprite {
                image: images.tank_green.clone(),
                custom_size: Some(Vec2::new(5.0, 7.0)),
                ..default()
            },
            MoveDir(Vec2::Y),
        ))
        .add_rollback();
}

fn move_players(
    mut players: Query<(&mut Transform, &Player, &mut MoveDir)>,
    inputs: Res<PlayerInputs<Config>>,
    time: Res<Time>,
) {
    for (mut transform, player, mut move_dir) in &mut players {
        let (input, _) = inputs[player.id];
        let direction = input::direction(input);

        if direction == Vec2::ZERO {
            continue;
        }

        move_dir.0 = direction;

        let move_delta = direction * SPEED_MOVE_STANDARD * time.delta_secs();

        let old_pos = transform.translation.xy();
        let limit = Vec2::new(
            TERRAIN_WIDTH as f32 / 2.0 - 0.5,
            TERRAIN_HEIGHT as f32 / 2.0 - 0.5,
        );
        let new_pos = (old_pos + move_delta).clamp(-limit, limit);

        transform.translation.x = new_pos.x;
        transform.translation.y = new_pos.y;
        transform.rotation = Quat::from_rotation_arc_2d(Vec2::Y, direction);
    }
}

fn start_synctest_session(
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
    args: Res<Args>,
) {
    info!("Starting synctest session");
    let num_players = 2;

    let mut session_builder = SessionBuilder::<Config>::new()
        .with_num_players(num_players)
        .with_input_delay(args.input_delay);

    for i in 0..num_players {
        session_builder = session_builder
            .add_player(PlayerType::Local, i)
            .expect("failed to add player");
    }

    let ggrs_session = session_builder
        .start_synctest_session()
        .expect("failed to start session");

    commands.insert_resource(bevy_ggrs::Session::SyncTest(ggrs_session));
    next_state.set(GameState::InGame);
}

fn start_matchbox_socket(mut commands: Commands) {
    let room_url = "wss://match.remcokranenburg.com/tunneltanktournament?next=2";
    info!("Connecting to matchbox room at: {}", room_url);
    commands.insert_resource(MatchboxSocket::new_unreliable(room_url));
}

fn wait_for_players(
    mut commands: Commands,
    mut socket: ResMut<MatchboxSocket>,
    mut next_state: ResMut<NextState<GameState>>,
    args: Res<Args>,
) {
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
        .with_desync_detection_mode(DesyncDetection::On { interval: 1 })
        .with_input_delay(args.input_delay);

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
    next_state.set(GameState::InGame);
}

fn fire_bullets(
    mut commands: Commands,
    inputs: Res<PlayerInputs<Config>>,
    images: Res<ImageAssets>,
    mut players: Query<(&Transform, &Player, &mut BulletReady, &MoveDir)>,
) {
    for (transform, player, mut bullet_ready, move_dir) in &mut players {
        if fire(inputs[player.id].0) && bullet_ready.0 {
            commands
                .spawn((
                    Bullet {
                        owner_id: player.id,
                    },
                    Transform::from_translation(transform.translation)
                        .with_rotation(Quat::from_rotation_arc_2d(Vec2::Y, move_dir.0)),
                    *move_dir,
                    Sprite {
                        image: images.bullet.clone(),
                        custom_size: Some(Vec2::new(1.0, 2.0)),
                        ..default()
                    },
                ))
                .add_rollback();

            bullet_ready.0 = false;
        }
    }
}

fn reload_bullet(
    inputs: Res<PlayerInputs<Config>>,
    mut players: Query<(&mut BulletReady, &Player)>,
) {
    for (mut can_fire, player) in players.iter_mut() {
        let (input, _) = inputs[player.id];
        if !fire(input) {
            can_fire.0 = true;
        }
    }
}

fn move_bullet(mut bullets: Query<(&mut Transform, &MoveDir), With<Bullet>>, time: Res<Time>) {
    for (mut transform, move_dir) in &mut bullets {
        let delta = move_dir.0 * SPEED_BULLET * time.delta_secs();
        transform.translation += delta.extend(0.0);
    }
}

fn destroy_players(
    mut commands: Commands,
    players: Query<(Entity, &Player, &Transform)>,
    bullets: Query<(&Bullet, &Transform)>,
) {
    for (entity, player, player_transform) in &players {
        for (bullet, bullet_transform) in &bullets {
            let distance = Vec2::distance(
                player_transform.translation.xy(),
                bullet_transform.translation.xy(),
            );
            if distance < PLAYER_RADIUS + BULLET_RADIUS && bullet.owner_id != player.id as usize {
                commands.entity(entity).despawn();
            }
        }
    }
}

fn destroy_terrain(
    players: Query<&Transform, With<Player>>,
    mut tile_storage: Query<&mut TileStorage>,
    mut visibility_query: Query<&mut TileVisible>,
) {
    for player_transform in &players {
        let player_tile = TilePos {
            x: (player_transform.translation.x + TERRAIN_WIDTH as f32 / 2.0).floor() as u32,
            y: (player_transform.translation.y + TERRAIN_HEIGHT as f32 / 2.0).floor() as u32,
        };

        let neighbors = get_neighbors_in_radius(&player_tile, 2);

        for tile_storage in &mut tile_storage {
            for neighbor in neighbors.iter() {
                if let Some(tile_entity) = tile_storage.get(neighbor) {
                    if let Ok(mut visibility) = visibility_query.get_mut(tile_entity) {
                        visibility.0 = false;
                    }
                }
            }
        }
    }
}

fn get_neighbors_in_radius(pos: &TilePos, radius: u32) -> Vec<TilePos> {
    let mut neighbors = Vec::new();

    for dx in -(radius as i32)..=(radius as i32) {
        for dy in -(radius as i32)..=(radius as i32) {
            let neighbor_x = pos.x as i32 + dx;
            let neighbor_y = pos.y as i32 + dy;

            if neighbor_x >= 0 && neighbor_y >= 0 {
                neighbors.push(TilePos {
                    x: neighbor_x as u32,
                    y: neighbor_y as u32,
                });
            }
        }
    }

    neighbors
}

fn handle_ggrs_events(mut session: ResMut<Session<Config>>) {
    if let Session::P2P(s) = session.as_mut() {
        for event in s.events() {
            match event {
                GgrsEvent::Disconnected { .. } | GgrsEvent::NetworkInterrupted { .. } => {
                    warn!("GGRS event: {event:?}")
                }
                GgrsEvent::DesyncDetected {
                    local_checksum,
                    remote_checksum,
                    frame,
                    ..
                } => {
                    error!(
                        "Desync on frame {frame}. Local checksum: {local_checksum:X}, remote checksum: {remote_checksum:X}"
                    );
                }
                _ => info!("GGRS event: {event:?}"),
            }
        }
    }
}
