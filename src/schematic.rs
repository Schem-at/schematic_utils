use std::collections::HashMap;
use quartz_nbt::{NbtCompound, NbtTag};
use serde::{Deserialize, Serialize};
use crate::{Region, GlobalPalette, BlockState, Entity, BlockEntity};
use crate::bounding_box::BoundingBox;
use crate::metadata::Metadata;

#[derive(Serialize, Deserialize)]
pub struct UniversalSchematic {
    pub metadata: Metadata,
    pub regions: HashMap<String, Region>,
    pub palette: GlobalPalette,
    default_region_name: String,
}

impl UniversalSchematic {
    pub fn new(name: String) -> Self {
        UniversalSchematic {
            metadata: Metadata {
                name: Some(name),
                ..Metadata::default()
            },
            regions: HashMap::new(),
            palette: GlobalPalette::new(),
            default_region_name: "Main".to_string(),
        }
    }

    pub fn get_json_string(&self) -> Result<String, String> {
        // Attempt to serialize the name
        let metadata_json = serde_json::to_string(&self.metadata)
            .map_err(|e| format!("Failed to serialize 'metadata' in UniversalSchematic: {}", e))?;

        // Attempt to serialize the regions
        let regions_json = serde_json::to_string(&self.regions)
            .map_err(|e| format!("Failed to serialize 'regions' in UniversalSchematic: {}", e))?;

        // Attempt to serialize the palette
        let palette_json = serde_json::to_string(&self.palette)
            .map_err(|e| format!("Failed to serialize 'palette' in UniversalSchematic: {}", e))?;

        // Combine everything into a single JSON object manually
        let combined_json = format!(
            "{{\"metadata\":{},\"regions\":{},\"palette\":{}}}",
            metadata_json, regions_json, palette_json
        );

        Ok(combined_json)
    }

    pub fn set_block(&mut self, x: i32, y: i32, z: i32, block: BlockState) -> bool {
        let region_name = self.default_region_name.clone();
        self.set_block_in_region(&region_name, x, y, z, block)
    }

    pub fn set_block_in_region(&mut self, region_name: &str, x: i32, y: i32, z: i32, block: BlockState) -> bool {
        let region = self.regions.entry(region_name.to_string()).or_insert_with(|| {
            Region::new(region_name.to_string(), (x, y, z), (1, 1, 1))
        });

        // Expand the region if necessary
        region.expand_to_fit(x, y, z);

        let block_index = self.palette.get_or_insert(block);
        region.set_block_index(x, y, z, block_index)
    }

    pub fn get_block(&self, x: i32, y: i32, z: i32) -> Option<&BlockState> {
        for region in self.regions.values() {
            if region.get_bounding_box().contains((x, y, z)) {
                if let Some(block) = self.get_block_from_region(&region.name, x, y, z) {
                    return Some(block);
                }
            }
        }
        None
    }

    pub fn get_region_bounding_box(&self, region_name: &str) -> Option<BoundingBox> {
        self.regions.get(region_name).map(|region| region.get_bounding_box())
    }

    pub fn get_schematic_bounding_box(&self) -> Option<BoundingBox> {
        if self.regions.is_empty() {
            return None;
        }

        let mut bounding_box = self.regions.values().next().unwrap().get_bounding_box();
        for region in self.regions.values().skip(1) {
            bounding_box = bounding_box.union(&region.get_bounding_box());
        }
        Some(bounding_box)
    }

    pub fn get_block_from_region(&self, region_name: &str, x: i32, y: i32, z: i32) -> Option<&BlockState> {
        self.regions.get(region_name)
            .and_then(|region| region.get_block_index(x, y, z))
            .and_then(|block_index| self.palette.get(block_index))
    }

    pub fn add_region(&mut self, region: Region) -> bool {
        if self.regions.contains_key(&region.name) {
            false
        } else {
            self.regions.insert(region.name.clone(), region);
            true
        }
    }

