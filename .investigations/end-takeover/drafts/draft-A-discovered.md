# draft-A — workflow-patterns.md 追加草稿（260906-04 end 接管开发块）

> 目标文件：`knowledge/discovered/workflow-patterns.md`（现最大 #55，本稿续接 #56 起）。
> 素材：`.investigations/end-takeover/260906-04-errors.md`（E1/E3/E5 + 附条）。E2 归 compiler-idioms（见 draft-B）。
> 主会话应用后须同步 INDEX.md。状态：draft（subagent 草稿，待主会话应用 + judge）。

---

## 发现 #56: 大 match 新增分支禁止落在 catch-all 之后——「编译过 + 告警不新增」不等于「命中」（260906-04）

- **时间/置信度/module**：260906-04；candidate（panic 日志 + 源码目视双证，修复后 end 分支命中实证）；workflow-patterns / Rust 移植陷阱。

### 现象（现象→根因）
`create_for_dim("end")` panic：`resolve minecraft:end/sloped_cheese failed: unsupported density type 'minecraft:end_islands'`——而 end_islands 的 match arm 明明已经写好。

### 根因（机制层面）
新分支被插在 `_ => return Err(...)` **之后**。Rust match 按序匹配，catch-all 先命中。与 Java switch-case 陷阱同类，但 Rust 更隐蔽：编译器对 unreachable pattern 只告警不报错，且 `_ =>` 在前时**新 arm 不算 unreachable**（它是可达的，只是永远轮不到），连告警都没有——完全静默。

### 定位（怎么发现的）
读 `density_builder.rs` L385-395 目视确认 arm 顺序。无工具成本——但前提是想到「先查顺序」这个方向。

### 修复
end_islands arm 移到 `_ =>` 之前。

### 教训/判据
1. **判据（MUST）**：给大 match 加分支时禁止落在 catch-all 之后；PR/review 时对「新增 arm」先 grep catch-all 位置。
2. 「编译过 + warning 不新增」≠「命中」——**新增分支必须有运行时命中证据**（一条日志/一个探针值），首次接入新数据（end.json）时首个端到端 run 即暴露，此例正是首跑 panic 才发现。
3. 家族索引：与 #20（死参数制造假判别）同构——「写了但没生效」的静默失效家族，本条是 match 序位置版。

### 证据
`.investigations/end-takeover/260906-04-errors.md` E1；`density_builder.rs` L385-395。

---

## 发现 #57: 哨兵值跨模块传递必须在边界做语义变换——「新维度新地形形态（全空气列）」是哨兵路径的天然压力测试（260906-04）

- **时间/置信度/module**：260906-04；candidate（RUST_BACKTRACE 链 + 哨兵定义 grep + 修复后空列 chunk 通过实证）；workflow-patterns / 跨模块边界语义（维度移植通用）。

### 现象（现象→根因）
benchseed fill 外围 chunk panic：`attempt to subtract with overflow` @ `biome.rs:349`（`block_y - 2`，block_y = i32::MIN）。

### 根因（机制层面）
`ChunkDensity.surface_height` 的空列哨兵 = i32::MIN（`terrain.rs:241`「无 solid」）。end 外岛空域存在**大量全空气列**——overworld 每列必有基岩，哨兵路径在生产中不可达；end 把它变成了热路径：`o = heightmap + 1` 把 MIN 直接喂进 biome 采样链。Java 语义：WORLD_SURFACE_WG 空列 get = `bottom - 1`（一个有界的合法值），不是 INT_MIN。这是**维度特有路径缺陷**——同一代码在旧维度全对，换维度即炸。

### 定位（怎么发现的）
`RUST_BACKTRACE=1` 给出 closure 链（build_surface closure$1 → biomeCellKey closure），确认溢出发生在哨兵消费点；grep `surface_height` 定义处见哨兵注释。

### 修复
在 `fill_terrain_column` 消费点统一映射 `MIN → min_y - 1`（对齐 Java Heightmap 空列语义）。选**消费点边界**而非生产点改哨兵：所有下游消费者（features/placement 等）同样受益，且 MIN 哨兵本身保留「无值」判别能力。

### 教训/判据
1. **判据（MUST）**：哨兵值跨模块传递前，必须在边界声明语义并做显式变换——下游代码不应当面对裸 INT_MIN 做算术。
2. **新维度接入的固定测试面**：「新维度新地形形态（全空气列/全液体列/超高层列）」是哨兵路径的天然压力测试——新维度首跑 MUST 覆盖空列 chunk，不能只用旧维度坐标烟测。
3. 判错经验：debug profile 下「subtract with overflow」+ 操作数 = 极值 → 先查上游哨兵/未初始化语义，不是先查公式精度。

### 证据
`.investigations/end-takeover/260906-04-errors.md` E3；`terrain.rs:241`（哨兵定义）、`biome.rs:349`（溢出点）。

---

## 发现 #58（最高价值，详写）: 维度 chunk 形状（bottomY/height）≠ noise.height，且维度间形状可碰撞——新维度接入首查「形状碰撞」，同形则 settings id 是唯一判别（260906-04）

