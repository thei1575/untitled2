//! Chunk data structures and management.

use voxel_core::{
    BlockId, Palette, IVec3, AIR_BLOCK,
    CHUNK_SIZE, CHUNK_HEIGHT, CHUNK_VOLUME,
    local_to_index, index_to_local, world_to_chunk, world_to_local,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A chunk of voxel data with palette compression.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chunk {
    /// Chunk position in chunk coordinates
    pub position: IVec3,
    /// Palette for block ID compression
    pub palette: Palette,
    /// Voxel data as palette indices
    pub voxels: Vec<u8>,
    /// Whether this chunk has been modified since last save
    pub dirty: bool,
}

impl Chunk {
    /// Create a new empty chunk filled with air.
    pub fn new(position: IVec3) -> Self {
        Self {
            position,
            palette: Palette::new(),
            voxels: vec![0; CHUNK_VOLUME], // All air (palette index 0)
            dirty: false,
        }
    }
    
    /// Get block ID at local coordinates.
    pub fn get_block(&self, local_pos: IVec3) -> BlockId {
        if let Some(index) = local_to_index(local_pos) {
            let palette_id = self.voxels[index];
            self.palette.get_block(palette_id)
        } else {
            AIR_BLOCK
        }
    }
    
    /// Set block ID at local coordinates.
    pub fn set_block(&mut self, local_pos: IVec3, block_id: BlockId) {
        if let Some(index) = local_to_index(local_pos) {
            let palette_id = self.palette.add_block(block_id);
            self.voxels[index] = palette_id;
            self.dirty = true;
        }
    }
    
    /// Fill the entire chunk with a single block type.
    pub fn fill(&mut self, block_id: BlockId) {
        self.palette = Palette::new();
        let palette_id = self.palette.add_block(block_id);
        self.voxels.fill(palette_id);
        self.dirty = true;
    }
    
    /// Check if chunk is entirely air.
    pub fn is_empty(&self) -> bool {
        self.palette.len() == 1 && self.palette.get_block(0) == AIR_BLOCK
    }
    
    /// Get the number of non-air blocks in this chunk.
    pub fn count_solid_blocks(&self) -> usize {
        if self.is_empty() {
            return 0;
        }
        
        self.voxels.iter()
            .filter(|&&palette_id| {
                let block_id = self.palette.get_block(palette_id);
                block_id != AIR_BLOCK
            })
            .count()
    }
    
    /// Iterate over all non-air blocks with their positions and block IDs.
    pub fn iter_blocks(&self) -> impl Iterator<Item = (IVec3, BlockId)> + '_ {
        self.voxels.iter().enumerate()
            .filter_map(|(index, &palette_id)| {
                let block_id = self.palette.get_block(palette_id);
                if block_id != AIR_BLOCK {
                    index_to_local(index).map(|pos| (pos, block_id))
                } else {
                    None
                }
            })
    }
    
    /// Mark chunk as clean (saved).
    pub fn mark_clean(&mut self) {
        self.dirty = false;
    }
    
    /// Check if chunk needs saving.
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }
}

/// Manages a collection of chunks.
#[derive(Debug, Default)]
pub struct ChunkManager {
    chunks: HashMap<IVec3, Chunk>,
}

impl ChunkManager {
    pub fn new() -> Self {
        Self {
            chunks: HashMap::new(),
        }
    }
    
    /// Get a chunk at the given chunk coordinates, creating if necessary.
    pub fn get_or_create_chunk(&mut self, chunk_pos: IVec3) -> &mut Chunk {
        self.chunks.entry(chunk_pos)
            .or_insert_with(|| Chunk::new(chunk_pos))
    }
    
    /// Get a chunk at the given chunk coordinates.
    pub fn get_chunk(&self, chunk_pos: IVec3) -> Option<&Chunk> {
        self.chunks.get(&chunk_pos)
    }
    
    /// Get a mutable chunk at the given chunk coordinates.
    pub fn get_chunk_mut(&mut self, chunk_pos: IVec3) -> Option<&mut Chunk> {
        self.chunks.get_mut(&chunk_pos)
    }
    
    /// Insert a chunk.
    pub fn insert_chunk(&mut self, chunk: Chunk) {
        self.chunks.insert(chunk.position, chunk);
    }
    
    /// Remove a chunk.
    pub fn remove_chunk(&mut self, chunk_pos: IVec3) -> Option<Chunk> {
        self.chunks.remove(&chunk_pos)
    }
    
    /// Get block at world coordinates.
    pub fn get_block(&self, world_pos: IVec3) -> BlockId {
        let chunk_pos = world_to_chunk(world_pos);
        let local_pos = world_to_local(world_pos);
        
        self.get_chunk(chunk_pos)
            .map(|chunk| chunk.get_block(local_pos))
            .unwrap_or(AIR_BLOCK)
    }
    
