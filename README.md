# CoreSwap — 一边逆向一边编程的混合工程框架

> **「任何声称都必须有可验证的实践锚点。」** — Anchorlaw 第一律

CoreSwap 是 Minecraft Java worldgen 的 C++ 重写项目（逐位对齐验证），同时是一个**混合工程框架**：
逆向（Java 源码/javap → 还原 C++）+ 编程（C++ 实现 + 逐位对比验证）一体化。

## 框架组成

```
E:\PYTHON\CoreSwap\
├── AGENTS.md                          ← 项目工作规则（铁律 + 协议 + 混合工作流）
├── README.md                          ← 本文件
├── protocol/
│   └── verification-protocol.md       ← 验证协议（Anchorlaw + RE 方法论 + CoreSwap 定制，跨版本通用）
├── scripts/
│   └── scan_cpp_anchors.py           ← C++ @anchor 扫描工具（source 校验 + 汇总，跨版本通用）
└── versions/                          ← 按大版本组织（多版本引擎；后续 1.18/1.19 各占一目录）
    └── 1.20.1/
        ├── cpp/                       ← C++ 工程（CMakeLists + worldgen/src，带 @anchor 注解）
        │   └── worldgen/src/
        │       ├── density.h          ← 密度函数引擎（插值/缓存，6 锚点）
        │       ├── aquifer.h          ← 含水层判定（4 锚点）
        │       ├── surface.h          ← 表面规则（SurfaceCondC，2 锚点）
        │       └── ...                ← 其余源码（未标注，增量推进）
        ├── data/                      ← worldgen JSON + 参照 blocks（该版本）
        └── docs/                      ← 知识库（01-09 主题 + 时间线）
```

## 来源融合

| 来源 | 引入内容 | 状态 |
|---|---|---|
| [Anchorlaw v0.4](E:\PYTHON\Anchorlaw) | @anchor.test/@anchor.idk + source 溯源 + noise cards + 验证分层 + retry cap + 第三律挑战 | ✅ 协议落地 |
| [RE-Framework](E:\PYTHON\RE-Framework) | 置信度状态机（confirmed 用户拍板）+ Phase 0 轻量计划 + Scout/Worker/Judge + Lift 原则（禁信反编译） | ✅ 选择性吸收 |
| CoreSwap 工程实践 | 知识库链条铁律 + 验证载体（block_probe 等）+ worldgen.dll 对齐 + 差块分类 | ✅ 已有 |

**明确不引入**：Lift 汇编流程、.artifacts YAML 体系、多 Agent 目录骨架（CoreSwap 是 Java 源码级逆向，无汇编场景；docs 已是成熟知识库，引入=双轨制负担）。

## 工作区关系

- **CoreSwap（本目录）= 唯一主工作区**：代码 + CMake 构建 + 协议 + 工具 + 知识库 + 参照数据全部自洽。
- **Anchorlaw 仓库 = 协议来源**，禁止反向修改。
- 历史参考（Java 探针工程等）只读引用，改动以 CoreSwap 为准。

## 快速开始

```powershell
# 扫描 C++ 注解（要求 PYTHONPATH 指向 Anchorlaw scanner）
set PYTHONPATH=E:\PYTHON\Anchorlaw\python\anchorlaw-scanner
python scripts\scan_cpp_anchors.py versions\1.20.1\cpp\worldgen\src

# 读协议
# protocol/verification-protocol.md
```

## 状态（2026-08-08）

- 注解：density.h（6）+ aquifer.h（4）+ surface.h（2）= 12 anchors 全部 valid（11 test + 1 idk）
- 核心对齐：正坐标 100%（8576/3200 区域）、负坐标 -288 确认 **非 density bug**（= 结构/FEATURE 假 diff）
- 已知边界（idk）：结构 Beardifier 密度修正未实现（结构附近 density 差 ~0.12）
