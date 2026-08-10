#pragma once
// deprecated-vegetation/vegetation_features.h — 树花植被 Feature 实现（已废弃，非必要勿动）
//
// ⚠️ 本文件为废弃代码归档（2026-08-10 深夜从 feature.h 剪出，见本目录 README.md）：
//   - 树花植被（SimpleBlock/RandomPatch/Tree/RandomSelector）已实现但 2026-08-10 用户拍板不做
//   - 原因：细节版本改动太多 + MOD 特别容易碰到的位置，兼容工作量不可接受
//   - 规则：非必要直接无视本文件；不参与编译、不接入调度（feature_loader.h 已移除对应分支）
//   - 恢复方法见 README.md
//
// 原代码（feature.h 2026-08-10 c04768e 版本）：
//   - SimpleBlockFeature（Java SimpleBlockFeature.java + noise_provider 简化）
//   - RandomPatchFeature（Java RandomPatchFeature.java）
//   - TreeFeature（Java TreeFeature.java + StraightTrunkPlacer + BlobFoliagePlacer，oak/birch 直树，fancy_oak 简化）
//   - RandomSelectorFeature（Java random_selector——trees_flower_forest 等）
//
// 依赖：feature.h（OreFeatureContext / BlockRegistry）——剪出时保持原状未调整；如需单独编译需自行补依赖

#include <algorithm> // std::max
#include "../feature.h"

namespace wg {

// ===== SimpleBlockFeature（Java SimpleBlockFeature.java + noise_provider 简化）=====
struct SimpleBlockFeatureConfig {
    std::vector<int> states;      // to_place 状态列表（noise_provider 简化：states[0]；simple 单状态）
    bool noiseProvider = false;
    double scale = 0.0;
    int noiseSeed = 0;

    static SimpleBlockFeatureConfig parse(const JsonValue* cfg, BlockRegistry& blocks) {
        SimpleBlockFeatureConfig sc;
        if (!cfg) return sc;
        if (const JsonValue* tp = cfg->get("to_place")) {
            std::string ttype = tp->get("type") ? tp->get("type")->strVal : "";
            if (ttype.find("noise_provider") != std::string::npos) {
                sc.noiseProvider = true;
                if (const JsonValue* s = tp->get("scale")) sc.scale = s->numVal;
                if (const JsonValue* sd = tp->get("seed")) sc.noiseSeed = (int)sd->numVal;
                if (const JsonValue* st = tp->get("states")) {
                    for (const auto& s : st->arr) {
                        sc.states.push_back(blocks.id(s.get("Name") ? s.get("Name")->strVal : ""));
                    }
                }
            } else if (const JsonValue* st = tp->get("state")) {
                sc.states.push_back(blocks.id(st->get("Name") ? st->get("Name")->strVal : ""));
            } else if (const JsonValue* st = tp->get("state_provider")) {
                // 嵌套 simple_state_provider
                if (const JsonValue* fb = st->get("fallback")) {
                    if (const JsonValue* s2 = fb->get("state")) {
                        sc.states.push_back(blocks.id(s2->get("Name") ? s2->get("Name")->strVal : ""));
                    }
                }
            }
        }
        return sc;
    }
};

class SimpleBlockFeature {
public:
    bool generate(OreFeatureContext& ctx, const SimpleBlockFeatureConfig& config) {
        int x = ctx.originX, y = ctx.originY, z = ctx.originZ;
        if (config.states.empty()) return false;
        int state = config.states[0]; // noise_provider 简化：states[0]（Phase 5 后补 Perlin 精确）
        ctx.setBlock(x, y, z, state);
        return true;
    }
};

// ===== RandomPatchFeatureConfig（Java RandomPatchFeatureConfig.java）=====
struct RandomPatchFeatureConfig {
    int tries = 0;
    int xzSpread = 0;
    int ySpread = 0;
    // 内嵌 configured feature（simple_block 等）+ 简化 placement（block_predicate_filter matching_blocks air）
    SimpleBlockFeatureConfig simpleConfig;
    bool requireAir = false;

