# knowledge/INDEX.md 追加行草稿（260919-06 块；主会话应用，本文件只含草稿）

> 编号以 knowledge-draft-discovered.md 为准（build-tooling #160/#161、workflow-patterns #195、#100 追加注记）；应用前重核两文件末尾防撞号。

```
> 260919-06 追加：build-tooling 新增**发现 #160（最高价值·错误优先）**（Sponge MixinProcessor 对已注册 mixin 类的直接 classload 是设计性拒绝——IllegalClassLoadError「cannot be referenced directly」（三条消息分支：已注册 mixin/accessor 变体/包级兜底），经 KnotClassDelegate.java:422-427 包装成「Mixin transformation failed」顶层消息、被应用侧 BlobProbe.java:46 吞掉 cause 的三段链；修复模式 = 计数器外移普通 holder 类（1.21.6 BlobProbeStats 先例，1.20.1 出货树登记已知限制）；判错签名 =「stats read failed + 只见顶层消息」即展开 cause 链/查反射自载，不立版本兼容课题；#40/#157/#176 家族的机制根，一手 = fabric-loader sources + sponge-mixin javap 字节码（Degraded），升 candidate 条件 = getCause 打点一次）+ **发现 #161 简记**（比较器抽样显示缺陷模式——对 str key 做序列切片 → 显示失真但计数正确，判读时「计数与显示分离核」：sample 行手工对原始 key 验显示再信计数，交付诚实登记缺陷层次）；workflow-patterns 新增**发现 #195**（双臂对拍判据预登记模板四件套——preconditions 表含负自证硬门 + 存在性判据无阈值带 + VOID 非零退出 + 覆盖面/外推边界声明；判据/驱动/比较器同批定稿先于采集，预登记读法机械执行）+ **#100 追加注记**（E1 run 间噪声带 ≠ E2 跨执行体上界——量级旁证引用跨档位带 MUST 声明档位差且主判据不依赖该句，judge N2 教训）。来源：`.investigations/legacy-sweep-260919-06/`（backlog-map + knot-selftransform-static）+ `.artifacts/legacy-sweep-260919-06/`（record-lowcost + judge-review-260919-06 PASS-with-conditions N1-N7；draft，confirmed 留用户）；时间线 → versions/1.20.1/docs/10-timewise-archive.md 260919-06 块。
```

> 备注：本块 8 项旧遗留 sweep 的用户拍板结论（B/C 续挂、B3 续挂、open② 不动出货树、T7 转关账+idk、T8 FAIL 课题化留用户）均属一次性/范围决策，按价值门不进 INDEX/discovered，只进 10 时间线。
