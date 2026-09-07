// block_tags.rs — 方块 tag 注册表（#minecraft:xxx 展开），数据驱动（260907-04）
// 数据源：<wg_dir>/data/minecraft/tags/blocks/*.json（从 server jar 提取，170 文件）。
// 消费点：carver.rs build_overworld_replaceable + feature.rs expand_tag。
// 语义对齐 Java TagJson / RegistryEntryList（1.20.1）：
//   - 文件格式 {"replace":bool,"values":[ "minecraft:x" | "#minecraft:x" | {"id":..,"required":bool} ]}
//   - "#tag" 前缀 = 嵌套 tag 递归展开（环检测：路径 visited 集）
//   - "replace" 字段仅影响 datapack 合并语义（vanilla 单源数据无合并），忽略
//   - required=false 条目未知时静默跳过；required=true 条目未知时一次性日志 + 跳过
//     （blocks.id 对未知名返回 AIR——跳过比 push AIR 更安全，防「tag 含 air」假语义；
//      注意：fallback 硬编码路径保留旧 blocks.id 行为不变）
// 缺失策略（用户拍板 260907-04）：tag 文件缺失/解析失败 → 调用方 fallback 到硬编码集 + 一次性日志，
// 不 fail（跨版本升级数据未跟上不炸生成管线）。

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock, RwLock};

use crate::blocks::{BlockId, BlockRegistry};
use crate::json::{self, JsonValue};

pub struct BlockTagRegistry {
    /// tags 根目录所在 wg_dir（tag 文件 = <wg_dir>/data/<ns>/tags/blocks/<path>.json）
    wg_dir: String,
    /// tag id -> 展开结果缓存（None = 文件缺失）
    cache: Mutex<HashMap<String, Option<Vec<String>>>>,
    /// 一次性日志去重
    logged: Mutex<HashSet<String>>,
}

static REGISTRY: OnceLock<RwLock<Option<BlockTagRegistry>>> = OnceLock::new();

fn registry() -> &'static RwLock<Option<BlockTagRegistry>> {
    REGISTRY.get_or_init(|| RwLock::new(None))
}

/// 初始化/替换全局注册表（worldgen_handle::create_for_dim 在 blocks 加载后调用；
/// bin-diag 未初始化时 expand 返回 false → 调用方 fallback，向后兼容）。
/// 边界（judge 260907-04 条件 3）：全局单例 last-init-wins——同进程多 handle
/// **不同** wg_dir 时后 init 覆盖前者（tag 解析跟着最后创建的 handle 走）。
/// 当前生产/诊断场景各 handle 共用同一解压目录（CoreSwapFixHelper 单一
/// coreswap-data 路径），无实际冲突；未来多 wg_dir 并存时须改 per-handle 注入。
pub fn init(wg_dir: &str) {
    let mut g = registry().write().unwrap();
    let changed = match g.as_ref() {
        Some(r) => r.wg_dir != wg_dir,
        None => true,
    };
    if changed {
        *g = Some(BlockTagRegistry {
            wg_dir: wg_dir.to_string(),
            cache: Mutex::new(HashMap::new()),
            logged: Mutex::new(HashSet::new()),
        });
    }
}

/// 当前注册表的 wg_dir（诊断用；未初始化返回 None）
pub fn current_dir() -> Option<String> {
    registry().read().unwrap().as_ref().map(|r| r.wg_dir.clone())
}

impl BlockTagRegistry {
    fn tag_path(&self, tag: &str) -> Option<PathBuf> {
        // tag 形如 "minecraft:foo"（默认 ns=minecraft）→ data/<ns>/tags/blocks/foo.json
        let (ns, path) = match tag.split_once(':') {
            Some((ns, p)) if !ns.is_empty() && !p.is_empty() => (ns, p),
            _ => return None, // 无命名空间/畸形 id → 不查表（走 fallback）
        };
        Some(PathBuf::from(&self.wg_dir).join("data").join(ns).join("tags").join("blocks")
            .join(format!("{}.json", path.replace('\\', "/"))))
    }

    /// 读单文件原始 values（字符串列表，含 "#" 前缀与 required 对象归一化）。
    /// 返回 None = 文件缺失/畸形（→ 整 tag fallback）。
    fn load_raw(&self, tag: &str) -> Option<Vec<String>> {
        let path = self.tag_path(tag)?;
        let txt = std::fs::read_to_string(path).ok()?;
        let root = json::parse(&txt).ok()?;
        let arr = root.get("values")?.as_array()?;
        let mut out = Vec::with_capacity(arr.len());
        for entry in arr {
            // 字符串形态
            if let Some(s) = entry.as_str() {
                out.push(s.to_string());
                continue;
            }
            // 对象形态 {"id": "..", "required": bool}
            if let Some(id) = entry.get("id").and_then(|x| x.as_str()) {
                let required = entry.get("required").and_then(|x| x.as_bool()).unwrap_or(true);
                if !required {
                    // 可选条目：解析失败时不参与——先记录原值，resolve 阶段按 known 过滤
                    out.push(format!("?{}", id)); // '?' 前缀 = optional 标记
                } else {
                    out.push(id.to_string());
                }
            }
            // 其余形态（1.20.1 vanilla 不存在）静默忽略
        }
        Some(out)
    }

    fn log_once(&self, msg: &str) {
        let mut lg = self.logged.lock().unwrap();
        if lg.insert(msg.to_string()) {
            eprintln!("[block_tags] {}", msg);
        }
    }

