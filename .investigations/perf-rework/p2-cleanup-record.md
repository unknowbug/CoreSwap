# P2 遗留项清理记录（2026-08-15）

> 承接 G4 A 方案交付（a-plan-ssbo-implementation.md）。judge 审查（有条件通过）列 P2/P3 遗留项，
> 用户拍板「先清 P2 再立 block_probe 集成」。本文件记录 P2 四项清理。

## P2-1: z 采样覆盖（judge 项——原 e2e z=i/1024 恒 0）

- **问题**：e2e 采样坐标 z 恒 0（单列 64×16 网格），spline 4 种 coordType（ridges_folded/ridges/erosion/continents）触发未证实。
- **改法**：`dfc_final_backend_e2e.cpp` 加 `WG_E2E_Z` 环境开关——设置时 z 覆盖 {-2,0,2,4} 四平面（i=0..255→-2, 256..511→0, 512..767→2, 768..1023→4；每 256 组内 x=i%64 4 轮 × y=-64+(i/64%16) 4 层）；默认保持 z=0 基线。
- **验证**：z 覆盖 e2e maxDiff=3.148e-07 / avgDiff=1.290e-08（旧 shader 与 P2 后 shader 一致）——4 种 coordType 在更多 (x,z) 组合下求值正确，正确性保持 ~3.1e-07 量级。
- **意义**：judge「采样覆盖有限」项闭环（z=0 基线 3.128e-07 仍可复现，多 z 平面 3.148e-07 同量级）。

## P2-2: binding 号导出（D19 补全）

- **问题**：judge 指出架构计划 A2 声称「binding 号随生成器产出」，实际 e2e wb 数组硬编码 {0,1,3,4,5,6..11}。
- **改法**：dfc_gen.py 类初始化 `self.spline_bind_base = 6`（spline 6 表 binding 起始号）；gen_cpu 输出 `CpuBackend.splineBindBase`；e2e wb 数组改用 `backend.splineBindBase + k`。
- **注意坑**：spline_layout 是 `_spline_ssbo_glsl`（gen_shader 内）设置的，而 gen_cpu 先于 gen_shader 调用 → 不能从 spline_layout 读 bindBase（AttributeError）→ 提到类初始化共用。
- **验证**：cpu_backend.h L21 `int splineBindBase = 6`；e2e 编译通过、回归一致。

## P2-3: fp64 死代码清理（binding 2 OriginBuf）

- **问题**：judge 发现 binding 2（OriginBuf）从未绑定，但 shader 声明读取——经查 pn_sample5 无调用者（死代码）。
- **死代码链**（grep 全调用者确认）：`pn_sample5`（无调用者）→ `pn_sectionD`（仅 pn_sample5 用）、`gradD/perlinFadeD/lerpD/maintainPrecision`（仅死链用）、`octave_noise_f32`（无调用者）→ 全删。
- **保留**：`GRADIENTS` double 版（活代码 gradDotF 用 float() 读它）、`mapPermD`（pn_sample3_f32/pn_section_f32 用）、fp64 扩展（GRADIENTS double 数组需要）。
- **实际路径**：old_blended 走 `interp_noise`→`pn_section_f32`（float + splitCoord 数据驱动），fp64 链是 F6 时代遗留（GPU fp64 方案被 D17 后 CPU 拆分方案取代）。
- **改法**：dfc_gen.py `_shader_template` else 分支 fp64_funcs/octave_func 置空；主模板 + `_shader_template_alt` 两处 OriginBuf 声明删除。
- **验证**：final_density.comp grep 无 OriginBuf/pn_sample5/pn_sectionD/octave_noise_f32/maintainPrecision/gradD；e2e 基线 3.128e-07 逐位一致 + z 覆盖 3.148e-07。

## P2-4: gen_split_shaders 弃用标注

- **问题**：C 方案（角点级拆 shader）被 G1-G4 实测否定；A 方案改了 `_spline_ssbo_glsl` 输出形态（binding 6-11 SSBO + COORD_SLOT_TABLE），corner 宿主未适配未验证。
- **改法**：gen_split_shaders docstring 加弃用标注（P2-4：启用需同步宿主绑定 + 重新验证）。
- **确认**：该路径 `interp_5 缺失` 断言失败是既有问题（D2 重构后遗留，与本次改动无关——本次只删 OriginBuf/加注释，不涉及 interp 收集）。

## 回归汇总（P2 全部改动后）

| 验证 | 结果 |
|---|---|
| e2e 基线（z=0） | maxDiff=3.128e-07 / avgDiff=1.097e-08（**与 A 方案交付逐位一致**） |
| e2e z 覆盖 | maxDiff=3.148e-07 / avgDiff=1.290e-08 |
| pipeline 编译 | 74.8s / 83.6s（均 <120s 达标） |
| 死代码 | grep 确认全清 |
| binding 导出 | splineBindBase=6 生成器产出，宿主零硬编码 |

## 遗留（P2 后）

- block_probe 集成立项（重量级，下一步，用户批准后实施）。