    pub fn remove_region(&mut self, name: &str) -> Option<Region> {
        self.regions.remove(name)
    }

    pub fn get_region(&self, name: &str) -> Option<&Region> {
        self.regions.get(name)
    }

    pub fn get_region_mut(&mut self, name: &str) -> Option<&mut Region> {
        self.regions.get_mut(name)
    }


    pub fn add_block_entity_in_region(&mut self, region_name: &str, block_entity: BlockEntity) -> bool {
        let region = self.regions.entry(region_name.to_string()).or_insert_with(|| {
            Region::new(region_name.to_string(), block_entity.position, (1, 1, 1))
        });

        region.add_block_entity(block_entity);
        true
    }

    pub fn remove_block_entity_in_region(&mut self, region_name: &str, position: (i32, i32, i32)) -> Option<BlockEntity> {
        self.regions.get_mut(region_name)?.remove_block_entity(position)
    }

    pub fn add_block_entity(&mut self, block_entity: BlockEntity) -> bool {
        let region_name = self.default_region_name.clone();
        self.add_block_entity_in_region(&region_name, block_entity)
    }

    pub fn remove_block_entity(&mut self, position: (i32, i32, i32)) -> Option<BlockEntity> {
        let region_name = self.default_region_name.clone();
        self.remove_block_entity_in_region(&region_name, position)
    }

    pub fn add_entity_in_region(&mut self, region_name: &str, entity: Entity) -> bool {
        let region = self.regions.entry(region_name.to_string()).or_insert_with(|| {
            let rounded_position = (entity.position.0.round() as i32, entity.position.1.round() as i32, entity.position.2.round() as i32);
            Region::new(region_name.to_string(), rounded_position, (1, 1, 1))
        });

        region.add_entity(entity);
        true
    }

    pub fn remove_entity_in_region(&mut self, region_name: &str, index: usize) -> Option<Entity> {
        self.regions.get_mut(region_name)?.remove_entity(index)
    }

    pub fn add_entity(&mut self, entity: Entity) -> bool {
        let region_name = self.default_region_name.clone();
        self.add_entity_in_region(&region_name, entity)
    }

    pub fn remove_entity(&mut self, index: usize) -> Option<Entity> {
        let region_name = self.default_region_name.clone();
        self.remove_entity_in_region(&region_name, index)
    }

    pub fn to_nbt(&self) -> NbtCompound {
        let mut root = NbtCompound::new();

        // Serialize name
        root.insert("Metadata", self.metadata.to_nbt());

        // Serialize regions
        let mut regions_tag = NbtCompound::new();
        for (name, region) in &self.regions {
            regions_tag.insert(name, region.to_nbt());
        }
        root.insert("Regions", NbtTag::Compound(regions_tag));

        // Serialize palette
        root.insert("Palette", self.palette.to_nbt());

        // Serialize default region name
        root.insert("DefaultRegion", NbtTag::String(self.default_region_name.clone()));

        root
    }

    pub fn from_nbt(nbt: NbtCompound) -> Result<Self, String> {
        let metadata = Metadata::from_nbt( nbt.get::<_, &NbtCompound>("Metadata")
                                               .map_err(|e| format!("Failed to get Metadata: {}", e)).unwrap()
        )?;

        let regions_tag = nbt.get::<_, &NbtCompound>("Regions")
            .map_err(|e| format!("Failed to get Regions: {}", e))?;
        let mut regions = HashMap::new();
        for (region_name, region_tag) in regions_tag.inner() {
            if let NbtTag::Compound(region_compound) = region_tag {
                regions.insert(region_name.to_string(), Region::from_nbt(&region_compound.clone())?);
            }
        }

        let palette = GlobalPalette::from_nbt(nbt.get::<_, &NbtCompound>("Palette")
            .map_err(|e| format!("Failed to get Palette: {}", e))?)?;

        let default_region_name = nbt.get::<_, &str>("DefaultRegion")
            .map_err(|e| format!("Failed to get DefaultRegion: {}", e))?
            .to_string();

        Ok(UniversalSchematic {
            metadata,
            regions,
            palette,
            default_region_name,
        })
    }
    pub fn get_bounding_box(&self) -> BoundingBox {
        let mut bounding_box = BoundingBox::new((i32::MAX, i32::MAX, i32::MAX), (i32::MIN, i32::MIN, i32::MIN));

        for region in self.regions.values() {
            let region_bb = region.get_bounding_box();
            bounding_box = bounding_box.union(&region_bb);
        }

        bounding_box
    }

