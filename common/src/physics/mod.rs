use crate::world::BlockPos;

pub mod simulation;
pub mod aabb;
mod camera;
pub mod player;

pub trait BlockContainer {
    fn is_block_full(&self, pos: BlockPos) -> bool;
}