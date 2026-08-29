// surface.rs — SurfaceBuilder / MaterialRules 深带规则（第一阶段：块类型替换的深带部分）。
// 对应 C++ surface.h 的 buildOverworldRule 顶部序列（bedrock_floor + deepslate gradient）
// + buildSurface 列引擎对 defaultBlock(=stone) 的规则应用。
//
// 范围（v1 深带替换）：只实现确定性深带规则：
//   - bedrock_floor : verticalGradient("minecraft:bedrock_floor", minY, minY+5) → bedrock
//   - deepslate     : verticalGradient("minecraft:deepslate", 0, 8) → deepslate（在 stone 上）
//   - tuff 带（silverfish/tuff 嵌入带）: 暂用近似的 y 带（需 noise 规则，v1 延后）
// surface 顶块（草/沙/gravel/雪）、red 陶带、badlands pillar、完整 mr1-10 树 → v2（移植完整 surface.h）。
//
// VerticalGradientCond.test（Java VerticalGradientPredicate）：
//   blockY <= trueAtAndBelow → true（确定性）
//   blockY >= falseAtAndAbove → false
//   否则 → 随机（splitterFor(name).split(x,y,z).nextFloat() < lerp(y, trueY,falseY, 1.0, 0.0)）
//
// 判定顺序 = C++ finalRules：bedrock_floor 先（序列第一），surface→mr9 中间（v1 不处理明确 surface 顶块，
// 保持原块），deepslate 最后（覆盖 stone）。
pub struct DeepSurfaceRule {
    pub min_y: i32,
}

// 深带替换：对单个已分类块 (is_solid, original_block) 应用 bedrock/深板岩规则，返回最终块 id。
// is_solid: 该块是否是实心（密度>0 → stone-family）。v1 只处理实心块。
// original_block: 当前块 id（stone=1/或已在之前阶段设置的块）。
// y: 世界 y；min_y: 维度最低 y。
// 返回替换后的块 id（若不应替换，返回 original_block）。
pub fn apply_deep_rules(original_block: i32, is_solid: bool, y: i32, min_y: i32) -> i32 {
    const STONE: i32 = 1;
    const DEEPSLATE: i32 = 970;
    const BEDROCK: i32 = 31;
    if !is_solid || original_block != STONE {
        // 只处理 solid stone；其它（水/空气/岩浆/已替换块）不动
        return original_block;
    }
    // bedrock_floor: y ∈ [minY, minY+5)（verticalGradient trueAt=minY, falseAt=minY+5）
    // 简化：底部 3 层 bedrock（fixpreview 实测 y<=minY+2 → bedrock 吻合 vanilla）
    if y <= min_y + 2 {
        return BEDROCK;
    }
    // deepslate: y <= 0（fixed(0) 为 trueAt；lake 区 surface 全在 y<0，故 stone 全为 deepslate）
    // 注意：y>0 且 <8 是概率带（v1 简化：y<=0 才替换）
    if y <= 0 {
        return DEEPSLATE;
    }
    original_block
}
