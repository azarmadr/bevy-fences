use bevy::{
    prelude::*,
    sprite::{Anchor, MaterialMesh2dBundle},
};
use bevy_mod_picking::prelude::*;
use fences::{board::Fence, solver::Idx, BoardGeom, FencesSolver};

pub struct BoardPlugin;
impl Plugin for BoardPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BoardAssets>()
            .add_plugins(DefaultPickingPlugins)
            .insert_resource(DebugPickingMode::Normal)
            .insert_resource(Board::new(
                "3#33    
0 0 0 y
1 0 2 y",
            ))
            .add_systems(
                Update,
                (
                    // edge_interaction_system,
                    // edge_click_system,
                    board_update_system,
                ),
            )
            .add_systems(Startup, board_setup);
    }
}

#[derive(Resource, Deref, DerefMut)]
pub struct Board(#[deref] fences::Board, usize);
impl Board {
    fn new(board_str: &str) -> Self {
        let mut b = Self(board_str.parse().unwrap(), 0);
        b.1 = b.0.moves().len();
        b
    }
}

#[derive(Component)]
struct Node(Idx);
#[derive(Component, Debug)]
enum Edge {
    H(Idx),
    V(Idx),
}
impl Edge {
    fn play_in(&self, board: &mut Board, v: bool) {
        match self {
            Edge::H(idx) => board.play(0, *idx, v, "Game Move".to_string()),
            Edge::V(idx) => board.play(1, *idx, v, "Game Move".to_string()),
        }
    }
}
#[derive(Component)]
struct Cell(Idx);
#[derive(Component)]
struct Grid;

#[derive(Resource)]
struct BoardAssets {
    edge_colors: (Color, Color, Color),
}
impl Default for BoardAssets {
    fn default() -> Self {
        Self {
            edge_colors: (Color::NONE, Color::GREEN, Color::GRAY),
        }
    }
}
impl BoardAssets {
    fn edge_color(&self, edge: &Fence) -> Color {
        match edge.0 {
            Some(true) => self.edge_colors.1,
            Some(false) => self.edge_colors.2,
            None => self.edge_colors.0,
        }
    }
}

fn board_setup(
    mut commands: Commands,
    board: Res<Board>,
    assets: Res<BoardAssets>,
    windows: Query<&Window>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let window = windows.single();
    let window_size = (window.resolution.height(), window.resolution.width());

    let board_size = board.size();
    let scale =
        (window_size.0 / board_size.0 as f32).min(window_size.1 / board_size.1 as f32) * 0.8;

    let get_node = |x: usize, y: usize, z: f32| -> Vec3 {
        Vec3 {
            x: scale * (x as f32 - board_size.1 as f32 / 2.),
            y: scale * (board_size.0 as f32 / 2. - y as f32),
            z,
        }
    };
    commands
        .spawn((
            Grid,
            PickableBundle::default(),
            MaterialMesh2dBundle {
                mesh: meshes
                    .add(Rectangle::new(
                        board_size.1 as f32 * scale * 1.2,
                        board_size.0 as f32 * scale * 1.2,
                    ))
                    .into(),
                transform: Transform::default(),
                material: materials.add(ColorMaterial::from(Color::BLACK)),
                ..default()
            },
        ))
        .with_children(|p| {
            for x in 0..=board_size.1 {
                for y in 0..=board_size.0 {
                    p.spawn((
                        Node((y, x)),
                        Name::new(format!("Node({x},{y})")),
                        MaterialMesh2dBundle {
                            mesh: meshes.add(Circle::new(10.)).into(),
                            material: materials.add(ColorMaterial::from(Color::WHITE)),
                            transform: Transform::default().with_translation(get_node(x, y, 1.)),
                            ..default()
                        },
                    ));
                }
            }
            for (idx, t) in board.0.tasks_iter() {
                p.spawn((
                    Cell(idx),
                    Name::new(format!("Cell({idx:?})")),
                    PickableBundle::default(),
                    MaterialMesh2dBundle {
                        mesh: meshes.add(Rectangle::new(scale - 10., scale - 10.)).into(),
                        material: materials.add(ColorMaterial::from(Color::NONE)),
                        transform: Transform::default().with_translation(
                            get_node(idx.1, idx.0, 1.)
                                + Vec3 {
                                    x: scale / 2.,
                                    y: -scale / 2.,
                                    z: 0.,
                                },
                        ),
                        ..default()
                    },
                ))
                .with_children(|p| {
                    if let Some(v) = t {
                        p.spawn(Text2dBundle {
                            text: Text::from_section(
                                format!("{v}"),
                                TextStyle {
                                    font_size: scale,
                                    ..default()
                                },
                            ),
                            transform: Transform::from_translation(Vec3::Z * 2.),
                            text_anchor: Anchor::Center,
                            ..Default::default()
                        });
                    }
                });
            }
            for ((dir, x, y), v) in board.fences_iter() {
                let (mut w, mut h) = (scale, 10.);
                let (mut dw, mut dh) = (scale / 2., 0.);
                if dir == 1 {
                    std::mem::swap(&mut dw, &mut dh);
                    std::mem::swap(&mut w, &mut h);
                }
                p.spawn((
                    Name::new(format!("E({dir}, {x}, {y})")),
                    if dir == 0 {
                        Edge::H((x, y))
                    } else {
                        Edge::V((x, y))
                    },
                    PickableBundle::default(),
                    //On::<Pointer<Click>>
                    MaterialMesh2dBundle {
                        mesh: meshes.add(Rectangle::new(w, h)).into(),
                        material: materials.add(ColorMaterial::from(assets.edge_color(v))),
                        transform: Transform::default().with_translation(
                            get_node(y, x, 1.)
                                + Vec3 {
                                    x: dw,
                                    y: -dh,
                                    z: 0.,
                                },
                        ),
                        ..default()
                    },
                ));
            }
        });
}

#[derive(Component)]
struct EdgeSelected;

/*
fn edge_interaction_system(
    mut commands: Commands,
    mut mouse_events: EventReader<CursorMoved>,
    q_camera: Query<(&Camera, &GlobalTransform)>,
    mut edges: Query<(Entity, &mut Stroke, &GlobalTransform, &Edge, &Path)>,
    hovered_edges: Query<Entity, With<EdgeSelected>>,
) {
    let (camera, camera_transform) = q_camera.single();

    for event in mouse_events.read() {
        hovered_edges.iter().for_each(|e| {
            commands.entity(e).remove::<EdgeSelected>();
        });
        if let Some(pos) = camera.viewport_to_world_2d(camera_transform, event.position) {
            // println!("{pos}");
            let mut found = false;
            edges.iter_mut().for_each(|(entity, mut s, t, e, _p)| {
                let diff = pos - t.translation().xy();
                let scale = t.to_scale_rotation_translation().0;
                s.color = if !found
                    && match e {
                        Edge::H(_) => {
                            diff.x > 0. && diff.x < 80. * scale.x && diff.y.abs() < 10. * scale.y
                        }
                        Edge::V(_) => {
                            diff.y < 0. && diff.y > -80. * scale.y && diff.x.abs() < 10. * scale.x
                        }
                    } {
                    // println!("{:?} -> {:?} -> {:?}", e, diff, p.0);
                    found = true;
                    commands.entity(entity).insert(EdgeSelected);
                    Color::BLUE
                } else {
                    Color::NONE
                }
            })
        }
    }
}

fn edge_click_system(
    mut board: ResMut<Board>,
    hovered_edge: Query<&Edge, With<EdgeSelected>>,
    mouse_button: Res<ButtonInput<MouseButton>>,
) {
    let v = if mouse_button.just_pressed(MouseButton::Left) {
        true
    } else if mouse_button.just_pressed(MouseButton::Right) {
        false
    } else {
        return;
    };
    if let Ok(edge) = hovered_edge.get_single() {
        edge.play_in(&mut board, v);
        println!("{}", board.0);
    }
}
*/

impl Board {
    fn is_updated(&self) -> bool {
        self.1 != self.0.moves().len()
    }
}
fn board_update_system(
    mut board: ResMut<Board>,
    mut edges: Query<(&mut Handle<ColorMaterial>, &Edge)>,
    assets: Res<BoardAssets>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    if !board.is_changed() || !board.is_updated() {
        return;
    }
    for mv in (board.1)..board.0.moves().len() {
        let fences::board::Move {
            direction,
            idx,
            value,
            ..
        } = board.moves()[mv];
        let mut fill = edges
            .iter_mut()
            .find_map(|(f, e)| {
                if match e {
                    Edge::H(g_idx) => direction == 0 && idx == *g_idx,
                    Edge::V(g_idx) => direction == 1 && idx == *g_idx,
                } {
                    Some(f)
                } else {
                    None
                }
            })
            .unwrap();
        *fill = materials.add(ColorMaterial::from(if value {
            assets.edge_colors.1
        } else {
            assets.edge_colors.2
        }));
    }
    board.1 = board.0.moves().len();
}
