//! Terrain generation using layered noise.

use voxel_core::{BlockId, BlockRegistry, IVec3, CHUNK_SIZE, CHUNK_HEIGHT};
use crate::chunk::{Chunk, ChunkManager};
use noise::{NoiseFn, Perlin, Seedable};

/// Terrain generator configuration.
#[derive(Debug, Clone)]
pub struct TerrainConfig {
    pub seed: u32,
    pub sea_level: i32,
    pub max_height: i32,
    pub height_scale: f64,
    pub height_frequency: f64,
    pub cave_frequency: f64,
    pub cave_threshold: f64,
}

impl Default for TerrainConfig {
    fn default() -> Self {
        Self {
            seed: 12345,
            sea_level: 64,
            max_height: 128,
            height_scale: 32.0,
            height_frequency: 0.01,
            cave_frequency: 0.05,
            cave_threshold: 0.3,
        }
    }
}

/// Procedural terrain generator.
pub struct TerrainGenerator {
    config: TerrainConfig,
    height_noise: Perlin,
    cave_noise: Perlin,
    registry: BlockRegistry,
}

impl TerrainGenerator {
    pub fn new(config: TerrainConfig) -> Self {
        let mut height_noise = Perlin::new();
        height_noise = height_noise.set_seed(config.seed);
        
        let mut cave_noise = Perlin::new();
        cave_noise = cave_noise.set_seed(config.seed.wrapping_add(1));
        
        Self {
            config,
            height_noise,
            cave_noise,
            registry: BlockRegistry::new(),
        }
    }
    
    /// Generate terrain height at given x, z coordinates.
    pub fn get_height(&self, x: i32, z: i32) -> i32 {
        let height = self.height_noise.get([
            x as f64 * self.config.height_frequency,
            z as f64 * self.config.height_frequency,
        ]);
        
        let scaled_height = height * self.config.height_scale;
        let final_height = self.config.sea_level + scaled_height as i32;
        
        final_height.clamp(0, self.config.max_height)
    }
    
    /// Check if position should be a cave.
    pub fn is_cave(&self, x: i32, y: i32, z: i32) -> bool {
        if y <= 5 || y >= self.config.max_height - 5 {
            return false; // No caves near bedrock or surface
        }
        
        let cave_value = self.cave_noise.get([
            x as f64 * self.config.cave_frequency,
            y as f64 * self.config.cave_frequency * 0.5, // Stretch caves vertically
            z as f64 * self.config.cave_frequency,
        ]);
        
        cave_value.abs() < self.config.cave_threshold
    }
    
    /// Get block type for given world position.
    pub fn get_block_at(&self, world_pos: IVec3) -> BlockId {
        let height = self.get_height(world_pos.x, world_pos.z);
        
        // Above terrain
        if world_pos.y > height {
            return self.registry.get_by_name("air").unwrap().id;
        }
        
        // Check for caves
        if self.is_cave(world_pos.x, world_pos.y, world_pos.z) {
            return self.registry.get_by_name("air").unwrap().id;
        }
        
        // Bedrock layer
        if world_pos.y <= 0 {
            return self.registry.get_by_name("stone").unwrap().id;
        }
        
        // Surface layer
        if world_pos.y == height {
            return self.registry.get_by_name("grass").unwrap().id;
        }
        
        // Subsurface layers
        if world_pos.y >= height - 3 {
            return self.registry.get_by_name("dirt").unwrap().id;
        }
        
        // Deep stone
        self.registry.get_by_name("stone").unwrap().id
    }
    
    /// Generate a single chunk.
    pub fn generate_chunk(&self, chunk_pos: IVec3) -> Chunk {
        let mut chunk = Chunk::new(chunk_pos);
        
        // Calculate world coordinates for this chunk
        let world_x_start = chunk_pos.x * CHUNK_SIZE;
        let world_z_start = chunk_pos.z * CHUNK_SIZE;
        
        // Generate blocks for the entire chunk
        for local_x in 0..CHUNK_SIZE {
            for local_z in 0..CHUNK_SIZE {
                let world_x = world_x_start + local_x;
                let world_z = world_z_start + local_z;
                
                for local_y in 0..CHUNK_HEIGHT {
                    let world_pos = IVec3::new(world_x, local_y, world_z);
                    let local_pos = IVec3::new(local_x, local_y, local_z);
                    
                    let block_id = self.get_block_at(world_pos);
                    chunk.set_block(local_pos, block_id);
                }
            }
        }
        
        chunk.mark_clean(); // Newly generated chunks are clean
        chunk
    }
    
