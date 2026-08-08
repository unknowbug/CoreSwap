# CoreSwap

> **We took the 'Java' out of Minecraft Java Edition. Same mods. Same worlds. Different FPS.**

[中文版 / Chinese](./README.zh-CN.md)

Rewrite Minecraft Java Edition's performance-critical cores — **world generation** (and eventually entity AI / pathfinding) — in C++, while keeping the **full Java mod ecosystem** intact. Same seed. Same world. Same mods. The C++ goes underneath.

**Why:** Java Edition's performance has been a meme for two decades. Every existing fix has a fatal flaw:

| Approach | Flaw |
|---|---|
| Paper & optimization plugins | Still Java — treats symptoms, not causes |
| Cuberite (full C++ rewrite) | Fast, but the mod ecosystem dies |
| Switch to Bedrock | Ecosystem gone, version drift forever |

CoreSwap walks the path nobody has walked: **C++ performance core + Java mod layer (JNI bridge)** — the mod API stays Java and untouched; everything below it is free to become C++.

## Status (as of 2026-08-08)

**Latest: v1.0.18 (pre-release)** — installable Fabric mod for MC 1.20.1. 连续修复：heightmap 索引、并发崩溃、原生崩溃日志 handler（异常+调用栈+crash 文件）、完整调用栈 + dll sha256 诊断、内存损坏诊断、**VEH 崩溃日志 handler 与 JVM 兼容（不再干扰 JIT/GC，JVM 进程内自动降级）**。兼容 Forge（Sinytra Connector）。

