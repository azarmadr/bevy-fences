use std::cmp;

use bevy::{prelude::*, sprite::Anchor};
use bevy_prototype_lyon::{prelude::*, shapes};
use fences::{solver::Idx, BoardGeom, FencesSolver};

#[derive(Resource, Deref)]
pub struct Board(fences::Board);

#[derive(Component)]
struct Node(Idx);
#[derive(Component)]
enum Edge {
    H(Idx),
    V(Idx),
}
#[derive(Component)]
struct Cell(Idx);
#[derive(Component)]
struct Grid;

fn board_setup(mut commands: Commands, board: Res<Board>, windows: Query<&Window>) {
    let window = windows.single();
    let window_size = (window.resolution.height(), window.resolution.width());

    let board_size = board.size();
    let scale =
        (window_size.0 / board_size.0 as f32).min(window_size.1 / board_size.1 as f32) * 0.8;

    let get_node = |x: usize, y: usize, z: f32| -> Vec3 {
        Vec3 {
            x: scale * (x as f32 - board_size.1 as f32 / 2.),
            y: scale * (y as f32 - board_size.0 as f32 / 2.),
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
                                        y: scale / 2.,
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
            for ((dir, x, y), _) in board.fences_iter() {
                p.spawn((
                    Name::new(format!("E({dir}, {y}, {x})")),
                    if dir == 0 {
                        Edge::H((x, y))
                    } else {
                        Edge::V((x, y))
                    },
                    ShapeBundle {
                        path: GeometryBuilder::build_as(&shapes::Line(
                            Vec2::default(),
                            if dir == 0 {
                                Vec2 { y: 0., x: scale }
                            } else {
                                Vec2 { x: 0., y: scale }
                            },
                        )),
                        spatial: SpatialBundle::from_transform(
                            Transform::default().with_translation(get_node(y, x, 1.)),
                        ),
                        ..default()
                    },
                    Stroke::new(Color::NONE, 10.0),
                ));
            }
        });
}

pub struct BoardPlugin;
impl Plugin for BoardPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Board("3#33 ".parse().unwrap()))
            .add_systems(Startup, board_setup);
    }
}
