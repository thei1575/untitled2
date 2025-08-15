//! Core shared types and utilities for the voxel sandbox.

pub mod math;
pub mod registry;

pub use math::*;
pub use registry::*;

/// Unique identifier for block types.
pub type BlockId = u16;

/// Air block ID constant.
pub const AIR_BLOCK: BlockId = 0;

/// Basic classification of block behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BlockKind {
    Air,
    Solid,
}

impl BlockKind {
    pub fn is_solid(self) -> bool {
        matches!(self, BlockKind::Solid)
    }

    pub fn is_air(self) -> bool {
        matches!(self, BlockKind::Air)
    }
}

