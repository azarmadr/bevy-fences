use bevy::prelude::*;

#[derive(Resource)]
pub struct Board(pub fences::Board);

#[derive(Component)]
struct Node;
#[derive(Component)]
struct Edge;
#[derive(Component)]
struct Cell;
#[derive(Component)]
struct Grid;

fn board_setup(mut commands: Commands, board: Res<Board>) {
    commands.spawn(Grid);
}

struct BoardPlugin;
impl Plugin for BoardPlugin {
    fn build()
}
