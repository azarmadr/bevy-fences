use bevy::prelude::*;
use bevy_prototype_lyon::{prelude::*, shapes};
use fences::{board::Fence, solver::Idx, BoardGeom, FencesSolver};

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {}
}
