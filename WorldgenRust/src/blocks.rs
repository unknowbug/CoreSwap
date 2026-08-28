// blocks.rs — 方块 ID 表 + 区块方块存储（16×16×height）
// C++ 参考：versions/1.20.1/cpp/worldgen/src/blocks.h（BlockId = vanilla block 注册表 raw id）
// 数据源：versions/1.20.1/data/blocks.json（{ "minecraft:stone": 1, ... }）

use std::collections::HashMap;

use crate::json;

pub type BlockId = i32;

pub const AIR: BlockId = 0;

// 方块 ID 注册表：从 blocks.json 加载 id↔name 双向表
#[derive(Default)]
pub struct BlockRegistry {
    name_to_id: HashMap<String, BlockId>,
    id_to_name: Vec<String>,
}

impl BlockRegistry {
    pub fn load_from_json(json_text: &str) -> Option<Self> {
        let root = json::parse(json_text).ok()?;
        let obj = root.as_object()?;
        let mut name_to_id = HashMap::new();
        for (k, v) in obj {
            if let Some(id) = v.as_f64() {
                name_to_id.insert(k.clone(), id as BlockId);
            }
        }
        let mut id_to_name = vec![String::new(); 16384];
        for (name, id) in &name_to_id {
            if *id >= 0 && (*id as usize) < id_to_name.len() {
                id_to_name[*id as usize] = name.clone();
            }
        }
        if name_to_id.is_empty() { return None; }
        Some(Self { name_to_id, id_to_name })
    }

    pub fn id(&self, name: &str) -> BlockId {
        self.name_to_id.get(name).copied().unwrap_or(AIR)
    }

    pub fn name(&self, id: BlockId) -> &str {
        if id >= 0 && (id as usize) < self.id_to_name.len() && !self.id_to_name[id as usize].is_empty() {
            &self.id_to_name[id as usize]
        } else {
            "?"
        }
    }

    pub fn contains(&self, name: &str) -> bool {
        self.name_to_id.contains_key(name)
    }
}

// 区块方块列：16×16×height，index = (y - minY) * 256 + z * 16 + x（维度参数化）
pub struct BlockColumn {
    min_y: i32,
    height: i32,
    blocks: Vec<BlockId>,
}

impl BlockColumn {
    pub fn new(min_y: i32, height: i32) -> Self {
        Self {
            min_y,
            height,
            blocks: vec![AIR; 16 * 16 * height as usize],
        }
    }

    #[inline]
    pub fn at(&self, x: i32, y: i32, z: i32) -> BlockId {
        self.blocks[((y - self.min_y) as usize * 256) + z as usize * 16 + x as usize]
    }

    #[inline]
    pub fn at_mut(&mut self, x: i32, y: i32, z: i32) -> &mut BlockId {
        &mut self.blocks[((y - self.min_y) as usize * 256) + z as usize * 16 + x as usize]
    }

    pub fn data(&self) -> &[BlockId] {
        &self.blocks
    }
}
