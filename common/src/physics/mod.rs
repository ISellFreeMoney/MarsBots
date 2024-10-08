use crate::world::BlockPos;

pub(crate) mod simulation;
mod aabb;
mod camera;
mod player;

pub trait BlockContainer {
    fn is_block_full(&self, pos: BlockPos) -> bool;
}