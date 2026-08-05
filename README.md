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

## Status (as of 2026-08-05)

**POC milestone — density field 100% + block layer first run.** Not yet a usable product.

- ✅ C++ density field is **100% bit-identical to vanilla** (12288/12288 sample points, maxErr=0, exact IEEE double)
- ✅ **2.43× speedup** on density evaluation (C++ 4.42 ms/chunk vs Java 10.75 ms/chunk, -O2 baseline)
- ✅ **JNI bridge** (`worldgen.dll`): `wg_create` / `wg_fill_density` / `wg_fill_blocks` — big-block data exchange, verified 100% identical over JNI
- ✅ **Block layer** (density → aquifer → surface rules → ore veins): **98.71% all-block / 96.07% non-air match** vs vanilla SURFACE state (fixed seed, 4×4 chunk region x=3200..3223, z=3208..3231)
- ⚠️ Current release (`1.20.1-poc`) contains **verification tools only** (probes + `worldgen.dll` JNI stub) — there is no installable mod / server plugin yet

📄 POC details: [`versions/1.20.1/bench/report.md`](versions/1.20.1/bench/report.md)

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
