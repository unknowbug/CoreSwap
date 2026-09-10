# CoreSwap

> **We took the 'Java' out of Minecraft Java Edition. Same mods. Same worlds. Different FPS.**

[中文版 / Chinese](./README.zh-CN.md)

Rewrite Minecraft Java Edition's performance-critical cores — **world generation** (and eventually entity AI / pathfinding) — in native code, while keeping the **full Java mod ecosystem** intact. Same seed. Same world. Same mods. The native core goes underneath.

**Why:** Java Edition's performance has been a meme for two decades. Every existing fix has a fatal flaw:

| Approach | Flaw |
|---|---|
| Paper & optimization plugins | Still Java — treats symptoms, not causes |
| Cuberite (full C++ rewrite) | Fast, but the mod ecosystem dies |
| Switch to Bedrock | Ecosystem gone, version drift forever |

CoreSwap walks the path nobody has walked: **native performance core + Java mod layer (JNI bridge)** — the mod API stays Java and untouched; everything below it is free to go native.

## Project Adjustment (2026-08-30): Core rewritten in Rust

The worldgen core has **migrated from C++ to Rust**. One `worldgen.dll` now ships everything — the JNI bridge (`Java_wg_CppWorldgen_*`) and the engine (`wg_*` C ABI) in a single Rust cdylib. The C++ line is archived (historical reference only); all active development happens in [`worldgen-core/`](./worldgen-core) + the per-version thin shells under [`versions/`](./versions).

**Why Rust:** one language for bridge + engine (no second toolchain), memory safety in a hot multi-threaded path, and a build-time **density-function transpiler** (vanilla JSON → specialized native code) that doubles as a correctness oracle — the transpiled pipeline is proven equivalent to the runtime interpreter to floating-point residual (<5e-7), catching semantic bugs invisible to production sampling.

## Status (as of 2026-09-06, v1.0.26)

