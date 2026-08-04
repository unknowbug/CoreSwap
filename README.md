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

Adding a new MC version = adding a directory.

## Current Status (1.20.1)

- ✅ **Density field 100% bit-identical to vanilla** (12288/12288 sample points, maxErr=0, exact IEEE double)
- ✅ **2.43× speedup** (C++ 4.42 ms/chunk vs Java 10.75 ms/chunk, -O2 baseline, no memory optimizations yet)
- 📄 Details: [`versions/1.20.1/bench/report.md`](versions/1.20.1/bench/report.md)

## Roadmap

1. **Memory optimization**: compact arrays + indexing + cache-friendly layouts (projected 2-5× more)
2. **JNI bridge**: `generateRegion` — bulk data exchange, one call
3. **Block layer**: density → block states (surface rules + chunk fill)
4. **Entity AI / pathfinding**: second core to C++-ify (community precedent: JNI-accelerated pathfinding)

## Prebuilt Binaries

No compiler or JDK needed — download and run on Windows x64:

[**Download CoreSwap-1.20.1-poc.zip**](https://github.com/unknowbug/CoreSwap/releases) (1.6 MB)

Includes `density_probe.exe` / `noise_probe.exe` / `router_probe.exe` / `worldgen.dll` +
vanilla reference density + worldgen JSON data. Quick verify:

```
density_probe.exe -8248318472910187742 vanilla_reference.density worldgen-data
# expect: match=12288/12288 (100.0000%) maxErr=0
```

Built with statically-linked MinGW runtime — the binaries are self-contained.

## Getting Started

### Prerequisites

- **Windows** (currently the target platform)
- **CMake**
- Toolchains (portable zips, not committed to the repo) under a `tools/` dir at the repo root:

```
tools/
├── mingw/mingw64/bin/        # MinGW-w64 (gcc 16.x) — needed for the C++ core + JNI DLL
└── jdk17/jdk-17.0.20+8/      # Temurin JDK 17 — Loom 1.20.1 toolchain (JDK 24 is too new)
```

### Build

```powershell
powershell -File versions\1.20.1\build.ps1
```

Builds the C++ core + `worldgen.dll` (JNI), compiles the Java JNI test, and runs it (expect a `seed=... => <hash>` line).

### Verify the C++ core against vanilla

1. **Extract vanilla worldgen data** (needed by `density_probe`):

```powershell
# from the 1.20.1 minecraft jar (client or server)
jar xf minecraft-1.20.1.jar data/minecraft/worldgen
# put the resulting data/ under e.g. versions\1.20.1\data\worldgen
```

2. **Generate the vanilla density reference** (starts a dedicated server via Loom, generates chunks, exports density samples):

```powershell
cd versions\1.20.1\java
# JAVA_HOME must point to JDK 17
gradle runServer -PbenchSeed=-8248318472910187742 -PbenchSize=4 -PbenchOriginX=200 -PbenchOriginZ=200
# → writes data/vanilla_<seed>_<size>.density + .json (big-endian doubles, see bench/report.md for format)
```

3. **Compare C++ against it**:

```powershell
cd versions\1.20.1
cpp\build\density_probe.exe -8248318472910187742 data\vanilla_-8248318472910187742_4.density data\worldgen
# expect: match=12288/12288 (100.0000%) maxErr=0
```

4. **Noise primitive probe** (C++ side, 54 noise keys × N points):

```powershell
cpp\build\noise_probe.exe <seed> <count>
# compare with: gradle runServer -PprobeCount=<count>  (outputs the Java reference)
```

### Probes (Java side, via Loom)

| Mode | Command | Output |
|---|---|---|
| Noise reference | `gradle runServer -PprobeCount=64` | 54 noise keys × 64 points |
| Router components | `gradle runServer -ProuterProbe=true` | all router components + density timing |
| Chunk baseline | `gradle runServer -PbenchSeed=<s> -PbenchSize=<n> -PbenchOriginX=<x> -PbenchOriginZ=<z>` | chunk gen timing + density file |

The Java probes need `eula.txt` (`eula=true`) in `versions/1.20.1/java/run/`.

## How It Works

The C++ core reconstructs the density field exactly as vanilla does:

- **Noise primitives**: Xoroshiro128PlusPlus RNG, MD5-based seed derivation, Perlin / octave / double-perlin samplers — bit-identical to Mojang's implementation
- **Density function tree**: assembled at runtime from vanilla's `worldgen` JSON (`noise_settings/overworld.json` + `density_function/overworld/*.json`), mirroring `NoiseConfig`'s visitor semantics
- **InterpolatedNoiseSampler** (`old_blended_noise`): the terrain backbone, reproduced exactly

No tolerance was needed: the C++ density field matches vanilla to the exact IEEE double.

## License

MIT