    /// 递归展开 tag → 原始条目名列表（含嵌套 tag 展开，环检测）。
    fn expand_raw(&self, tag: &str, visited: &mut Vec<String>, depth: usize) -> Option<Vec<String>> {
        if depth > 16 {
            self.log_once(&format!("tag nesting > 16 at '{}' (cycle?) — fallback", tag));
            return None;
        }
        if visited.iter().any(|v| v == tag) {
            self.log_once(&format!("tag cycle detected at '{}' — fallback", tag));
            return None;
        }
        if let Some(cached) = self.cache.lock().unwrap().get(tag) {
            return cached.clone();
        }
        let raw = self.load_raw(tag)?;
        visited.push(tag.to_string());
        let mut out: Vec<String> = Vec::new();
        for entry in &raw {
            if let Some(nested) = entry.strip_prefix('#') {
                match self.expand_raw(nested, visited, depth + 1) {
                    Some(sub) => out.extend(sub),
                    None => {
                        // 嵌套 tag 缺失：整个外层 tag 数据不完整 → 整体 fallback（保守）
                        visited.pop();
                        self.cache.lock().unwrap().insert(tag.to_string(), None);
                        self.log_once(&format!("nested tag '#{}' missing (in '{}') — fallback", nested, tag));
                        return None;
                    }
                }
            } else {
                out.push(entry.clone());
            }
        }
        visited.pop();
        self.cache.lock().unwrap().insert(tag.to_string(), Some(out.clone()));
        Some(out)
    }
}

/// 展开 tag → BlockId 列表。返回 true = JSON 数据命中（out 已填充）；
/// false = 数据缺失/畸形（out 未动，调用方走硬编码 fallback）。
/// 未知名处理：required 条目未知 → 一次性日志 + 跳过；optional（'?' 前缀）未知 → 静默跳过。
pub fn expand_tag(tag: &str, blocks: &BlockRegistry, out: &mut Vec<BlockId>) -> bool {
    let g = registry().read().unwrap();
    let reg = match g.as_ref() { Some(r) => r, None => return false };
    let raw = match reg.expand_raw(tag, &mut Vec::new(), 0) {
        Some(r) => r,
        None => {
            reg.log_once(&format!("tag '{}' not resolved from JSON — hardcoded fallback", tag));
            return false;
        }
    };
    let before = out.len();
    for entry in &raw {
        if let Some(opt) = entry.strip_prefix('?') {
            if blocks.contains(opt) { out.push(blocks.id(opt)); }
        } else if blocks.contains(entry) {
            out.push(blocks.id(entry));
        } else {
            reg.log_once(&format!("tag '{}': unknown block '{}' skipped", tag, entry));
        }
    }
    if out.len() == before {
        // 全部条目未解析（如 blocks.json 不含该域）——视为数据不完整，仍算命中但为空
        // 与 Java「tag 解析成功但引用全部缺失」语义一致（空集），调用方不 fallback。
    }
    true
}

// ===== golden 等值测试（架构计划要点 3：JSON 展开 vs 硬编码 fallback 逐位对比）=====

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blocks::BlockRegistry;

    const WG_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../versions/1.20.1/data/worldgen");

    fn load_blocks() -> BlockRegistry {
        let txt = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"),
            "/../versions/1.20.1/data/blocks.json")).expect("blocks.json");
        BlockRegistry::load_from_json(&txt).expect("parse blocks.json")
    }

    fn sort_dedup(v: &mut Vec<BlockId>) {
        v.sort_unstable();
        v.dedup();
    }

    /// golden 1：carver replaceable —— JSON 展开(overworld_carver_replaceables) == 硬编码 NAMES
    #[test]
    fn golden_carver_replaceables() {
        crate::block_tags::init(WG_DIR);
        let blocks = load_blocks();
        let mut json_ids = Vec::new();
        assert!(expand_tag("minecraft:overworld_carver_replaceables", &blocks, &mut json_ids),
            "JSON tag must resolve (data files extracted)");
        let mut fallback = crate::carver::CarverConfig::build_overworld_replaceable(&blocks);
        sort_dedup(&mut json_ids);
        sort_dedup(&mut fallback);
        assert_eq!(json_ids, fallback,
            "JSON expansion must equal hardcoded fallback (block id sets)");
    }

    /// golden 2：feature expand_tag 消费的 6 个 tag（netherrack 无 tag 文件，由 fallback 覆盖）
    #[test]
    fn golden_feature_tags() {
        crate::block_tags::init(WG_DIR);
        let blocks = load_blocks();
        let tags = [
            "minecraft:base_stone_overworld",
            "minecraft:stone_ore_replaceables",
            "minecraft:deepslate_ore_replaceables",
            "minecraft:base_stone_nether",
            "minecraft:sand",
            "minecraft:dirt",
        ];
        for t in tags {
            let mut json_ids = Vec::new();
            assert!(expand_tag(t, &blocks, &mut json_ids), "tag {} must resolve", t);
            let mut fb = Vec::new();
            crate::feature::expand_tag_fallback(&blocks, t, &mut fb);
            sort_dedup(&mut json_ids);
            sort_dedup(&mut fb);
            assert_eq!(json_ids, fb, "tag {} JSON == fallback", t);
        }
    }

    /// golden 3（负向）：tag 文件缺失 → expand_tag 返回 false（fallback 协议）
    /// —— 用独立注册表（指向不存在目录）验证，不污染全局。
    #[test]
    fn negative_missing_dir_falls_back() {
        // 直接构造 registry 而非全局 init（全局已被其他测试设置）
        let reg = BlockTagRegistry {
            wg_dir: "Z:/nonexistent-260907-04".to_string(),
            cache: Mutex::new(HashMap::new()),
            logged: Mutex::new(HashSet::new()),
        };
        assert!(reg.load_raw("minecraft:overworld_carver_replaceables").is_none(),
            "missing dir must yield None (fallback signal)");
    }
}
