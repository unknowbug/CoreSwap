# 双跑修复设计 — wg_set_flags 句柄级显式 flag（2026-09-08）

## 背景
存档链路 = Rust 管线（含 carver+features）× Java 分步拦截（mixin 只拦 populateNoise + cancel buildSurface）→ **双重 feature 应用**（judge C2-P2 confirmed，消融 +5508）。
现有开关 `WG_SKIP_CARVER`/`WG_SKIP_FEATURES` 是**进程全局 env**（worldgen_handle.rs L438/443）——翻转默认会破坏 block_probe/standalone FULL 口径与所有 bin-diag 工具。judge CONCERN 在案：需句柄/调用级显式 flag。

## 选型
| 方案 | 判定 |
|---|---|
| A. wg_create 加参数 | ❌ 5 参签名被 C++ worldgen_api.h + JNI 两侧对齐锁死（jni_bridge.rs 注释明示「对齐 C++ Java_wg_CppWorldgen_init：wg_create 5 参」），改签名破坏 FFI 契约 |
| B. **新导出 `wg_set_flags(handle, mask)`** | ✅ 最小侵入；旧调用方零改动默认行为不变 |

## 设计
### Rust 侧
- `WorldgenHandle` 加 `pub flags: std::sync::atomic::AtomicU32`（create 时置 0 = 现状行为）。
- 语义：**flag 位设置时覆盖 env 判定；未设置（0）回落 env**——bin-diag/probe 工具零改动。
  - bit0 `SKIP_CARVER`、bit1 `SKIP_FEATURES`、bit2 `SKIP_SURFACE`（对称预留，暂无调用方）。
- `fill_chunk_blocks` 判定改为：`flags & bit != 0 || env.is_ok()`（skip 方向 OR，二者任一即 skip）。
- api.rs 新导出：`wg_set_flags(handle, mask: c_int)` + `wg_get_flags(handle) -> c_int`（诊断用）。

### Java 调用侧
- `CppWorldgen` 加 native `setFlags(long handle, int mask)`（jni_bridge.rs 加 `Java_wg_CppWorldgen_setFlags`）。
- `CppBridge.init / initNether` 创建句柄后设 flag：mask 由系统属性 `-Dcoreswap.rust.stages` 控制，**默认 = skip carver+features（mask=0b011）**——理由：服务端存档链路 carver/features 由 Java vanilla 执行（mixin 拦截面决定），Rust 双跑即 confirmed 缺陷；`coreswap.rust.stages=all` 可回旧行为（对照用）。
- standalone block_probe 路径不走 CppBridge（ctypes/独立 probe），不受影响。

### 兼容性
- env 门保留（诊断/消融实验仍可用）；flag 与 env 是 OR 关系，无新冲突面。
- 旧 jar + 新 dll：无 setFlags 调用 → flags=0 → 行为与现状完全一致。

## 回归判据（Phase 1c）
- 删 run/world → seed B（1DDE3B09 nether FULL 参照 `.tmp/ref-full/`）→ initNether + mixin 存档链路回归。
- 通过线：存档口径 **≥94.42%**（=SKIP_FEATURES 消融上界），≥3 采样区间一致；数字带 seed+region+口径三要素。

## 风险
- overworld 句柄（init）同样默认 skip → overworld 存档行为改变（清单 #3 量化课题正好需要此通道；回归先盯 nether 判据，overworld 消融后续量化）。
- 若 nether 存档回归 <94.42%：回数据层对照消融基线（+5508 已知），不空转 3 轮。
