# placeBadlandsPillar 实现（C++ worldgen，对齐 MC 1.20.1 vanilla）

- 日期/会话：Phase 2（worker swe 子代理）
- 目标：补齐 `SurfaceBuilder.buildSurface` 中缺失的 `placeBadlandsPillar` 前置填充 + 主循环起点抬升（chunk(50,-26) seed 8576294172403134396 的 797 块差异根因）
- 范围：仅 `versions/1.20.1/cpp/worldgen/src/surface.h` 一个文件

## 1. 改动说明

### 文件 `versions/1.20.1/cpp/worldgen/src/surface.h`

| 位置 | 改动 |
|---|---|
| L443-445（类私有区，`std::vector<int> terracottaBands;` 之后） | 新增成员声明 `void placeBadlandsPillar(BlockColumn& col, int wx, int wz, int cx, int cz, int surfaceY, int& columnHeight, int bottomY, int worldTopY);` |
| L703-718（`buildSurface` 列循环开头） | 重构：原 `int p = heightmap[l*16+k] + 1;` 拆为「pillar 前 surfaceY(o)」→「biome 采样(eroded_badlands 判断)」→「pillar 填充」→「pillar 后 p」 |
| L781-818（`buildSurface` 定义之后、`NoiseThresholdCond::nextId` 之前） | 新增 `inline void SurfaceBuilder::placeBadlandsPillar(...)` 实现 |

### buildSurface 列循环时序（对齐 Java L113-131）

```
旧：p = heightmap[l*16+k]+1 → ctx 初始化 → 主循环 from p
新：o = heightmap[idx]+1                    // Java L117（pillar 前 heightmap+1）
    biome = biomeAtCached(m, o, n)          // Java L119（y=o 采样 biome）
    if (biome == "minecraft:eroded_badlands")  // Java L120 matchesKey(ERODED_BADLANDS)
        placeBadlandsPillar(...)            // Java L121
    p = heightmap[idx]+1                    // Java L124（pillar 后重采样 heightmap+1 = j+2）
    ctx 初始化 → 主循环 from p
```

`heightmap[idx]`（idx = l*16+k，与 fillOneChunk 填充语义一致）即 WORLD_SURFACE_WG 高度。

## 2. placeBadlandsPillar 实现（Java L208-234 逐行对应）

```cpp
inline void SurfaceBuilder::placeBadlandsPillar(BlockColumn& col, int wx, int wz, int cx, int cz,
                                                int surfaceY, int& columnHeight, int bottomY, int worldTopY) {
    const int defaultBlock = blocks->id("minecraft:stone");
    const int airBlock = blocks->id("minecraft:air");
    const int waterBlock = blocks->id("minecraft:water");
    // e = min(|badlands_surface(x,0,z)*8.25|, badlands_pillar(x*0.2,0,z*0.2)*15.0)
    double e = std::min(std::abs(getNoise("minecraft:badlands_surface").sample(wx, 0.0, wz) * 8.25),
                        getNoise("minecraft:badlands_pillar").sample(wx * 0.2, 0.0, wz * 0.2) * 15.0);
    if (e <= 0.0) return;                 // Java L211
    double h = std::abs(getNoise("minecraft:badlands_pillar_roof").sample(wx * 0.75, 0.0, wz * 0.75) * 1.5);
    double i = 64.0 + std::min(e * e * 2.5, std::ceil(h * 50.0) + 24.0);   // Java L215
    int j = (int)std::floor(i);           // Java L216 MathHelper.floor（向 -inf）
    if (surfaceY > j) return;             // Java L217 surfaceY <= j 才填充
    for (int y = j; y >= bottomY; y--) {  // Java L218-227 校验：遇 stone break、遇 water 整列 return
        int state = (y >= worldTopY) ? airBlock : col.at(cx, y, cz);
        if (state == defaultBlock) break;  // isOf(defaultState.getBlock())：方块类型==stone
        if (state == waterBlock) return;   // isOf(Blocks.WATER)
    }
    bool filled = false;
    for (int y = j; y >= bottomY; y--) {  // Java L229-231 填充：从 j 向下 while air → stone
        int state = (y >= worldTopY) ? airBlock : col.at(cx, y, cz);
        if (state != airBlock) break;
        if (y < worldTopY) { col.at(cx, y, cz) = defaultBlock; filled = true; }
    }
    if (filled) columnHeight = std::max(columnHeight, j + 1);   // Java trackUpdate：首个填充 y=j → heightmap=j+1
}
```

### j 公式（Java L210-216）
- `e = min(|badlands_surface(x, 0, z) * 8.25|, badlands_pillar(x*0.2, 0, z*0.2) * 15.0)`
  - badlands_surface 用**原始坐标**（wx, wz）；badlands_pillar 用 **x*0.2 / z*0.2**
  - pillar 项**无 abs**（Java 源码如此），可为负 → `e <= 0` 时整列跳过
- `h = |badlands_pillar_roof(x*0.75, 0, z*0.75) * 1.5|`（采样坐标 x*0.75 / z*0.75）
- `i = 64.0 + min(e*e*2.5, ceil(h*50.0) + 24.0)`；`j = floor(i)`（向 -inf）

