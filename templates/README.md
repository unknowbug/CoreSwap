# templates/ — 产物 schema 模板（语言无关）

> Spec: engineering-framework-v1.md §5
> 定位方式三选一，按模块选用：`address`（虚拟地址，re-binary）/ 字节码偏移或类全名（re-code）/ 成员名。

| 文件 | 用途 | 归属 |
|------|------|------|
| `index.yaml` | 主索引（必须维护） | core |
| `class.yaml` | 类/结构产物 | core |
| `method.yaml` | 方法产物 | core |
| `function.yaml` | 未归类函数产物 | core |
| `xref.yaml` | 跨引用 | core |
| `uncompilable_functions.yaml` | 降级声明（不可独立编译） | re-binary 常用 |
| `noise_cards.json` | 噪声卡（失败反馈累积） | 引用 Anchorlaw §3 |

## anchor 载体（验证层）

本框架的验证协议继承 [Anchorlaw v0.13](https://github.com/unknowbug/anchorlaw)（不复制实现，协议引用）：
- Python anchor 载体：`@anchor.test` / `@anchor.idk`（`anchorlaw` 包，或 `anchorlaw init` 生成的 stub）
- 无 anchorlaw 安装时：装饰器自动降级为 no-op（Anchorlaw §2 Uninstall Guarantee）
- 其他语言：Anchorlaw §13 三语言等价（Python 装饰器 / TS JSDoc / C++ 行注释）
