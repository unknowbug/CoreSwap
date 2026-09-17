# CP-1 光照增量形态对齐 · 错误台账（260916-01）

五段式（现象/根因/定位/修复/教训），错误优先原则。

## E1 域任务线程读 PalettedContainer 触发 vanilla 多线程 crash 检测器

- **现象**：CP1-D3 run1 boot 期（Preparing spawn 34%）`Server thread/ERROR: net.minecraft.util.crash.CrashException: Accessing PalettedContainer from multiple threads`，服务器中止；崩栈直指 `wgLightFillSectionFast → PalettedContainer.writePacket → lock()`，dump 中一线程持锁、域批 worker 线程在 `LockHelper.lock` acquire（与 #113 崩溃形态同构：持锁帧 + acquire 帧）。
- **根因**：首版把 5×5 收集放在域批任务线程（Util.getMainWorkerExecutor）；多个域批并行 + 5×5 窗重叠 ⇒ 同一 chunk 的 PalettedContainer 被两个域任务**并发 writePacket**。writePacket 的锁附带 crash 检测器（#24「检测器随优化移除」的对偶面：检测器在场时并发读=崩溃而非脏数据）。legacy 路径同样有跨线程并发读（相邻 chunk 的 3×3 窗重叠、各自 caller 线程收集）但碰撞窗口小，历史上未触发——**「历史上没崩」是碰撞概率低，不是安全**。
- **定位**：crash dump 双线程栈直接给出持锁/acquire 两帧；两帧都落在本 mixin 收集函数（wgLightFillSectionFast ← wgLightCollectBlocks25 ← wgLightDomainTaskRun），排除写回/JNI 侧。
- **修复**：收集回提交线程——light() HEAD 预检通过后**当场收集本中心 blocks9 快照**（与 legacy 同线程同构，碰撞面回到历史已验证形态），快照随提交入批（LightDomainBatch.State.blocks9s）；域批任务只做「拼帧 + JNI + 写回」，**零 PalettedContainer 访问**。5×5 收集减省（81→25）随之放弃，批 JNI + 执行位置解耦保留。
- **教训**：①「接管类改造把工作搬到别的线程」时，**所有**触碰 vanilla 带锁/带检测器对象的代码必须随行迁移审查——收集、写回、状态查询逐一过（#113 判据的主动版）；②「同窗重叠读」在旧形态下安全只是概率性安全，放大并发度（×9 批化）会把概率性缺陷变必然——**并发度是隐藏缺陷的放大器**（#150 家族）；③ vanilla 系统里「读」不等于无锁安全（writePacket 是读，但要拿排他锁）。

## E2 残留 java 进程占用 .fabric processedMods 致 runServer 启动即败

- **现象**：CP1-D3b `[FAIL] early exit rc=1`，`FileSystemException: .\.fabric\processedMods\chunky-*.jar 另一个程序正在使用`。
- **根因**：CP1-D3 失败轮脚本 `taskkill /F /IM java.exe` 只在 gradle.bat 未退分支执行且实际未杀干净（bat 包装层），java 子进程残留持有 processedMods 文件锁。
- **定位**：启动即败 + 文件占用报错 → 查 Get-Process java 的 StartTime（22:01-22:25 四个残留）。
- **修复**：按 StartTime 归属定向 Stop-Process（#153），全清后重跑。
- **教训**：run 脚本的进程清理要核对**实际 JVM PID**（gradle.bat 的 PID ≠ java PID）；失败轮后先清场再复跑。

## E3 run_g3_cp1.py 无 cmd-output 目录 / 日志覆盖风险

- **现象**：首轮 FileNotFoundError（目录不存在）。
- **修复**：New-Item 目录 + run_cp1.py 里日志存在即退出换标签（#144/#146）。
- **教训**：复用脚本改 OUTDIR 时，目录创建与不覆盖守卫要一并带上（#146 判据第二实例）。