- **时间/置信度/module**：260906-04；candidate（生产环境放行日志实侧 + judge PASS-3/PASS-4 复核）；workflow-patterns / 维度接管判别（BUG-002 #55 的同构延续）。

### 现象（现象→根因）
生产环境 forceload end 后无 `[Mixin] populateNoise(end)` 接管日志，只有一次性放行日志：`release to vanilla: settings=minecraft:end shape=0/256`。首版 mixin 按 `bottomY==0 && height==128` 判 end（照抄 noise_settings/end.json 的 `noise.height=128`），与实侧 0/256 不符 → end 永远落不进接管判定。

### 根因（机制层面）
两个机制叠加：
1. **`chunk.getHeight()` 返回的是维度类型（DimensionType）高度，不是噪声高度**。end.json `noise.height=128` 是密度函数域的高度，the_end 维度类型的 chunk 形状 = bottomY 0 / height 256（fillChunkEnd 写 128 高度、上半 vanilla 空，是正确的双层结构）。「noise.height ≠ 维度 height」——用 settings JSON 的 noise 字段外推 chunk 形状是域混淆。
2. **维度间形状可碰撞**：end 与 nether 同为 0/256——形状指纹（#55 时代 BUG-002 的根因载体）在 end 接管场景**原理上不可用**：同形维度靠形状永远区分不了。settings id 是唯一判别。

### 定位（怎么发现的）
放行日志的 `shape=0/256` 直接给出实侧值，与 mixin 条件里的 128 对照即见差——行为日志打「实侧形状」这一诊断姿势一次定位。

### 修复
end 判定改为 `bottomY==0 && height==256 && endActive && settings==minecraft:end`（settings id 判别 + endActive 双闸，承 #55 的 registry entry key 路线）；形状条件变量改名 `zeroShape`，在命名上表达「这是与 nether 共享的 0/256 形状，非 end 专属」的事实，防后人再把形状当维度身份读。fillChunkEnd 仍写 128 高度（8 sections）。

### 教训/判据
1. **判据（MUST）**：维度 chunk 形状判定只能用**维度类型实测值**（运行时日志/DimensionType），禁止从 noise_settings JSON 的 `noise.min_y/height` 外推——两者是不同层的量。
2. **新维度接入首查形状碰撞**：接入清单第一步 = 「该维度形状与哪个既有维度同形？」（已知同形对：end/nether = 0/256）。同形 → 形状指纹废弃，settings id（registry entry key，#55 判据）+ 显式开关是唯一判别组合。
3. **命名编码事实**：共享形状的判定变量不要叫 `isEndShape` 这类排他名——`zeroShape` 这类「描述形状本身」的命名防止下一个接入者照名字误用（BUG-002→本条的两次事故都源于把共享形状读成专属身份）。
4. 家族索引：#55（BUG-002，形状指纹 + 裸反射失效；本条是其 end 接管延续——反射已修，形状不可判别性显式化）；#53（「当公理续推」家族——noise.height=128 从 JSON 直推 chunk 形状属同型）。

### 证据
`.investigations/end-takeover/260906-04-errors.md` E5；`.investigations/end-takeover/review-002-final.md` PASS-3/PASS-4（zeroShape 条件集等价 + 双闸边界复核）；生产日志 `release to vanilla: settings=minecraft:end shape=0/256`。

---

## 发现 #59: 跨语言数值对拍必须数值化比较，禁字符串比对——格式差异制造全量假阴性（260906-04）

- **时间/置信度/module**：260906-04；candidate（两侧数值实际全等、仅格式不同的比对记录实证）；workflow-patterns / 对拍方法（#12 对拍解析产物家族）。

### 现象（现象→根因）
多 seed 对拍第一轮全 FAIL。实际 Java `%.17g` 与 Rust `{:.17e}` 输出格式不同（同一 double 的两种 17 位有效数字表示），PowerShell `Compare-Object` 按行字符串比对 → 数值全等也被判差异。

### 根因（机制层面）
比对器作用在**表示层**而非**值层**。跨语言浮点序列化的格式约定（%g 的有效数字/指数形式 vs Rust 的 mantissa e exponent 形式）注定不同，字符串相等从未是判据。

### 定位（怎么发现的）
抽一对「FAIL」行手工解析为 double 比较——全等，确认为假阴性。

### 修复
对拍脚本改数值化比较：两侧解析为 f64 后按位/精确相等判定（或两侧统一格式化函数后比对）。

### 教训/判据
1. **判据（MUST）**：跨语言数值对拍必须数值化比较（解析成数值后比），或两侧统一由同一个格式化函数输出；禁止直接 diff/Compare-Object 原始输出行。
2. 「第一轮全 FAIL」是假阴性高危签名——真差异通常有分布（部分 PASS），全 FAIL 先怀疑比对方法再怀疑实现（与 #13「100% 单向假象」同型：全量级的异常结果先怀疑测量层）。
3. 家族索引：#12（对拍解析产物而非输入原文——比对对象层级错置家族）。

### 证据
`.investigations/end-takeover/260906-04-errors.md` 附条。