- ✅ **Overworld fully native**: density → aquifer → ore veins → surface rules → carvers → features. End-to-end **save-path block match ≈ 99.0%** vs vanilla (3-sample avg 99.01%, large-region sweeps; residual is an isolated floating-point edge band around the density zero-crossing, not terrain structure); density fields aligned to floating-point residual (<5e-7). Verified in-game (server + client)
- ✅ **Nether fully native**: end-to-end block match **99.9992%** (two 4×4 regions, 16 mismatched blocks total, all traced to a closed density-edge mechanism); extreme coordinates verified (±30M corners, 98.85–99.85%)
- ✅ **End fully native (new in 1.0.26)**: main island + outer islands via a from-scratch port of vanilla's `EndIslands` density function (SimplexNoiseSampler + world-seed-direct noise chain) and the position-based `TheEndBiomeSource` classifier — **bit-exact vs vanilla: 36/36 chunks, 0 mismatched blocks** in a same-seed production save comparison (Forge dedicated server, features included: end spikes, exit portal, obsidian platform all placed by the vanilla layer on top of the native terrain)
- ✅ **Worldgen performance — faster than vanilla Java**: large-sample end-to-end benchmark (256 chunks, fresh world, stable medians) puts the Rust pipeline at **~28 ms/chunk vs vanilla Java's ~32–33 ms/chunk** — and after the aquifer-estimation rewrite (-63.5% on that stage) real-world chunk loading is **player-verified noticeably faster than vanilla**, on top of full parallelism (adaptive worker pool, shared-column caches). No approximation anywhere — all gains are lossless
- ✅ **Self-contained jar**: the mod jar bundles the complete worldgen dataset (849 files) + the native dll — drop it into `mods/`, no configuration, no external data folder; extraction is version-hashed and self-updating
- ✅ **Startup safety net**: every noise sampler the surface engine can query is validated against the preload table at startup — a missing key fails fast at boot with an exact diagnostic instead of crashing mid-gameplay in a rare biome
- ✅ **Dual loader support — Fabric + Forge**: one jar for both. Fabric native; Forge via [Sinytra Connector](https://modrinth.com/mod/connector) (400+ modpacks tested there). Forge is now **first-class and production-verified**: dedicated Forge 47.4.5 server, production SRG remapped runtime, all three vanilla dimensions taken over and bitwise-verified against vanilla saves on the same seed
- ✅ **Pairs with Sodium/Iris**: Sodium owns rendering (FPS), CoreSwap owns generation (chunk loading) — complementary, no conflict
- 📦 Download: [Releases](https://github.com/unknowbug/CoreSwap/releases) — `1.0.26`
- 🔭 Roadmap: LIGHT stage, entity AI (Brain / Goal / Pathfinding) in Rust

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
5. **(Recommended) Add Sodium + Iris** (from [Modrinth](https://modrinth.com/)) — **Sodium owns rendering (FPS), CoreSwap owns generation (chunk loading) — they complement each other, no conflict.**
6. **Launch** the Fabric profile. Verify it's active in `logs/latest.log`:
   ```
   [BenchMod] CoreSwap replace mode: C++ worldgen active
   [CppBridge] init seed=... enabled=true
   [CppBridge] initNether seed=... enabled=true
   [CppBridge] initEnd seed=... enabled=true
   ```

### Notes

- **Server**: works on dedicated Fabric servers too — put the same jar in the server's `mods/` folder
- **Forge**: fully supported via [Sinytra Connector](https://modrinth.com/mod/connector) — install Forge 47.4.5 + Connector, then drop the same jar into `mods/`. Verified on a production (SRG-remapped) dedicated server across all three vanilla dimensions
- The log line still says "C++ worldgen" for historical reasons — since 1.0.19 the native core is **Rust**
- Overworld, Nether and End are engine-generated; mod dimensions fall through to vanilla

## Versioning

The repo is organized by **Minecraft Java version number**. Each version lives in its own directory:

```
CoreSwap/
├── README.md
├── worldgen-core/             # ← the cross-version Rust worldgen engine (active)
│   └── src/                   # engine: density / aquifer / surface / carver / features / noise / biomes
└── versions/
    ├── 1.20.1/              # ← current
    │   ├── java/            # Fabric mod project (source; run env lives in runtime/)
    │   ├── rust/            # version thin shell → builds worldgen.dll (cdylib)
    │   ├── cpp/             # archived C++ core (historical reference)
    │   ├── data/            # worldgen JSON + reference block data (for verification)
    │   └── docs/            # engineering knowledge base (01-11 topic docs)
    └── <future versions>/
```

The Fabric mod project lives in [`versions/1.20.1/java`](./versions/1.20.1/java) (fabric-loom). Its build syncs the freshly compiled Rust dll into the mod jar automatically. Its **run environment** (worlds/mods/config) lives separately in [`runtime/1.20.1/java/run`](./runtime/1.20.1/java/run) — the project is source, `runtime/` is environment (2026-09-10 split).

## Building from source

**Toolchain (Windows x64):**

- **Rust** (stable, `cargo`) — builds the native core
- **JDK 17** — JNI headers + Fabric/loom builds
- **Gradle 8.x** — mod packaging (fabric-loom 1.10)

```bat
:: 1. build the Rust core + version shell (emits worldgen.dll; build.rs also
::    regenerates the transpiled density code from vanilla JSON)
cargo build --release -p worldgen

:: 2. build the mod (syncs the dll into the jar automatically)
cd versions\1.20.1\java
gradle build
:: jar lands in build\libs\coreswap-1.20.1-*.jar
```

The transpiler inside `build.rs` reads `versions/1.20.1/data/worldgen` (vanilla's worldgen JSON tree) at build time. Verification probes (`worldgen-core/src/bin-diag/*`) additionally need `blocks.json` + reference `.blocks` dumps — exported from a vanilla 1.20.1 server; intentionally kept out of the repo.

## How It Works

The Rust core reconstructs the density field following vanilla's exact semantics:

- **Noise primitives**: Xoroshiro128PlusPlus RNG, MD5-based seed derivation, Perlin / octave / double-perlin samplers — matching Mojang's implementation
- **Density function tree**: loaded at runtime from vanilla's `worldgen` JSON (`noise_settings/<dim>.json` + `density_function/<dim>/*.json`), mirroring `NoiseConfig`'s visitor semantics — **data-driven, no per-dimension code** (interpolation cell sizes, heights, sea level and surface rules all come from the JSON; multi-world native)
- **Build-time transpiler** (`build.rs`): compiles the same JSON into specialized native functions (splines inlined, caches resolved, CSE'd) — a second, independent evaluation path used as a correctness oracle and wired into production behind an env gate
- **Block pipeline**: density → aquifer → ore veins → surface rules → carvers → features, mirroring vanilla stage semantics (including dual noise/world heights for the Nether and the End's simplex-based island density + position-based biome classifier)

## Roadmap

1. ✅ **JNI bridge**: bulk chunk data exchange (now in Rust)
2. ✅ **Block layer**: density → block states (surface rules + chunk fill)
3. ✅ **Integration**: installable Fabric mod / Forge (via Connector) / server
4. ✅ **Multi-world**: Overworld + Nether + End engines with in-game dimension dispatch
5. ✅ **Nether polish**: conversion-surface drift, basalt/blackstone conversion bands and lava-ocean interface closed (99.9992% end-to-end)
6. ✅ **End engine**: EndIslands (SimplexNoise) density function + positional biome classifier — bit-exact vs vanilla (36/36 chunks, 0 diffs)
7. **Entity AI / pathfinding**: second core to nativify

## Credits

- **dustinmoon78** — Forge + Sinytra Connector compatibility: multi-level mod jar resolution (`CoreSwapFixHelper`) + direct `JarFile` extraction, tested on 400+ mod packs. See [#3](https://github.com/unknowbug/CoreSwap/pull/3).

## License

MIT