    pub fn to_schematic(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        crate::formats::schematic::to_schematic(self)
    }

    pub fn from_schematic(data: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        crate::formats::schematic::from_schematic(data)
    }

}

#[cfg(test)]
mod tests {
    use std::io::Cursor;
    use quartz_nbt::io::{read_nbt, write_nbt};
    use super::*;

    #[test]
    fn test_schematic_operations() {
        let mut schematic = UniversalSchematic::new("Test Schematic".to_string());

        // Test automatic region creation and expansion
        let stone = BlockState::new("minecraft:stone".to_string());
        let dirt = BlockState::new("minecraft:dirt".to_string());

        assert!(schematic.set_block(0, 0, 0, stone.clone()));
        assert_eq!(schematic.get_block(0, 0, 0), Some(&stone));

        assert!(schematic.set_block(5, 5, 5, dirt.clone()));
        assert_eq!(schematic.get_block(5, 5, 5), Some(&dirt));

        // Check that the default region was created and expanded
        let default_region = schematic.get_region("Main").unwrap();
        assert_eq!(default_region.size, (6, 6, 6));

        // Test explicit region creation and manipulation
        let obsidian = BlockState::new("minecraft:obsidian".to_string());
        assert!(schematic.set_block_in_region("Custom", 10, 10, 10, obsidian.clone()));
        assert_eq!(schematic.get_block_from_region("Custom", 10, 10, 10), Some(&obsidian));

        // Check that the custom region was created
        let custom_region = schematic.get_region("Custom").unwrap();
        assert_eq!(custom_region.position, (10, 10, 10));
        assert_eq!(custom_region.size, (1, 1, 1));

        // Test manual region addition
        let region2 = Region::new("Region2".to_string(), (20, 0, 0), (5, 5, 5));
        assert!(schematic.add_region(region2));
        assert!(!schematic.add_region(Region::new("Region2".to_string(), (0, 0, 0), (1, 1, 1))));

        // Test getting non-existent blocks
        assert_eq!(schematic.get_block(100, 100, 100), None);
        assert_eq!(schematic.get_block_from_region("NonexistentRegion", 0, 0, 0), None);

        // Test removing regions
        assert!(schematic.remove_region("Region2").is_some());
        assert!(schematic.remove_region("Region2").is_none());

        // Test that removed region's blocks are no longer accessible
        assert_eq!(schematic.get_block_from_region("Region2", 20, 0, 0), None);
    }

    #[test]
    fn test_schematic_large_coordinates() {
        let mut schematic = UniversalSchematic::new("Large Schematic".to_string());

        let far_block = BlockState::new("minecraft:diamond_block".to_string());
        assert!(schematic.set_block(1000, 1000, 1000, far_block.clone()));
        assert_eq!(schematic.get_block(1000, 1000, 1000), Some(&far_block));

        let main_region = schematic.get_region("Main").unwrap();
        assert_eq!(main_region.position, (1000, 1000, 1000));
        assert_eq!(main_region.size, (1, 1, 1));

        // Test that blocks outside the region are not present
        assert_eq!(schematic.get_block(999, 1000, 1000), None);
        assert_eq!(schematic.get_block(1001, 1000, 1000), None);
    }