### heightmap 抬升（Java trackUpdate 等效）
- Java `Chunk.setBlockState` → `Heightmap.update(x,y,z)`：仅当 `y >= heights - 1` 且 opaque 时 `heights = y+1`
- 填充从 j 向下，首个填充 y=j（stone opaque，j ≥ surfaceY > 原 heightmap）→ heightmap = j+1；后续 y<j 满足 `y <= heights-2` 不再触发
- C++ 用 `columnHeight = std::max(columnHeight, j+1)`（前提 `filled`，与 Java「setState 越界无效 → 无更新」一致）
- 主循环起点 `p = heightmap+1` 在 pillar 后重算，即 j+2

### 关键语义对齐点
- **biome 采样在 pillar 前**、用 **pillar 前 heightmap+1(o)** 的高度（Java L119 `getBiome(m, o, n)`）；非 eroded_badlands 列零行为变化（timing 不变、无额外写入）
- 校验用 `state == defaultBlock`（方块类型 == stone，deepslate 等不同 Block 不 break，与 Java `isOf` 一致）
- 水判定仅 `minecraft:water` 方块（Java `isOf(Blocks.WATER)`），非流体状态判断
- air 判定 `state == airBlock`，与主循环 `isAir` 一致

## 3. @anchor 标注

- `surface.h` L782：
  `// @anchor.test("placeBadlandsPillar 对齐 Java SurfaceBuilder.placeBadlandsPillar（eroded_badlands pillar 顶/填充/heightmap 抬升）", source="probe:block_probe!PILLAR#001")`
- 位于 `placeBadlandsPillar` 实现定义（L783）正上方，与现有 `SURF#001/002` anchor 风格一致

## 4. 编译结果

### 状态：代码改动完成；**编译在本子代理环境无法执行（被权限拦截）**，需父代理（有 Bash 工具）执行下述命令

- 本子代理工具集无 bash/shell 工具；只读子代理验证（`read_only_task`）尝试执行删除 obj 与 `cmake --build` 均被权限层拦截（`blocked: read-only subagents can run only permission-classified foreground read-only commands`）
- LSP（clangd）不可用（未安装），无语言服务器诊断

### 父代理执行命令（参考本会话已知可用命令；MSVC + Ninja）

```powershell
cd E:\PYTHON\CoreSwap\versions\1.20.1\cpp\build-msvc
Remove-Item worldgen\CMakeFiles\worldgen_core.dir\src\worldgen_api.cpp.obj -Force -ErrorAction SilentlyContinue
cmd /c 'call "D:\Program Files\Microsoft Visual Studio\18\Community\VC\Auxiliary\Build\vcvars64.bat" >nul 2>&1 && set PATH=D:\Program Files\Microsoft Visual Studio\18\Community\Common7\IDE\CommonExtensions\Microsoft\CMake\Ninja;%PATH% && cmake --build . --target block_probe got_export 2>&1' | Select-Object -Last 20
```

说明：
- `surface.h` 只被 `worldgen_api.cpp` 包含（`tbands_test.cpp` 未编入 worldgen_core）；删 `worldgen_api.cpp.obj` 即强制重编；`bin\worldgen_core.lib` 会在 obj 重建后由 ninja 自动重链
- 编译通过标准：ninja 输出含 `worldgen_api.cpp.obj` 重编 + `worldgen_core.lib` 重链 + `block_probe.exe` / `got_export.exe` 链接成功，退出码 0

### 代码自审（替代编译的人工验证）
- `std::abs/std::floor/std::ceil` 依赖 `<cmath>`（surface.h L4 已含）；`std::min/std::max` 依赖 `<algorithm>`（现有 `SteepCond`/`SurfaceCondC` 已使用，编译环境必然可用）
- 新增声明（L444-445）与定义（L783-784）签名一致；调用点（L713）在 `buildSurface` 内，成员声明在类内可见
- 所有新变量（`idx`/`o`/`pillarBiome`）与现有作用域无冲突；`worldTopY`/`minY`/`heightmap`/`biomeAtCached` 均在 `buildSurface` 作用域内
- 与 Java 逐位对应关系见上文第 2 节

## 5. 已知边界

- **y=60 light_gray_terracotta 未覆盖**：`getTerracottaBlock` 用 `clay_bands_offset` 2D 噪声列缓存 + `std::lround`，红陶带数组随机生成已对齐（192 项，`clay_bands` splitter）；light_gray 带只出现在 `k±1` 随机布尔处（L370-371），属概率性存在，此问题不在本改动范围
- **越界防御**：pillar 填充循环对 `y >= worldTopY` 的读按 Java `getState` 越界返回 AIR、写跳过（Java `setState` 越界无效），语义与 vanilla 一致；实际 overworld 中 `j ≤ ~190 << 320`，该分支不会触发
- **净下界/其他维度**：`buildSurface` 仅 overworld 使用（nether 不调用 surface 规则），`bottomY`=维度 minY；若未来复用需确认 pillar 噪声注册（`badlands_surface/badlands_pillar/badlands_pillar_roof` 已在 worldgen_api.cpp L112-114/390 注册，overworld 专属）
- **非 eroded_badlands 列零行为变化**：新增逻辑仅在一个 biome 分支内，其余路径与改动前逐字节相同（timing 不变）
- **性能**：每列新增 1 次 biomeAtCached（cell 缓存，主循环内复用）与 pillar 分支内 ≤3 次 2D 噪声采样，量级可忽略

## 6. 后续验证（Phase 2.5，本任务不执行）

- `block_probe` 回归：chunk(50,-26)（seed 8576294172403134396）应消除 797 块差异（C++ air vs Java terracotta）
- 3200 回归铁律：非 eroded_badlands 区块应零差异
