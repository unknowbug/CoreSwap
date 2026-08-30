
**First Rust-core release.** The worldgen native core has migrated from C++ to Rust — one `worldgen.dll` now ships the JNI bridge and the engine together.

## Highlights

- **Rust core, single dll**: JNI bridge (`Java_wg_CppWorldgen_*`) + engine (`wg_*` C ABI) in one Rust cdylib; C++ line archived
- **Overworld NOISE+SURFACE**: density → aquifer → ore veins → surface rules → carvers through the Rust engine. Block match vs vanilla **95.40%** (baseline) / **94.27%** (transpiler engine); density aligned to floating-point residual (<5e-7). Verified in-game (server + client)
- **Build-time transpiler in production** (env-gated): vanilla density JSON compiled to native code; proven consistent with the runtime interpreter across a 98304-point full-chunk sweep (max diff <5e-7), block match 99.30%. Recent milestone: fixed a `flat_cache` quantization semantics bug found by expanded alignment sampling (M13, confirmed)
- **Multi-world — the Nether**: same Rust engine via a new `initDim` JNI path + per-dimension mixin dispatch (with End mis-route protection). Nether block match **74%** vs vanilla; known gaps: lava-ocean fluid fill, bedrock edge layers. Overworld unaffected
- **Measured performance**: large-sample end-to-end **~1.2× vanilla Java**; 16-chunk in-engine JNI sweep (full pipeline incl. carvers + features) **~14 ms/chunk**. Parity between transpiler and baseline engines (multi-run 0.92–1.05×)
- **Dual loader support — Fabric + Forge**: one jar for both (Fabric native; Forge via Sinytra Connector — jar structure verified identical to the Connector-tested 1.0.18). Works on dedicated servers too

## Install

Same as before: drop the jar into `mods/` (Fabric Loader 0.15.x, MC 1.20.1, Java 17+). Verify in `logs/latest.log`:

```
[BenchMod] CoreSwap replace mode: C++ worldgen active
[CppBridge] init seed=... enabled=true
[CppBridge] initNether seed=... enabled=true
```

(The "C++ worldgen" log line is kept for historical reasons — the native core is Rust since this build.)

## Known limitations

- Block-level match is 94–95% (not yet bit-identical worlds; remaining diffs concentrate in deep layers / density-near-zero boundary flips)
- Nether: lava oceans & bedrock edges not yet fluid-filled/aligned (74%)
- End dimension: vanilla (engine not started; mis-route protection wired)
- FEATURES stage coverage for trees/vegetation decoration remains out of scope

## Notes for builders

- Toolchain: Rust (stable) + JDK 17 + Gradle 8.x — see README "Building from source"
- `build.rs` regenerates transpiled density code from vanilla JSON at build time

**Full changelog**: https://github.com/unknowbug/CoreSwap/compare/coreswap-1.20.1-1.0.18...coreswap-1.20.1-1.0.19-beta
