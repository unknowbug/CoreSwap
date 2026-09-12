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

## 验证门（V1）

抽取属**等价重构**：抽取前冻结两版 jar（整 jar sha + 条目级 sha 清单），抽取后同配方重建，
判据 = **非 mixin 条目 sha 全等**（差异条目逐条声明）。
工具：`.tmp/shared-java-core-260912-01/jar_manifest.py` + `jar_manifest_diff.py`。
