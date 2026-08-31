# M11【复发·第 3 次】跨工具对比 worldSeed 错位——烧掉整条「Octave createLegacy 缺口」结论链（草稿，status: candidate）

> 应用位置：`multiworld-errors.md` 中 **M10 补遗之后、「附：错误 → 根因 速查表」之前**；速查表行插表末（见文末「速查表追加 1 行」）。

## M11【复发·第 3 次】跨工具对比 worldSeed 错位——烧掉整条「Octave createLegacy 缺口」结论链

### 现象
- BIOME6（Java yarn NoiseRouter 直采，跑在 run/server world，seed=**-2032795982907864146**）vs Rust multiworld_nether_blocks 探针（seed=**-8248318472910187742** 参照 seed）对比 6 维 climate 值（@ mismatch 同坐标）：
  - 「temperature 符号相反」：Java **+0.0775** vs Rust **-0.115**
  - 「humidity 完全错」：Java **-0.1533** vs Rust **≈0**
- 由此推导出的结论链：①「legacy climate visitor 是唯一对齐路径」②「OctavePerlin createLegacy 存在数值缺口」（M10 定论）③「v7 净负 = visitor 替换连锁」——**部分建立在错误对比上**（两侧根本不是同一个世界）。

### 根因
**跨工具对比用了不同 worldSeed**。派生噪声（`randomDeriver.split(temperature/vegetation/...)`）与 worldSeed 相关 → seed 不同则派生噪声值完全不同 → 「t 符号相反 / h 归零」全是 seed 错位假象，与 Rust 实现无关。固定种子特例（CheckedRandom(0)/(2)）不受 worldSeed 影响，所以 M9/M10 中「特例态符号一致」的结论碰巧成立——**混合正确与错误证据导致定位被带偏多个循环**（M10 四层对拍本身没问题，但对比基准侧的 seed 就是错的）。

### 定位（怎么发现的）
- Java 侧 ShiftedNoise 递归树 dump（BIOME6 探针反射，参数类型 NoisePos 匹配）：shiftX/Y/Z 全恒 0（OFFSET 特例生效 ✓）+ `noise.sample(3,0,0)=0.0919` 是派生值 → 逐层核对派生输入时发现两侧 worldSeed 不一致（-2032795982907864146 vs -8248318472910187742）。
- 诊断路径仍是「顺着采样链逐输入核对」——与 M10 分段对拍同思路，只是核对对象从噪声构造换到派生输入；核对到 worldSeed 这一层才暴露错位。

### 修复
- Rust 探针加 **`WG_SEED` env**（默认保持参照 seed，不改历史行为）→ 同 seed（-2032795982907864146）重比：
  - t：Java +0.0775 / Rust **+0.171**（残差 0.094，**符号一致**）
  - h：Java -0.1533 / Rust **-0.187**（残差 0.034，**量级一致**）
- 修正后的格局：
  - vegetation 采样链基本对齐（h 残差 0.034 在量级一致范围）；
  - temperature 残差 0.094 待查（候选：noise_params 表 vs 注册表 / shift_a 细节）；
  - soul_sand 残差定性重估：同 seed 下两边最近邻都判 nether_wastes → vanilla soul_sand 块**可能来自 features 层**而非表面/biome 层（新方向）。

### 教训（⚠️ 重点标记：复发错误——seed/坐标错位类第 3 次）
- **这是 seed/坐标错位类错误第 3 次复发**：
  1. ① #23/#24 采样坐标错位（-337 vs -336/-340，误判「湿度差 0.0054」）；
  2. ② 参照 seed 与 server.properties level-seed 混淆；
  3. ③ 本次跨工具 worldSeed 错位（Java 探针跑 server world / Rust 探针用参照 seed，两套 seed 无人核对）。
- **铁律早已存在**（AGENTS「探针/参照采集核对铁律」seed 三查）**但没被套用到「Java↔Rust 跨工具数值对比」场景**——三查约束的是参照导出流程，没约束「对比动作本身」。修正：**任何 Java↔Rust 数值对比的第一动作 = 核对两侧 worldSeed 一致**（在对比之前，不是得出怪结论之后）。
- **可复用判据**：跨工具对比出现「符号翻转 / 量级级（非残差级）差异」时，**先怀疑 seed/坐标错位，再怀疑实现**——seed 错位产生的差异恰好长得像实现 bug（符号反、量级差），与残差级偏差（0.03~0.09）完全不同族。
- **对比工具链应强制 seed 自检**：探针输出行自带 seed 并由对比脚本核对——人眼核对不可靠，已三犯。

### 遗留（被 seed 错位污染的旧结论逐条复核）
1. 「t 符号相反」（M10 现象）——**已反转**（同 seed 符号一致）。
2. 「OctavePerlin createLegacy 数值缺口」（M10 定论）——S3 对拍是**同 seed 固定种子**（CheckedRandom(0)）下做的，**仍成立**；但「湿度缺口」的定性需按新残差（h 0.034 量级一致）重估。
3. 「legacy climate visitor 净负」（M9）的连锁解读——需按修正后格局重审（正确的部分哪些来自 seed 假象、哪些真实）。
4. temperature 残差 0.094 具体差异（noise_params 表 vs 注册表 / shift_a 细节）——未查。
5. soul_sand 块来自 features 层假设——未验证。

---

## 速查表追加 1 行（插表末）

| Java t 符号相反 / h≈0，推导出「Octave createLegacy 缺口」结论链，后被同 seed 重比推翻（M11【复发·第3次】） | 跨工具对比用了不同 worldSeed（Java 探针跑 server world -2032795982907864146，Rust 探针用参照 seed -8248318472910187742）；派生噪声与 worldSeed 相关 → seed 不同则派生值全不同，seed 错位假象长得像实现 bug；固定种子特例不受影响 → 混合正确与错误证据带偏定位多个循环 | **seed/坐标错位类错误第 3 次复发**——铁律（seed 三查）存在但没套到「Java↔Rust 跨工具数值对比」场景。修正：**任何跨工具数值对比的第一动作 = 核对两侧 worldSeed 一致**（对比前，非得出怪结论后）；判据：出现**符号翻转/量级级差异**先怀疑 seed/坐标错位再怀疑实现；对比工具链强制 seed 自检（探针输出自带 seed + 脚本核对，人眼核对已三犯不可靠） |