    static RandomPatchFeatureConfig parse(const JsonValue* cfg, BlockRegistry& blocks) {
        RandomPatchFeatureConfig rc;
        if (!cfg) return rc;
        if (const JsonValue* t = cfg->get("tries")) rc.tries = (int)t->numVal;
        if (const JsonValue* x = cfg->get("xz_spread")) rc.xzSpread = (int)x->numVal;
        if (const JsonValue* y = cfg->get("y_spread")) rc.ySpread = (int)y->numVal;
        if (const JsonValue* f = cfg->get("feature")) {
            // feature: { "feature": { configured feature }, "placement": [...] }
            if (const JsonValue* feat = f->get("feature")) {
                rc.simpleConfig = SimpleBlockFeatureConfig::parse(feat->get("config"), blocks);
            }
            // placement：block_predicate_filter matching_blocks air → requireAir
            if (const JsonValue* pl = f->get("placement")) {
                for (const auto& m : pl->arr) {
                    std::string t = m.get("type") ? m.get("type")->strVal : "";
                    if (t.find("block_predicate_filter") != std::string::npos) {
                        if (const JsonValue* pred = m.get("predicate")) {
                            std::string pt = pred->get("type") ? pred->get("type")->strVal : "";
                            if (pt.find("matching_blocks") != std::string::npos) rc.requireAir = true;
                        }
                    }
                }
            }
        }
        return rc;
    }
};

// ===== RandomPatchFeature（Java RandomPatchFeature.java）=====
class RandomPatchFeature {
public:
    bool generate(OreFeatureContext& ctx, const RandomPatchFeatureConfig& config) {
        int x = ctx.originX, y = ctx.originY, z = ctx.originZ;
        int airId = ctx.blocks.id("minecraft:air");
        int j = config.xzSpread + 1;
        int k = config.ySpread + 1;
        int placed = 0;
        int airHit = 0;
        for (int i = 0; i < config.tries; i++) {
            // Java：nextInt(j) - nextInt(j)（两个 nextInt 相减 → -(j-1)..(j-1)，消费 2 次随机）
            int px = x + ctx.random.nextInt(j) - ctx.random.nextInt(j);
            int py = y + ctx.random.nextInt(k) - ctx.random.nextInt(k);
            int pz = z + ctx.random.nextInt(j) - ctx.random.nextInt(j);
            int cur = ctx.blockAt(px, py, pz);
            if (config.requireAir) {
                if (cur != airId) continue;
                airHit++;
            }
            ctx.originX = px; ctx.originY = py; ctx.originZ = pz;
            SimpleBlockFeature f;
            if (f.generate(ctx, config.simpleConfig)) placed++;
        }
        if (getenv("WG_FEATURELOG")) {
            std::fprintf(stderr, "[RPATCH] tries=%d xz=%d yz=%d requireAir=%d airHit=%d placed=%d\n",
                         config.tries, config.xzSpread, config.ySpread, (int)config.requireAir, airHit, placed);
        }
        ctx.originX = x; ctx.originY = y; ctx.originZ = z;
        return placed > 0;
    }
};

// ===== TreeFeature（Java TreeFeature.java——oak/birch 直树，fancy_oak 简化）=====
// Java 参照：world/gen/feature/TreeFeature.java + trunkplacers/StraightTrunkPlacer.java + foliagesplacer/BlobFoliagePlacer.java
struct TreeFeatureConfig {
    // trunk_placer：straight（base + nextInt(randA) + nextInt(randB)）
    int trunkBase = 0, trunkRandA = 0, trunkRandB = 0;
    // foliage_placer：blob（radius/height/offset）
    int blobRadius = 0, blobHeight = 0, blobOffset = 0;
    // minimum_size：two_layers（limit/lower/upper）
    int sizeLimit = 0, sizeLower = 0, sizeUpper = 0;
    // 方块
    int trunkState = 0;   // oak_log / birch_log
    int foliageState = 0; // oak_leaves / birch_leaves
    int dirtState = 0;    // dirt
    bool forceDirt = false;
    bool ignoreVines = true;
    // 可放树的地面（dirt/grass_block/podzol/coarse_dirt/mycelium）
    std::vector<int> groundBlocks;