- ✅ NOISE+SURFACE (density / aquifer / ore veins / surface rules) **bit-identical to vanilla** — same seed, same terrain, block-for-block (3200 区域 100%，玩家 seed 8576 区域 99.9768%+，剩余为 terracotta 带边缘排查中)
- ✅ **10-20× faster worldgen**: batched parallel generation (~3 ms/chunk vs ~60 ms vanilla), adaptive `min(cores, tasks)` threading
- ✅ All pure-algorithm optimizations **lossless** (FlatCache / Cache2D / spline caching) — no approximation
- ✅ **Pairs with Sodium/Iris**: Sodium owns rendering (FPS), CoreSwap owns generation (chunk loading) — complementary, no conflict. Tested: RTX 4060 laptop + BSL shaders + max render distance, zero stutter
- 📦 Download: [CoreSwap 1.20.1 v1.0.18](https://github.com/unknowbug/CoreSwap/releases)
- 🗺️ Version plan: **full coverage on the roadmap** — 1.20.x ships first, then 1.18/1.19 and 1.17- progressively (worldgen architecture differs per version)
- 🔭 Roadmap: LIGHT stage, entity AI (Brain / Goal / Pathfinding) in C++

## Installation

### Requirements

- **Minecraft 1.20.1** (Java Edition)
- **Fabric Loader 0.15.x** — if you don't have Fabric yet, install it with the [Fabric installer](https://fabricmc.net/use/) (select MC 1.20.1, click Install)
- **Java 17** — Fabric Loader 0.15 requires Java 17+

### Steps

1. **Download** the latest `coreswap-1.20.1-*.jar` from [Releases](https://github.com/unknowbug/CoreSwap/releases)
2. **Install Fabric** (skip if already installed): run the Fabric installer, pick Minecraft **1.20.1**, Install. It creates a "fabric-loader-…" profile in the launcher
3. **Open the mods folder**: in the Fabric launcher profile click **Open Mods Folder**, or navigate manually to:
   - Windows: `%appdata%\.minecraft\mods`
   - macOS: `~/Library/Application Support/minecraft/mods`
   - Linux: `~/.minecraft/mods`
4. **Drop the CoreSwap jar into `mods/`** — done
5. **(Recommended) Add Sodium + Iris** (from [Modrinth](https://modrinth.com/)) — Sodium 0.5.x and Iris 1.7.x for 1.20.1, drop into `mods/` too. **Sodium owns rendering (FPS), CoreSwap owns generation (chunk loading) — they complement each other, no conflict.** Add a shaderpack (e.g. BSL, Complementary) via `Options → Video Settings → Shader Packs` if you want shaders
6. **Launch** the Fabric profile. Verify it's active in `logs/latest.log`:
   ```
   [BenchMod] CoreSwap replace mode: C++ worldgen active
   ```

### Notes

- **Server**: works on dedicated Fabric servers too — put the same jar in the server's `mods/` folder
- **Forge**: supported via [Sinytra Connector](https://modrinth.com/mod/connector)
- **FEATURES stage** (ores / decoration) is still vanilla — **NOISE+SURFACE** is bit-identical to vanilla
- If you don't see the log lines above: check the jar is in `mods/`, MC is 1.20.1, Fabric Loader is 0.15.x, and Java is 17

## Versioning

The repo is organized by **Minecraft Java version number**. Each version lives in its own directory:

```
CoreSwap/
├── README.md
└── versions/
    ├── 1.20.1/          # ← current
    │   ├── cpp/         # C++ core (noise + density field + surface rules)
    │   └── data/        # worldgen JSON + reference block data (for verification)
    └── <future versions>/
```

> **Note**: `data/` is **not** distributed in the repo (it contains worldgen JSON exported from vanilla + reference block dumps; see "Building from source" below for how to obtain it). The C++ code compiles without it; the verification tools (`block_probe` etc.) need it at runtime.

## Building from source

The C++ core is **MSVC-only** (MinGW is not supported — `thread_local` semantics break under MinGW's static linking, corrupting the shared caches). CI-like requirements:

- **Visual Studio 2022+** (MSVC C++ toolset, x64)
- **JDK 17+** (only for the JNI bridge headers — `jni.h`; set `JAVA_HOME`)
- **CMake 3.20+** and **Ninja**

Build steps (Windows, from a **Developer PowerShell / cmd with vcvars64** loaded):

```bat
:: load the MSVC environment (VS 2022 example)
call "C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvars64.bat"
:: ninja must be on PATH (VS ships it under Common7\IDE\CommonExtensions\Microsoft\CMake\Ninja)

cd versions\1.20.1\cpp
cmake -G Ninja -DCMAKE_BUILD_TYPE=Release -S . -B build-msvc
cmake --build build-msvc
```

Outputs land in `build-msvc\bin\` (`block_probe.exe`, `worldgen.dll`, etc.).

**To run the verification tools** you also need the worldgen data directory (`versions/1.20.1/data/worldgen` — vanilla's `worldgen` JSON tree) and `blocks.json` + reference `.blocks` dumps. These are exported from a vanilla 1.20.1 server/client (the `data/minecraft/worldgen` folder inside the jar) and regenerated per-seed with the probe tools; they are intentionally kept out of the repo. Contact the maintainers if you need a copy.

## How It Works

The C++ core reconstructs the density field exactly as vanilla does:

- **Noise primitives**: Xoroshiro128PlusPlus RNG, MD5-based seed derivation, Perlin / octave / double-perlin samplers — bit-identical to Mojang's implementation
- **Density function tree**: assembled at runtime from vanilla's `worldgen` JSON (`noise_settings/overworld.json` + `density_function/overworld/*.json`), mirroring `NoiseConfig`'s visitor semantics
- **InterpolatedNoiseSampler** (`old_blended_noise`): the terrain backbone, reproduced exactly

No tolerance was needed: the C++ density field matches vanilla to the exact IEEE double.

## Roadmap

1. ✅ **JNI bridge**: bulk chunk data exchange
2. ✅ **Block layer**: density → block states (surface rules + chunk fill)
3. ✅ **Integration**: installable Fabric mod / server plugin
4. **Memory optimization**: compact arrays + indexing + cache-friendly layouts (projected 2-5× more)
5. **Entity AI / pathfinding**: second core to C++-ify (community precedent: JNI-accelerated pathfinding)

## Credits

- **dustinmoon78** — Forge + Sinytra Connector compatibility: multi-level mod jar resolution (`CoreSwapFixHelper`) + direct `JarFile` extraction, tested on 400+ mod packs. See [#3](https://github.com/unknowbug/CoreSwap/pull/3).

## License

MIT