    /// Generate multiple chunks in a radius around center.
    pub fn generate_chunks_around(&self, center: IVec3, radius: i32, chunk_manager: &mut ChunkManager) {
        for x in (center.x - radius)..=(center.x + radius) {
            for z in (center.z - radius)..=(center.z + radius) {
                let chunk_pos = IVec3::new(x, 0, z);
                
                // Only generate if chunk doesn't exist
                if chunk_manager.get_chunk(chunk_pos).is_none() {
                    let chunk = self.generate_chunk(chunk_pos);
                    chunk_manager.insert_chunk(chunk);
                }
            }
        }
    }
    
    /// Get the terrain configuration.
    pub fn config(&self) -> &TerrainConfig {
        &self.config
    }
    
    /// Get the block registry.
    pub fn registry(&self) -> &BlockRegistry {
        &self.registry
    }
}

/// Biome-based terrain generation (future expansion).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Biome {
    Plains,
    Hills,
    Mountains,
    Desert,
}

impl Biome {
    pub fn get_surface_block(&self, registry: &BlockRegistry) -> BlockId {
        match self {
            Biome::Plains | Biome::Hills => registry.get_by_name("grass").unwrap().id,
            Biome::Mountains => registry.get_by_name("stone").unwrap().id,
            Biome::Desert => registry.get_by_name("dirt").unwrap().id, // Would be sand in full implementation
        }
    }
    
    pub fn get_height_scale(&self) -> f64 {
        match self {
            Biome::Plains => 16.0,
            Biome::Hills => 32.0,
            Biome::Mountains => 64.0,
            Biome::Desert => 8.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terrain_config() {
        let config = TerrainConfig::default();
        assert_eq!(config.seed, 12345);
        assert_eq!(config.sea_level, 64);
        assert!(config.height_scale > 0.0);
        assert!(config.height_frequency > 0.0);
    }

    #[test]
    fn test_terrain_generator() {
        let config = TerrainConfig::default();
        let generator = TerrainGenerator::new(config);

        // Test height generation
        let height1 = generator.get_height(0, 0);
        let height2 = generator.get_height(100, 100);

        // Heights should be within reasonable bounds
        assert!(height1 >= 0);
        assert!(height1 <= generator.config.max_height);
        assert!(height2 >= 0);
        assert!(height2 <= generator.config.max_height);

        // Same coordinates should give same height
        assert_eq!(height1, generator.get_height(0, 0));

        // Test block generation
        let surface_pos = IVec3::new(0, height1, 0);
        let surface_block = generator.get_block_at(surface_pos);
        assert_eq!(surface_block, generator.registry.get_by_name("grass").unwrap().id);

        let above_surface = IVec3::new(0, height1 + 10, 0);
        let air_block = generator.get_block_at(above_surface);
        assert_eq!(air_block, generator.registry.get_by_name("air").unwrap().id);

        let bedrock_pos = IVec3::new(0, 0, 0);
        let bedrock_block = generator.get_block_at(bedrock_pos);
        assert_eq!(bedrock_block, generator.registry.get_by_name("stone").unwrap().id);
    }

    #[test]
    fn test_chunk_generation() {
        let config = TerrainConfig::default();
        let generator = TerrainGenerator::new(config);
        let chunk_pos = IVec3::new(0, 0, 0);

        let chunk = generator.generate_chunk(chunk_pos);
        assert_eq!(chunk.position, chunk_pos);
        assert!(!chunk.is_dirty()); // Newly generated chunks are clean
        assert!(!chunk.is_empty()); // Should have terrain

        // Should have some solid blocks (terrain)
        assert!(chunk.count_solid_blocks() > 0);

        // Test that blocks are properly set
        let mut found_surface = false;
        let mut found_air = false;

        for (local_pos, block_id) in chunk.iter_blocks() {
            if block_id == generator.registry.get_by_name("grass").unwrap().id {
                found_surface = true;
            }
        }

        // Check for air blocks above terrain
        for y in 200..CHUNK_HEIGHT {
            let pos = IVec3::new(8, y, 8);
            if chunk.get_block(pos) == generator.registry.get_by_name("air").unwrap().id {
                found_air = true;
                break;
            }
        }

        assert!(found_surface, "Should have surface blocks");
        assert!(found_air, "Should have air blocks above terrain");
    }

    #[test]
    fn test_biome_properties() {
        let registry = BlockRegistry::new();

        let plains = Biome::Plains;
        let mountains = Biome::Mountains;

        assert_eq!(plains.get_surface_block(&registry), registry.get_by_name("grass").unwrap().id);
        assert_eq!(mountains.get_surface_block(&registry), registry.get_by_name("stone").unwrap().id);

        assert!(mountains.get_height_scale() > plains.get_height_scale());
    }
}