    /// Set block at world coordinates.
    pub fn set_block(&mut self, world_pos: IVec3, block_id: BlockId) {
        let chunk_pos = world_to_chunk(world_pos);
        let local_pos = world_to_local(world_pos);
        
        let chunk = self.get_or_create_chunk(chunk_pos);
        chunk.set_block(local_pos, block_id);
    }
    
    /// Get all loaded chunk positions.
    pub fn loaded_chunks(&self) -> impl Iterator<Item = IVec3> + '_ {
        self.chunks.keys().copied()
    }
    
    /// Get all dirty chunks that need saving.
    pub fn dirty_chunks(&self) -> impl Iterator<Item = &Chunk> + '_ {
        self.chunks.values().filter(|chunk| chunk.is_dirty())
    }
    
    /// Get number of loaded chunks.
    pub fn chunk_count(&self) -> usize {
        self.chunks.len()
    }
    
    /// Unload chunks outside a given radius from center.
    pub fn unload_distant_chunks(&mut self, center: IVec3, max_distance: i32) {
        let max_distance_sq = max_distance * max_distance;
        
        self.chunks.retain(|&chunk_pos, _| {
            let diff = chunk_pos - center;
            let distance_sq = diff.x * diff.x + diff.z * diff.z;
            distance_sq <= max_distance_sq
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_creation() {
        let pos = IVec3::new(1, 0, 2);
        let chunk = Chunk::new(pos);

        assert_eq!(chunk.position, pos);
        assert!(chunk.is_empty());
        assert_eq!(chunk.count_solid_blocks(), 0);
        assert!(!chunk.is_dirty());

        // All blocks should be air
        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_HEIGHT {
                for z in 0..CHUNK_SIZE {
                    let local_pos = IVec3::new(x, y, z);
                    assert_eq!(chunk.get_block(local_pos), AIR_BLOCK);
                }
            }
        }
    }

    #[test]
    fn test_chunk_block_operations() {
        let mut chunk = Chunk::new(IVec3::ZERO);
        let pos = IVec3::new(5, 10, 7);
        let stone_id = 1;

        // Set a block
        chunk.set_block(pos, stone_id);
        assert_eq!(chunk.get_block(pos), stone_id);
        assert!(chunk.is_dirty());
        assert_eq!(chunk.count_solid_blocks(), 1);
        assert!(!chunk.is_empty());

        // Set back to air
        chunk.set_block(pos, AIR_BLOCK);
        assert_eq!(chunk.get_block(pos), AIR_BLOCK);
        assert_eq!(chunk.count_solid_blocks(), 0);

        // Test out of bounds
        let oob_pos = IVec3::new(-1, 0, 0);
        chunk.set_block(oob_pos, stone_id); // Should be ignored
        assert_eq!(chunk.get_block(oob_pos), AIR_BLOCK);
    }

    #[test]
    fn test_chunk_fill() {
        let mut chunk = Chunk::new(IVec3::ZERO);
        let stone_id = 1;

        chunk.fill(stone_id);
        assert!(chunk.is_dirty());
        assert!(!chunk.is_empty());
        assert_eq!(chunk.count_solid_blocks(), CHUNK_VOLUME);

        // Check a few random positions
        assert_eq!(chunk.get_block(IVec3::new(0, 0, 0)), stone_id);
        assert_eq!(chunk.get_block(IVec3::new(8, 64, 12)), stone_id);
        assert_eq!(chunk.get_block(IVec3::new(15, 255, 15)), stone_id);
    }

    #[test]
    fn test_chunk_manager() {
        let mut manager = ChunkManager::new();
        let chunk_pos = IVec3::new(1, 0, 2);
        let world_pos = IVec3::new(20, 50, 35); // Should be in chunk (1, 0, 2)
        let stone_id = 1;

        // Initially no chunks
        assert_eq!(manager.chunk_count(), 0);
        assert_eq!(manager.get_block(world_pos), AIR_BLOCK);

        // Set a block (should create chunk)
        manager.set_block(world_pos, stone_id);
        assert_eq!(manager.chunk_count(), 1);
        assert_eq!(manager.get_block(world_pos), stone_id);

        // Verify chunk was created at correct position
        assert!(manager.get_chunk(chunk_pos).is_some());

        // Test unloading distant chunks
        let far_pos = IVec3::new(10, 0, 10);
        manager.get_or_create_chunk(far_pos);
        assert_eq!(manager.chunk_count(), 2);

        manager.unload_distant_chunks(IVec3::ZERO, 5);
        assert_eq!(manager.chunk_count(), 1); // Far chunk should be unloaded
        assert!(manager.get_chunk(chunk_pos).is_some()); // Near chunk should remain
        assert!(manager.get_chunk(far_pos).is_none()); // Far chunk should be gone
    }
}
