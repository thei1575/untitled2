//! Core shared types and utilities for the voxel sandbox.

pub mod math;

pub type BlockId = u16;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BlockKind {
    Air,
    Solid,
}

impl BlockKind {
    pub fn is_solid(self) -> bool {
        matches!(self, BlockKind::Solid)
    }
}

