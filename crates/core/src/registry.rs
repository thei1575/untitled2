//! Block registry and palette system for efficient voxel storage.

use crate::{BlockId, BlockKind, AIR_BLOCK};
use std::collections::HashMap;

/// Block definition with properties.
#[derive(Debug, Clone, PartialEq)]
pub struct BlockDef {
    pub id: BlockId,
    pub name: String,
    pub kind: BlockKind,
    pub texture_id: u16,
}

impl BlockDef {
    pub fn new(id: BlockId, name: impl Into<String>, kind: BlockKind, texture_id: u16) -> Self {
        Self {
            id,
            name: name.into(),
            kind,
            texture_id,
        }
    }
    
    pub fn air() -> Self {
        Self::new(AIR_BLOCK, "air", BlockKind::Air, 0)
    }
    
    pub fn stone() -> Self {
        Self::new(1, "stone", BlockKind::Solid, 1)
    }
    
    pub fn dirt() -> Self {
        Self::new(2, "dirt", BlockKind::Solid, 2)
    }
    
    pub fn grass() -> Self {
        Self::new(3, "grass", BlockKind::Solid, 3)
    }
    
    pub fn wood() -> Self {
        Self::new(4, "wood", BlockKind::Solid, 4)
    }
}

/// Global block registry.
#[derive(Debug, Clone)]
pub struct BlockRegistry {
    blocks: HashMap<BlockId, BlockDef>,
    name_to_id: HashMap<String, BlockId>,
}

impl BlockRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            blocks: HashMap::new(),
            name_to_id: HashMap::new(),
        };
        
        // Register default blocks
        registry.register(BlockDef::air());
        registry.register(BlockDef::stone());
        registry.register(BlockDef::dirt());
        registry.register(BlockDef::grass());
        registry.register(BlockDef::wood());
        
        registry
    }
    
    pub fn register(&mut self, block: BlockDef) {
        self.name_to_id.insert(block.name.clone(), block.id);
        self.blocks.insert(block.id, block);
    }
    
    pub fn get(&self, id: BlockId) -> Option<&BlockDef> {
        self.blocks.get(&id)
    }
    
    pub fn get_by_name(&self, name: &str) -> Option<&BlockDef> {
        self.name_to_id.get(name)
            .and_then(|&id| self.blocks.get(&id))
    }
    
    pub fn get_kind(&self, id: BlockId) -> BlockKind {
        self.blocks.get(&id)
            .map(|block| block.kind)
            .unwrap_or(BlockKind::Air)
    }
    
    pub fn is_solid(&self, id: BlockId) -> bool {
        self.get_kind(id).is_solid()
    }
    
    pub fn is_air(&self, id: BlockId) -> bool {
        id == AIR_BLOCK || self.get_kind(id).is_air()
    }
    
    pub fn iter(&self) -> impl Iterator<Item = &BlockDef> {
        self.blocks.values()
    }
}

impl Default for BlockRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Palette for efficient chunk storage - maps local indices to block IDs.
#[derive(Debug, Clone)]
pub struct Palette {
    /// Maps palette index to block ID
    id_to_block: Vec<BlockId>,
    /// Maps block ID to palette index
    block_to_id: HashMap<BlockId, u8>,
}

impl Palette {
    pub fn new() -> Self {
        let mut palette = Self {
            id_to_block: Vec::new(),
            block_to_id: HashMap::new(),
        };
        
        // Air is always palette index 0
        palette.add_block(AIR_BLOCK);
        palette
    }
    
    /// Add a block to the palette, returning its palette index.
    pub fn add_block(&mut self, block_id: BlockId) -> u8 {
        if let Some(&palette_id) = self.block_to_id.get(&block_id) {
            return palette_id;
        }
        
        let palette_id = self.id_to_block.len() as u8;
        if palette_id == 255 {
            panic!("Palette overflow: too many unique blocks in chunk");
        }
        
        self.id_to_block.push(block_id);
        self.block_to_id.insert(block_id, palette_id);
        palette_id
    }
    
    /// Get block ID from palette index.
    pub fn get_block(&self, palette_id: u8) -> BlockId {
        self.id_to_block.get(palette_id as usize)
            .copied()
            .unwrap_or(AIR_BLOCK)
    }
    
    /// Get palette index for block ID.
    pub fn get_palette_id(&self, block_id: BlockId) -> Option<u8> {
        self.block_to_id.get(&block_id).copied()
    }
    
    /// Number of unique blocks in palette.
    pub fn len(&self) -> usize {
        self.id_to_block.len()
    }
    
    pub fn is_empty(&self) -> bool {
        self.id_to_block.is_empty()
    }
    
    /// Iterate over (palette_id, block_id) pairs.
    pub fn iter(&self) -> impl Iterator<Item = (u8, BlockId)> + '_ {
        self.id_to_block.iter().enumerate()
            .map(|(i, &block_id)| (i as u8, block_id))
    }
}

impl Default for Palette {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_registry() {
        let registry = BlockRegistry::new();

        // Test default blocks
        assert!(registry.get(AIR_BLOCK).is_some());
        assert_eq!(registry.get(AIR_BLOCK).unwrap().name, "air");
        assert_eq!(registry.get_kind(AIR_BLOCK), BlockKind::Air);
        assert!(registry.is_air(AIR_BLOCK));
        assert!(!registry.is_solid(AIR_BLOCK));

        let stone = registry.get_by_name("stone").unwrap();
        assert_eq!(stone.id, 1);
        assert_eq!(stone.kind, BlockKind::Solid);
        assert!(registry.is_solid(stone.id));
        assert!(!registry.is_air(stone.id));

        // Test unknown block
        assert!(registry.get(999).is_none());
        assert_eq!(registry.get_kind(999), BlockKind::Air); // Default to air
    }

    #[test]
    fn test_palette() {
        let mut palette = Palette::new();

        // Air should be index 0
        assert_eq!(palette.get_block(0), AIR_BLOCK);
        assert_eq!(palette.get_palette_id(AIR_BLOCK), Some(0));
        assert_eq!(palette.len(), 1);

        // Add stone
        let stone_id = 1;
        let stone_palette_id = palette.add_block(stone_id);
        assert_eq!(stone_palette_id, 1);
        assert_eq!(palette.get_block(1), stone_id);
        assert_eq!(palette.get_palette_id(stone_id), Some(1));
        assert_eq!(palette.len(), 2);

        // Adding same block should return same palette ID
        let stone_palette_id2 = palette.add_block(stone_id);
        assert_eq!(stone_palette_id, stone_palette_id2);
        assert_eq!(palette.len(), 2);

        // Test iteration
        let blocks: Vec<_> = palette.iter().collect();
        assert_eq!(blocks, vec![(0, AIR_BLOCK), (1, stone_id)]);
    }
}