    static TreeFeatureConfig parse(const JsonValue* cfg, BlockRegistry& blocks) {
        TreeFeatureConfig tc;
        if (!cfg) return tc;
        if (const JsonValue* tp = cfg->get("trunk_placer")) {
            if (const JsonValue* b = tp->get("base_height")) tc.trunkBase = (int)b->numVal;
            if (const JsonValue* a = tp->get("height_rand_a")) tc.trunkRandA = (int)a->numVal;
            if (const JsonValue* b2 = tp->get("height_rand_b")) tc.trunkRandB = (int)b2->numVal;
        }
        if (const JsonValue* fp = cfg->get("foliage_placer")) {
            if (const JsonValue* r = fp->get("radius")) tc.blobRadius = (int)r->numVal;
            if (const JsonValue* h = fp->get("height")) tc.blobHeight = (int)h->numVal;
            if (const JsonValue* o = fp->get("offset")) tc.blobOffset = (int)o->numVal;
        }
        if (const JsonValue* ms = cfg->get("minimum_size")) {
            if (const JsonValue* l = ms->get("limit")) tc.sizeLimit = (int)l->numVal;
            if (const JsonValue* lw = ms->get("lower_size")) tc.sizeLower = (int)lw->numVal;
            if (const JsonValue* up = ms->get("upper_size")) tc.sizeUpper = (int)up->numVal;
        }
        if (const JsonValue* tr = cfg->get("trunk_provider")) {
            if (const JsonValue* st = tr->get("state")) tc.trunkState = blocks.id(st->get("Name") ? st->get("Name")->strVal : "");
        }
        if (const JsonValue* fl = cfg->get("foliage_provider")) {
            if (const JsonValue* st = fl->get("state")) tc.foliageState = blocks.id(st->get("Name") ? st->get("Name")->strVal : "");
        }
        if (const JsonValue* dp = cfg->get("dirt_provider")) {
            if (const JsonValue* st = dp->get("state")) tc.dirtState = blocks.id(st->get("Name") ? st->get("Name")->strVal : "");
        }
        if (const JsonValue* fd = cfg->get("force_dirt")) tc.forceDirt = fd->boolVal;
        if (const JsonValue* iv = cfg->get("ignore_vines")) tc.ignoreVines = iv->boolVal;
        for (const char* g : {"minecraft:dirt", "minecraft:grass_block", "minecraft:podzol", "minecraft:coarse_dirt", "minecraft:mycelium"}) {
            tc.groundBlocks.push_back(blocks.id(g));
        }
        return tc;
    }
};

class TreeFeature {
public:
    bool generate(OreFeatureContext& ctx, const TreeFeatureConfig& config) {
        int x = ctx.originX, y = ctx.originY, z = ctx.originZ;
        int height = config.trunkBase + ctx.random.nextInt(config.trunkRandA + 1) + ctx.random.nextInt(config.trunkRandB + 1);
        int airId = ctx.blocks.id("minecraft:air");
        int waterId = ctx.blocks.id("minecraft:water");
        // canGenerate：地面必须是 dirt/grass；树干空间必须 air（Java TreeFeature.canGenerate）
        int ground = ctx.blockAt(x, y - 1, z);
        bool groundOk = false;
        for (int g : config.groundBlocks) if (ground == g) { groundOk = true; break; }
        if (!groundOk) return false;
        for (int j = 1; j < height + 1; j++) {
            int k = (config.ignoreVines || j < config.sizeLimit) ? 1 : 2;
            for (int l = -k; l <= k; l++) {
                for (int m = -k; m <= k; m++) {
                    int b = ctx.blockAt(x + l, y + j, z + m);
                    if (b < 0 || (b != airId && b != waterId)) return false;
                }
            }
        }
        // 树干：从 y 到 y+height-1 每格 log（StraightTrunkPlacer）
        int topY = y + height - 1;
        for (int i = 0; i < height; i++) {
            ctx.setBlock(x, y + i, z, config.trunkState);
        }
        // 树冠（BlobFoliagePlacer.generate：从 offset 到 offset-foliageHeight，每层半径递减）
        // Java: for i = offset; i >= offset - foliageHeight; i--: j = max(radius + radiusOffset - 1 - i/2, 0)
        // radiusOffset=0（oak 单节点）；节点 pos = (x, topY, z)
        int foliageHeight = config.blobHeight;
        for (int i = config.blobOffset; i >= config.blobOffset - foliageHeight; i--) {
            int j = std::max(config.blobRadius + 0 - 1 - i / 2, 0);
            generateSquare(ctx, config, x, topY + i, z, j, i == 0 /*hasTrunk: 最顶层? Java generateSquare 的 trunk 标记 */);
        }
        // force_dirt：树底放 dirt
        if (config.forceDirt) ctx.setBlock(x, y - 1, z, config.dirtState);
        return true;
    }
private:
    // Java BlobFoliagePlacer.generateSquare：4 方向 + 中心（radius j）
    void generateSquare(OreFeatureContext& ctx, const TreeFeatureConfig& config, int cx, int cy, int cz, int radius, bool hasTrunk) {
        if (radius > 0) {
            for (int dx = -radius; dx <= radius; dx++) {
                for (int dz = -radius; dz <= radius; dz++) {
                    if (dx == 0 && dz == 0) continue; // 中心树干位置
                    placeLeaf(ctx, config, cx + dx, cy, cz + dz);
                }
            }
        }
        // 中心：hasTrunk 时不覆盖树干，否则放树叶
        if (!hasTrunk) placeLeaf(ctx, config, cx, cy, cz);
    }
    void placeLeaf(OreFeatureContext& ctx, const TreeFeatureConfig& config, int x, int y, int z) {
        int cur = ctx.blockAt(x, y, z);
        int airId = ctx.blocks.id("minecraft:air");
        int waterId = ctx.blocks.id("minecraft:water");
        if (cur == airId || cur == waterId) ctx.setBlock(x, y, z, config.foliageState);
    }
};

// ===== RandomSelectorFeature（Java random_selector——trees_flower_forest 等）=====
struct RandomSelectorFeatureConfig {
    std::string defaultFeature;   // "minecraft:oak_bees_002"
    std::vector<std::pair<float, std::string>> features; // (chance, id)

    static RandomSelectorFeatureConfig parse(const JsonValue* cfg, BlockRegistry& blocks) {
        (void)blocks;
        RandomSelectorFeatureConfig rc;
        if (!cfg) return rc;
        if (const JsonValue* d = cfg->get("default")) rc.defaultFeature = d->strVal;
        if (const JsonValue* f = cfg->get("features")) {
            for (const auto& e : f->arr) {
                float ch = e.get("chance") ? (float)e.get("chance")->numVal : 0.0f;
                std::string id = e.get("feature") ? e.get("feature")->strVal : "";
                rc.features.push_back({ch, id});
            }
        }
        return rc;
    }
};

} // namespace wg