    #[test]
    fn test_schematic_region_expansion() {
        let mut schematic = UniversalSchematic::new("Expanding Schematic".to_string());

        let block1 = BlockState::new("minecraft:stone".to_string());
        let block2 = BlockState::new("minecraft:dirt".to_string());

        assert!(schematic.set_block(0, 0, 0, block1.clone()));
        assert!(schematic.set_block(10, 20, 30, block2.clone()));

        let main_region = schematic.get_region("Main").unwrap();
        assert_eq!(main_region.position, (0, 0, 0));
        assert_eq!(main_region.size, (11, 21, 31));

        assert_eq!(schematic.get_block(0, 0, 0), Some(&block1));
        assert_eq!(schematic.get_block(10, 20, 30), Some(&block2));
        assert_eq!(schematic.get_block(5, 10, 15), None);
    }

    #[test]
    fn test_schematic_negative_coordinates() {
        let mut schematic = UniversalSchematic::new("Negative Coordinates Schematic".to_string());

        let neg_block = BlockState::new("minecraft:emerald_block".to_string());
        assert!(schematic.set_block(-10, -10, -10, neg_block.clone()));
        assert_eq!(schematic.get_block(-10, -10, -10), Some(&neg_block));

        let main_region = schematic.get_region("Main").unwrap();
        assert!(main_region.position.0 <= -10 && main_region.position.1 <= -10 && main_region.position.2 <= -10);
    }



    #[test]
    fn test_entity_operations() {
        let mut schematic = UniversalSchematic::new("Test Schematic".to_string());

        let entity = Entity::new("minecraft:creeper".to_string(), (10.5, 65.0, 20.5))
            .with_nbt_data("Fuse".to_string(), "30".to_string());

        assert!(schematic.add_entity(entity.clone()));

        let region = schematic.get_region("Main").unwrap();
        assert_eq!(region.entities.len(), 1);
        assert_eq!(region.entities[0], entity);

        let removed_entity = schematic.remove_entity( 0).unwrap();
        assert_eq!(removed_entity, entity);

        let region = schematic.get_region("Main").unwrap();
        assert_eq!(region.entities.len(), 0);
    }

    #[test]
    fn test_block_entity_operations() {
        let mut schematic = UniversalSchematic::new("Test Schematic".to_string());

        let block_entity = BlockEntity::new("minecraft:chest".to_string(), (5, 10, 15))
            .with_nbt_data("Items".to_string(), "[{id:\"minecraft:diamond\",Count:64b,Slot:0b}]".to_string());

        assert!(schematic.add_block_entity( block_entity.clone()));

        let region = schematic.get_region("Main").unwrap();
        assert_eq!(region.block_entities.len(), 1);
        assert_eq!(region.block_entities.get(&(5, 10, 15)), Some(&block_entity));

        let removed_block_entity = schematic.remove_block_entity((5, 10, 15)).unwrap();
        assert_eq!(removed_block_entity, block_entity);

        let region = schematic.get_region("Main").unwrap();
        assert_eq!(region.block_entities.len(), 0);
    }

    #[test]
    fn test_block_entity_helper_operations() {
        let mut schematic = UniversalSchematic::new("Test Schematic".to_string());

        // Create a chest block entity with a diamond in slot 0
        let block_entity = BlockEntity::new("minecraft:chest".to_string(), (5, 10, 15))
            .with_item(0, "minecraft:diamond", 64)
            .with_custom_data("Lock", "SecretKey");

        assert!(schematic.add_block_entity(block_entity.clone()));

        let region = schematic.get_region("Main").unwrap();
        assert_eq!(region.block_entities.len(), 1);
        assert_eq!(region.block_entities.get(&(5, 10, 15)), Some(&block_entity));

        let removed_block_entity = schematic.remove_block_entity((5, 10, 15)).unwrap();
        assert_eq!(removed_block_entity, block_entity);

        let region = schematic.get_region("Main").unwrap();
        assert_eq!(region.block_entities.len(), 0);
    }

