# java-core — 跨版本共享的 Java 宿主适配核（260912-01 立项，用户已批准）

> **这是什么**：1.20.1 与 1.21.6 两个 Java 模组工程**共用的一份**宿主适配实现。
> 与 Rust 侧的 `worldgen-core/`（跨版本共享引擎）+ `versions/<ver>/rust/`（版本薄壳）对称。
> 计划：`.investigations/000-架构设计/架构计划-260912-01-共享Java适配核.md`（D3 / 范围 A / 机制 M1）。
> 过程记录：`.investigations/shared-java-core-260912-01/record-260912-01.md`。

## 铁律：一个类只有一个家

- **共享类** → `java-core/src/main/java/wg/...`
- **分版类**（mixin / 版本 API 缝实现 / 构建接线） → `versions/<ver>/java/src/main/java/wg/...`
- **同一个类不得两处同时存在**（会出现编译期歧义 + 双真相源）。核对工具：`.tmp/shared-java-core-260912-01/class_home_check.py`（V6）。

## 接线方式（每版 `build.gradle`）

```groovy
sourceSets {
    main {
        java {
            srcDir '../../../java-core/src/main/java'   // 相对工程目录解析（同 runDir 约定）
        }
    }
}
```

## 分版缝（`WgCompat`）

两版映射差异（如 `new Identifier(..)` → `Identifier.of(..)`、`registryManager.get(..)` → `getOrThrow(..)`、
`Util.getMainWorkerExecutor().named(..)` 的可用性）与**分版默认常量**（如 `BULKWB_ON`）一律收进
**分版** `WgCompat`，共享文件保持单一写法。共享文件里出现任何版本分支/映射差异即违规。

## 共享件来源与抽取验证状态（260912-01 Wave 1/2）

| 共享文件 | 来源（事实） | 1.20.1 条目级 V1 | 1.21.6 条目级 V1 |
|---|---|---|---|
| `wg/CppWorldgen.java` | 两版逐字节相同 | 不变 | 不变 |
| `wg/bench/WgDiag.java` | 两版逐字节相同 | 不变 | 不变 |
| `wg/bench/BenchMod.java` | 两版逐字节相同 | 不变 | 不变 |
| `wg/bench/StallWatch.java` | 1.20.1 逐字（零版本缝） | 不变 | 新增 |
| `wg/bench/CoreSwapFixHelper.java` | 1.21.6 逐字（两版差异仅注释） | 变（注释 3→6 行 ⇒ LineNumberTable 位移） | 不变 |
| `wg/bench/ChunkTiming.java` | 1.21.6 逐字（超集）+ 并集 javadoc | 变（取超集：+7 `LongAdder` 等） | 变（注释行数 ⇒ LineNumberTable 位移） |
| `wg/bench/BulkWb.java` | 1.20.1 逐字 + `ON` 改走缝 | 变（`ON` 表达式 + 行号） | 新增 |
| `wg/bench/CppBridge.java` | 1.20.1 为基线 + 并入 1.21.6 独有项（超集） | 变（超集） | 变（超集） |
| 分版 `wg/bench/WgCompat.java` | 新建（每版一份，只放分版常量/缝） | 新增 | 新增 |
| 1.21.6 `wg/bench/mixin/ChunkSectionAccessor.java` | 1.20.1 同形副本（S-1，供共享 `BulkWb` 编译） | — | 新增 |

> **头注释纪律**：共享件默认**不加以免改变产物字节的头部注释**——逐字复制的件保持与源逐字相同（否则行号位移会改变 class 条目，白白丢掉一个「字节等价」对照点）。
> 来源/取舍记录写在本表 + commit message，不写进被逐字复制的文件里。

## 验证门（V1 及其适用边界）

抽取属**等价重构**：抽取前冻结两版 jar（整 jar sha + 条目级 sha 清单），抽取后同配方重建后对拍。
工具：`.tmp/shared-java-core-260912-01/jar_manifest.py` + `jar_manifest_diff.py`。

- **V1-strict（纯移动子集）**：判据 = **非 mixin 条目 sha 全等**——Wave 1 实测成立（3 个逐字节相同类移动后两版 jar 逐字节不变）。
- **V1b（语义统一子集）**：引入缝/超集后，**至少一侧的 class 字节必变**，此时判据 =
  ① 未变更侧条目逐字节不变；② **变更条目逐条声明**（文件 + 变更性质 + 依据）；③ **变更侧 MUST 补跑 V2（run 回归）+ V3（行为门）**。
  「jar 不同」不再蕴含「行为相同」——这是 V1 的适用边界，见计划 §14.1。

