# CoreSwap

> **We took the 'Java' out of Minecraft Java Edition. Same mods. Same worlds. Different FPS.**

[中文版 / Chinese](./README.zh-CN.md)

Rewrite Minecraft Java Edition's performance-critical cores — **world generation** and **entity AI / pathfinding** — in C++, while keeping the **full Java mod ecosystem** intact. Same seed. Same world. Same mods. The C++ goes underneath.

**Why:** Java Edition's performance has been a meme for two decades. Every existing fix has a fatal flaw:

| Approach | Flaw |
|---|---|
| Paper & optimization plugins | Still Java — treats symptoms, not causes |
| Cuberite (full C++ rewrite) | Fast, but the mod ecosystem dies |
| Switch to Bedrock | Ecosystem gone, version drift forever |

CoreSwap walks the path nobody has walked: **C++ performance core + Java mod layer (JNI bridge)** — the mod API stays Java and untouched; everything below it is free to become C++.

## Status (as of 2026-08-06)

**v1.0.0 released — installable Fabric mod for MC 1.20.1.**

- ✅ NOISE+SURFACE (density / aquifer / ore veins / surface rules) **100% bit-identical to vanilla** — same seed, same terrain, block-for-block
- ✅ **10-20× faster worldgen**: batched parallel generation (~3 ms/chunk vs ~60 ms vanilla), adaptive `min(cores, tasks)` threading
- ✅ All pure-algorithm optimizations **lossless** (FlatCache / Cache2D / spline caching) — no approximation
- ✅ **Pairs with Sodium/Iris**: Sodium owns rendering (FPS), CoreSwap owns generation (chunk loading) — complementary, no conflict. Tested: RTX 4060 laptop + BSL shaders + max render distance, zero stutter
- 📦 Download: [CoreSwap 1.20.1 v1.0.0](https://github.com/unknowbug/CoreSwap/releases/tag/coreswap-1.20.1-1.0.0)
- 🗺️ Version plan: **full coverage on the roadmap** — 1.20.x ships first, then 1.18/1.19 and 1.17- progressively (worldgen architecture differs per version, docs/08 covers the migration flow)
- 🔭 Roadmap: LIGHT stage, entity AI (Brain / Goal / Pathfinding) in C++

📄 Tech knowledge base: [`docs/`](docs/README.md) (8 articles, in Chinese — architecture / random / density / aquifer / ore veins / surface / pipeline / version migration)

## Installation

### Requirements

- **Minecraft 1.20.1** (Java Edition)
- **Fabric Loader 0.15.x** — if you don't have Fabric yet, install it with the [Fabric installer](https://fabricmc.net/use/) (select MC 1.20.1, click Install)
- **Java 17** — Fabric Loader 0.15 requires Java 17+

### Steps

1. **Download** `coreswap-1.20.1-1.0.0.jar` from [Releases](https://github.com/unknowbug/CoreSwap/releases)
2. **Install Fabric** (skip if already installed): run the Fabric installer, pick Minecraft **1.20.1**, Install. It creates a "fabric-loader-…" profile in the launcher
3. **Open the mods folder**: in the Fabric launcher profile click **Open Mods Folder**, or navigate manually to:
   - Windows: `%appdata%\.minecraft\mods`
   - macOS: `~/Library/Application Support/minecraft/mods`
   - Linux: `~/.minecraft/mods`
4. **Drop `coreswap-1.20.1-1.0.0.jar` into `mods/`** — done
5. **(Recommended) Add Sodium + Iris** (from [Modrinth](https://modrinth.com/)) — Sodium 0.5.x and Iris 1.7.x for 1.20.1, drop into `mods/` too. **Sodium owns rendering (FPS), CoreSwap owns generation (chunk loading) — they complement each other, no conflict.** Add a shaderpack (e.g. BSL, Complementary) via `Options → Video Settings → Shader Packs` if you want shaders
6. **Launch** the Fabric profile. Verify it's active in `logs/latest.log`:
   ```
   [BenchMod] CoreSwap replace mode: C++ worldgen active
   [Mixin] populateNoise intercepted chunk(...)
   ```

### Notes

- **Server**: works on dedicated Fabric servers too — put the same jar in the server's `mods/` folder
- **Full version coverage planned** — 1.20.x first, then 1.18/1.19 and 1.17- progressively
- **FEATURES stage** (ores / decoration) is still vanilla — **NOISE+SURFACE** is bit-identical to vanilla
- If you don't see the log lines above: check the jar is in `mods/`, MC is 1.20.1, Fabric Loader is 0.15.x, and Java is 17

## Versioning

The repo is organized by **Minecraft Java version number**. Each version lives in its own directory:

```
CoreSwap/
├── README.md
└── versions/
    ├── 1.20.1/          # ← current (frozen modern version, untouched by Mojang's Vulkan migration)
    │   ├── cpp/         # C++ core (noise + density field + JSON assembly)
    │   ├── java/        # Fabric Loom dev env (vanilla baseline + probes)
    │   ├── bench/       # comparison scripts + POC report
    │   └── build.ps1    # build script
    └── <future versions>/
```

## Roadmap

1. **JNI bridge**: `generateRegion` — bulk data exchange, one call
2. **Block layer**: density → block states (surface rules + chunk fill)
3. **Integration**: installable Fabric mod / server plugin (the point where users can actually use this)
4. **Memory optimization**: compact arrays + indexing + cache-friendly layouts (projected 2-5× more)
5. **Entity AI / pathfinding**: second core to C++-ify (community precedent: JNI-accelerated pathfinding)

## How It Works

The C++ core reconstructs the density field exactly as vanilla does:

- **Noise primitives**: Xoroshiro128PlusPlus RNG, MD5-based seed derivation, Perlin / octave / double-perlin samplers — bit-identical to Mojang's implementation
- **Density function tree**: assembled at runtime from vanilla's `worldgen` JSON (`noise_settings/overworld.json` + `density_function/overworld/*.json`), mirroring `NoiseConfig`'s visitor semantics
- **InterpolatedNoiseSampler** (`old_blended_noise`): the terrain backbone, reproduced exactly

No tolerance was needed: the C++ density field matches vanilla to the exact IEEE double.

## License

MIT