    #[test]
    fn test_block_entity_in_region_operations() {
        let mut schematic = UniversalSchematic::new("Test Schematic".to_string());

        let block_entity = BlockEntity::new("minecraft:chest".to_string(), (5, 10, 15))
            .with_nbt_data("Items".to_string(), "[{id:\"minecraft:diamond\",Count:64b,Slot:0b}]".to_string());

        assert!(schematic.add_block_entity_in_region("Main", block_entity.clone()));

        let region = schematic.get_region("Main").unwrap();
        assert_eq!(region.block_entities.len(), 1);
        assert_eq!(region.block_entities.get(&(5, 10, 15)), Some(&block_entity));

        let removed_block_entity = schematic.remove_block_entity_in_region("Main", (5, 10, 15)).unwrap();
        assert_eq!(removed_block_entity, block_entity);

        let region = schematic.get_region("Main").unwrap();
        assert_eq!(region.block_entities.len(), 0);
    }

    #[test]
    fn test_palette_operations() {
        let mut palette = GlobalPalette::new();

        let stone = BlockState::new("minecraft:stone".to_string());
        let dirt = BlockState::new("minecraft:dirt".to_string());

        assert_eq!(palette.get_or_insert(stone.clone()), 1);
        assert_eq!(palette.get_or_insert(dirt.clone()), 2);
        assert_eq!(palette.get_or_insert(stone.clone()), 1);

        assert_eq!(palette.get(0), Some(&BlockState::new("minecraft:air".to_string())));
        assert_eq!(palette.get(1), Some(&stone));
        assert_eq!(palette.get(2), Some(&dirt));
        assert_eq!(palette.get(3), None);

        assert_eq!(palette.len(), 3);
    }

    #[test]
    fn test_nbt_serialization_deserialization() {
        let mut schematic = UniversalSchematic::new("Test Schematic".to_string());

        // Add some blocks and entities
        schematic.set_block(0, 0, 0, BlockState::new("minecraft:stone".to_string()));
        schematic.set_block(1, 1, 1, BlockState::new("minecraft:dirt".to_string()));
        schematic.add_entity(Entity::new("minecraft:creeper".to_string(), (0.5, 0.0, 0.5)));

        // Serialize to NBT
        let nbt = schematic.to_nbt();


        let palette = nbt.get::<_, &NbtCompound>("Palette")
            .map_err(|e| format!("Failed to get Palette: {}", e)).unwrap();


        // Write NBT to a buffer
        let mut buffer = Vec::new();
        write_nbt(&mut buffer, None, &nbt, quartz_nbt::io::Flavor::Uncompressed).unwrap();

        // Read NBT from the buffer
        let (read_nbt, _) = read_nbt(&mut Cursor::new(buffer), quartz_nbt::io::Flavor::Uncompressed).unwrap();

        // Deserialize from NBT
        let deserialized_schematic = UniversalSchematic::from_nbt(read_nbt).unwrap();

        // Compare original and deserialized schematics
        assert_eq!(schematic.metadata, deserialized_schematic.metadata);
        assert_eq!(schematic.regions.len(), deserialized_schematic.regions.len());
        assert_eq!(schematic.palette.len(), deserialized_schematic.palette.len());

        // Check if blocks are correctly deserialized
        assert_eq!(schematic.get_block(0, 0, 0), deserialized_schematic.get_block(0, 0, 0));
        assert_eq!(schematic.get_block(1, 1, 1), deserialized_schematic.get_block(1, 1, 1));

        // Check if entities are correctly deserialized
        let original_entities = schematic.get_region("Main").unwrap().entities.clone();
        let deserialized_entities = deserialized_schematic.get_region("Main").unwrap().entities.clone();
        assert_eq!(original_entities, deserialized_entities);
    }

}
