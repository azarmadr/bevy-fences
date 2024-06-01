use bevy::{prelude::*, sprite::Anchor};
use bevy_prototype_lyon::{prelude::*, shapes};
use fences::{board::Fence, solver::Idx, BoardGeom, FencesSolver};

pub struct BoardPlugin;
impl Plugin for BoardPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BoardAssets>()
            .insert_resource(Board::new(
                "3#33    
0 0 0 y
1 0 2 y",
            ))
            .add_systems(
                Update,
                (
                    edge_interaction_system,
                    edge_click_system,
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
            ShapeBundle {
                path: GeometryBuilder::build_as(&shapes::Rectangle {
                    extents: Vec2 {
                        x: board_size.1 as f32 * (scale * 1.2),
                        y: board_size.0 as f32 * (scale * 1.2),
                    },
                    origin: RectangleOrigin::Center,
                }),
                ..default()
            },
            Fill::color(Color::BLACK),
        ))
        .with_children(|p| {
            for x in 0..=board_size.1 {
                for y in 0..=board_size.0 {
                    p.spawn((
                        Node((y, x)),
                        Name::new(format!("Node({x},{y})")),
                        ShapeBundle {
                            path: GeometryBuilder::build_as(&shapes::Circle {
                                radius: 10.,
                                ..default()
                            }),
                            spatial: SpatialBundle::from_transform(
                                Transform::default().with_translation(get_node(x, y, 1.)),
                            ),
                            ..Default::default()
                        },
                        Fill::color(Color::WHITE),
                    ));
                }
            }
            for (idx, t) in board.0.tasks_iter() {
                p.spawn((
                    Cell(idx),
                    Name::new(format!("Cell({idx:?})")),
                    ShapeBundle {
                        path: GeometryBuilder::build_as(&shapes::Rectangle {
                            extents: Vec2::splat(scale - 10.),
                            origin: RectangleOrigin::Center,
                        }),
                        spatial: SpatialBundle::from_transform(
                            Transform::default().with_translation(
                                get_node(idx.1, idx.0, 1.)
                                    + Vec3 {
                                        x: scale / 2.,
                                        y: -scale / 2.,
                                        z: 0.,
                                    },
                            ),
                        ),
                        ..default()
                    },
                    Fill::color(Color::NONE),
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
                let mut origin = Vec2 {
                    y: 0.,
                    x: scale / 2.,
                };
                let mut extents = Vec2 { x: scale, y: 10. };
                if dir == 1 {
                    extents = extents.yx();
                    origin = origin.yx();
                    origin.y = -origin.y;
                }
                let rect = shapes::Rectangle {
                    origin: RectangleOrigin::CustomCenter(origin),
                    extents,
                };
                p.spawn((
                    Name::new(format!("E({dir}, {x}, {y})")),
                    if dir == 0 {
                        Edge::H((x, y))
                    } else {
                        Edge::V((x, y))
                    },
                    ShapeBundle {
                        path: GeometryBuilder::build_as(&rect),
                        spatial: SpatialBundle::from_transform(
                            Transform::default().with_translation(get_node(y, x, 1.)),
                        ),
                        ..default()
                    },
                    Fill::color(assets.edge_color(v)),
                    Stroke::new(Color::NONE, 3.),
                ));
            }
        });
}

#[derive(Component)]
struct EdgeSelected;

fn edge_interaction_system(
    _board: Res<Board>,
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

impl Board {
    fn is_updated(&self) -> bool {
        self.1 != self.0.moves().len()
    }
}
fn board_update_system(
    board: Res<Board>,
    mut edges: Query<(&mut Fill, &Edge)>,
    assets: Res<BoardAssets>,
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
        fill.color = if value {
            assets.edge_colors.1
        } else {
            assets.edge_colors.2
        }
    }
}
