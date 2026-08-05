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
- 🗺️ Version plan: 1.20.x series first + next major; 1.18/1.19 on demand; 1.17- not planned (worldgen architecture too different)
- 🔭 Roadmap: LIGHT stage, entity AI (Brain / Goal / Pathfinding) in C++

📄 Tech knowledge base: [`docs/`](docs/README.md) (8 articles, in Chinese — architecture / random / density / aquifer / ore veins / surface / pipeline / version migration)

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
