// blocks.rs — 方块 ID 表 + 区块方块存储（16×16×height）
// C++ 参考：versions/1.20.1/cpp/worldgen/src/blocks.h（BlockId = vanilla block 注册表 raw id）
// 数据源：versions/1.20.1/data/blocks.json（{ "minecraft:stone": 1, ... }）

use std::collections::HashMap;

use crate::json;

pub type BlockId = i32;

pub const AIR: BlockId = 0;

// 方块 ID 注册表：从 blocks.json 加载 id↔name 双向表
// 260907-05（A 组缺口 1）：支持运行时注册（wg_register_block，mod 方块前置）；
// 未注册名查询不再静默回 AIR——一次性 stderr 告警（#27 先例：fallback 必配日志）。
#[derive(Default)]
pub struct BlockRegistry {
    name_to_id: std::sync::RwLock<HashMap<String, BlockId>>,
    id_to_name: std::sync::RwLock<Vec<String>>,
    // 运行时注册分配的下一个动态 id（load 后 = max(existing)+1）
    next_id: std::sync::atomic::AtomicI32,
    // 已告警过的未知名（一次性日志去重）
    warned: std::sync::Mutex<std::collections::HashSet<String>>,
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
        let mut max_id = 0;
        for (name, id) in &name_to_id {
            if *id >= 0 && (*id as usize) < id_to_name.len() {
                id_to_name[*id as usize] = name.clone();
            }
            if *id > max_id { max_id = *id; }
        }
        if name_to_id.is_empty() { return None; }
        Some(Self {
            name_to_id: std::sync::RwLock::new(name_to_id),
            id_to_name: std::sync::RwLock::new(id_to_name),
            next_id: std::sync::atomic::AtomicI32::new(max_id + 1),
            warned: std::sync::Mutex::new(std::collections::HashSet::new()),
        })
    }

    pub fn id(&self, name: &str) -> BlockId {
        if let Some(id) = self.name_to_id.read().unwrap().get(name) {
            return *id;
        }
        // 未注册名：一次性告警（不静默回 AIR）
        if let Ok(mut w) = self.warned.lock() {
            if w.insert(name.to_string()) {
                eprintln!("[BLOCKS] unknown block '{}' -> AIR (register via wg_register_block)", name);
            }
        }
        AIR
    }

    // 运行时注册：返回该名字的 id（已存在则返回既有 id；否则分配动态 id = max+1 递增）。
    // 调用时序约定：生成线程启动前（创建期注册，与 fill 并发读无写竞争）。
    pub fn register(&self, name: &str) -> BlockId {
        {
            let map = self.name_to_id.read().unwrap();
            if let Some(id) = map.get(name) { return *id; }
        }
        let mut map = self.name_to_id.write().unwrap();
        if let Some(id) = map.get(name) { return *id; } // 双检（并发注册）
        let id = self.next_id.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        map.insert(name.to_string(), id);
        let mut names = self.id_to_name.write().unwrap();
        if id >= 0 && (id as usize) < names.len() {
            names[id as usize] = name.to_string();
        } else {
            // O1（judge 260907-05）：超 id_to_name 容量不静默
            eprintln!("[BLOCKS] dynamic id {} >= table capacity, name lookup will return '?' for '{}'", id, name);
        }
        eprintln!("[BLOCKS] registered '{}' -> id {}", name, id);
        id
    }

    // 返回名字（克隆；注册表 RwLock 化后不再返回内部引用）。未知 id 返回 "?"。
    pub fn name(&self, id: BlockId) -> String {
        let names = self.id_to_name.read().unwrap();
        if id >= 0 && (id as usize) < names.len() && !names[id as usize].is_empty() {
            names[id as usize].clone()
        } else {
            "?".to_string()
        }
    }

    pub fn contains(&self, name: &str) -> bool {
        self.name_to_id.read().unwrap().contains_key(name)
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

#[cfg(test)]
mod tests {
    use super::*;

    // 260907-05（缺口 1）：运行时注册 + 告警语义自检
    #[test]
    fn register_and_unknown_warn() {
        let reg = BlockRegistry::load_from_json(r#"{"minecraft:stone": 1, "minecraft:air": 0}"#).unwrap();
        assert_eq!(reg.id("minecraft:stone"), 1);
        // 未注册名 → AIR（不 panic）
        assert_eq!(reg.id("mod:block"), AIR);
        // 注册分配 = max+1 递增；已存在返回既有 id
        let id1 = reg.register("mod:block");
        assert_eq!(id1, 2);
        assert_eq!(reg.register("mod:block"), 2);
        assert_eq!(reg.id("mod:block"), 2);
        assert_eq!(reg.name(2), "mod:block");
        assert!(reg.contains("mod:block"));
        let id2 = reg.register("mod:other");
        assert_eq!(id2, 3);
    }
}
