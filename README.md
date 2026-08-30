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

The worldgen core has **migrated from C++ to Rust**. One `worldgen.dll` now ships everything — the JNI bridge (`Java_wg_CppWorldgen_*`) and the engine (`wg_*` C ABI) in a single Rust cdylib. The C++ line is archived (historical reference only); all active development happens in [`WorldgenRust/`](./WorldgenRust).

**Why Rust:** one language for bridge + engine (no second toolchain), memory safety in a hot multi-threaded path, and a build-time **density-function transpiler** (vanilla JSON → specialized native code) that doubles as a correctness oracle — the transpiled pipeline is proven equivalent to the runtime interpreter to floating-point residual (<5e-7), catching semantic bugs invisible to production sampling.

## Status (as of 2026-08-30, v1.0.19-beta)

- ✅ **Overworld NOISE+SURFACE in Rust**: density → aquifer → ore veins → surface rules → carvers. Block-level match vs vanilla **95.40%** (baseline engine) / **94.27%** (transpiler engine); density fields aligned to floating-point residual (<5e-7). Zero crashes, **verified in-game** (server + client)
- ✅ **Build-time transpiler**: vanilla `density_function` JSON compiled to native code at build time; consistent with the runtime interpreter across a 98304-point full-chunk sweep (max diff <5e-7), block match 99.30%
- ✅ **Multi-world**: the Nether runs through the same Rust engine (block match **74%** vs vanilla — lava-ocean fluid fill and bedrock-edge layers are known gaps; overworld untouched). End: protection wired, engine not started
- ✅ **Worldgen performance (measured)**: large-sample end-to-end runs measured the Rust pipeline at **~1.2× vanilla Java**; a 16-chunk in-engine JNI sweep (full pipeline incl. carvers + features, adaptive threading) ran at **~14 ms/chunk**. Density internals (dual-height cells, column caches, macro grid) are all lossless — no approximation
- ✅ **Dual loader support — Fabric + Forge**: one jar for both. Fabric native; Forge via [Sinytra Connector](https://modrinth.com/mod/connector) (jar structure verified identical to the Connector-tested 1.0.18; 400+ modpacks tested there)
- ✅ **Pairs with Sodium/Iris**: Sodium owns rendering (FPS), CoreSwap owns generation (chunk loading) — complementary, no conflict
- 📦 Download: [Releases](https://github.com/unknowbug/CoreSwap/releases) — `1.0.19-beta` is a **pre-release (beta)** channel build
- 🔭 Roadmap: Nether fluid fill + bedrock edges, End engine, LIGHT stage, entity AI (Brain / Goal / Pathfinding) in Rust

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
   ```

### Notes

- **Server**: works on dedicated Fabric servers too — put the same jar in the server's `mods/` folder
- **Forge**: supported via [Sinytra Connector](https://modrinth.com/mod/connector)
- The log line still says "C++ worldgen" for historical reasons — since 1.0.19 the native core is **Rust**
- Overworld + Nether are engine-generated; other dimensions fall through to vanilla (End is protected from misrouting)

## Versioning

The repo is organized by **Minecraft Java version number**. Each version lives in its own directory:

```
CoreSwap/
├── README.md
├── WorldgenRust/            # ← the Rust worldgen core (active)
│   ├── src/                 # engine: density / aquifer / surface / carver / features / JNI bridge
│   ├── build/               # build-time transpiler (vanilla JSON → native code)
│   └── rust-dll/            # legacy artifacts (unused)
└── versions/
    ├── 1.20.1/              # ← current
    │   ├── cpp/             # archived C++ core (historical reference)
    │   ├── data/            # worldgen JSON + reference block data (for verification)
    │   └── docs/            # engineering knowledge base (01-11 topic docs)
    └── <future versions>/
```

The Fabric mod project lives in [`runtime/1.20.1/java`](./runtime/1.20.1/java) (fabric-loom). Its build syncs the freshly compiled Rust dll into the mod jar automatically.

## Building from source

**Toolchain (Windows x64):**

- **Rust** (stable, `cargo`) — builds the native core
- **JDK 17** — JNI headers + Fabric/loom builds
- **Gradle 8.x** — mod packaging (fabric-loom 1.10)

```bat
:: 1. build the Rust core (emits WorldgenRust.dll; also regenerates the
::    transpiled density code from vanilla JSON via build.rs)
cd WorldgenRust
cargo build --release

:: 2. build the Fabric mod (syncs the dll into the jar automatically)
cd ..\runtime\1.20.1\java
gradle build
:: jar lands in build\libs\coreswap-1.20.1-*.jar
```

The transpiler inside `build.rs` reads `versions/1.20.1/data/worldgen` (vanilla's worldgen JSON tree) at build time. Verification probes (`WorldgenRust/src/bin/*`) additionally need `blocks.json` + reference `.blocks` dumps — exported from a vanilla 1.20.1 server; intentionally kept out of the repo.

## How It Works

The Rust core reconstructs the density field following vanilla's exact semantics:

- **Noise primitives**: Xoroshiro128PlusPlus RNG, MD5-based seed derivation, Perlin / octave / double-perlin samplers — matching Mojang's implementation
- **Density function tree**: loaded at runtime from vanilla's `worldgen` JSON (`noise_settings/<dim>.json` + `density_function/<dim>/*.json`), mirroring `NoiseConfig`'s visitor semantics — **data-driven, no per-dimension code** (multi-world ready)
- **Build-time transpiler** (`build.rs`): compiles the same JSON into specialized native functions (splines inlined, caches resolved, CSE'd) — a second, independent evaluation path used as a correctness oracle and wired into production behind an env gate
- **Block pipeline**: density → aquifer → ore veins → surface rules → carvers → features, mirroring vanilla stage semantics (including dual noise/world heights for the Nether)

## Roadmap

1. ✅ **JNI bridge**: bulk chunk data exchange (now in Rust)
2. ✅ **Block layer**: density → block states (surface rules + chunk fill)
3. ✅ **Integration**: installable Fabric mod / server plugin
4. ✅ **Multi-world**: Nether engine + in-game dimension dispatch (End next)
5. **Nether polish**: fluid fill (lava oceans), bedrock edge layers
6. **Entity AI / pathfinding**: second core to nativify

## Credits

- **dustinmoon78** — Forge + Sinytra Connector compatibility: multi-level mod jar resolution (`CoreSwapFixHelper`) + direct `JarFile` extraction, tested on 400+ mod packs. See [#3](https://github.com/unknowbug/CoreSwap/pull/3).

## License

MIT
