# G3 漂移基底归因课题错误台账（260917-01）

## E1：settle 探针漏传 -PlightRust → 两臂 VOID（Vanilla 光照臂冒充接管臂）

- **现象**：G17-L60 / G17-D60 两臂自证行 lightInit_ok=0、domain hook=0、fallback=0；log 中零 [LightRust] 行。
- **根因**：光照接管的开关 = `-PlightRust`（build.gradle:103 → `-Dcoreswap.light.rust=1`，mixin 头部 `LIGHT_RUST = System.getProperty("coreswap.light.rust") != null`）。NEXT_SESSION 复测口径只写了「域臂开关 -Pdomainbatch=1」，**光照总开关未被转录**——上轮 run_g3_cp1 的 L3/D3c 实际带 -PlightRust（record 自证 lightInit ok ×2 可反推），口径交接时该前置项失传（#90 转抄漂移家族 / #8 -P→-D 映射遗漏家族的**复测口径失传**形态）。
- **定位**：SELFCERT 硬门（#118）lightInit_ok=0 → 逐层查：G17 日志原文 → 上轮 CP1-L3.log 对照（有 lightInit ok）→ 一手源码 ServerLightingProviderMixin.java:75 + build.gradle:103。
- **修复**：G17 两臂判 VOID 不挑臂（预登记自证门生效）；换标签 G17b-* 重跑，显式补 -PlightRust。
- **教训**：① 复测口径必须列**全部**使能开关，不只有「本块新增变量」；② 跨臂对比前先核两臂执行体形态一致（[LightRust] 行为形态自证），否则「塌缩」读数是执行体差异假象（#65/#66 家族）；③ 附带收益：G17 两臂（同 vanilla 形态 60s）互差 17/16 chunk ≈0.8%，可作 run 级噪声锚旁证（口径：vanilla 光照、60s settle）。

## E2：C-B2 离线对拍探针的 sections_diff 细节逻辑失效（非数据矛盾）

- **现象**：probe_cb2_offline.py 对 28 个不等 chunk 输出 sections_diff=[] / n_sections_diff=0（worker 曾标为「未裁决矛盾」）。
- **根因**：快照值是**单一 sha256[:16] 哈希字符串**（snap_light.py 每 chunk 一个值），探针细节逻辑按「等长列表逐元素」处理 → 恒空。属对拍脚本细节缺陷，非 section 集结构差。
- **定位**：主会话直读 snap_light.py 源码 + json 值结构（str）确认。
- **修复**：worker 草稿中该「矛盾」降级为脚本细节缺陷说明；哈希级不等结论不受影响（判据主体是 chunk 级哈希对比）。
- **教训**：对拍细节层的分解逻辑必须与载体值结构对表（build-tooling #59/#53 同族：解析前核值形态）。
