# CP-6 needsSaving 标脏完整性 — 一手源核对记录（260915-02）

> 状态：candidate（一手源 file:line 引证，静态核对；未做运行时复现——非必需，机制链完整）
> 疑点来源：form-audit-260915-01 CP-6（W3 主嫌疑，推理级）：「bulk 原地替换不经 setBlockState/标脏链 → 早 unload 存盘场景生成方块可能不落盘」
> 一手源：`.tmp/scout-260905-08/mcsrc`（1.20.1 yarn，DataVersion 3465 已核）

## 核对结论：疑点不成立（vanilla 侧标脏不依赖 setBlockState 链）

**证据链（全部一手源 file:line）**：

1. **门控真实存在**：`ThreadedAnvilChunkStorage.save()` :797-802 —— `if (!chunk.needsSaving()) return false;` 才序列化。✅ 与候选池描述一致（交接结论廉价验证通过）。
2. **块级写不标脏（两侧同构）**：
   - vanilla `ProtoChunk.setBlockState`（ProtoChunk.java :90-158）：写 section/heightmap/light 检查，**无任何 needsSaving 置位**。
   - CoreSwap bulk 写回（`CppBridge.writeChunk` → `ChunkSection.setBlockState` → `PalettedContainer.swap`）：同样不标脏。
   - ⇒ vanilla 自己的 populateNoise 块写也不标脏——「标脏靠块写链」前提本身对 vanilla 不成立。
3. **真正的标脏点 = 每个生成阶段完成时统一置位**：
   - `ChunkStatus.runGenerationTask` :357-363：`doWork(...).thenApply(... protoChunk.setStatus(this))` —— **每个 status 阶段任务完成后**（`!status.isAtLeast(this)` 时）调 `setStatus`。
   - `ProtoChunk.setStatus` :215-222：末行 `this.setNeedsSaving(true)`。
   - CoreSwap 拦截点在 NOISE（populateNoise）阶段内；**阶段完成即 thenApply 置位**，与块写入路径无关。
4. **补充标脏面**（后续阶段再兜底）：`Chunk.setLightOn` :389-391（光照 finalize 置位）；`Chunk` 结构四方法 :214/224/235/247；`ChunkHolder.markForLightUpdate` :190；`ChunkSerializer` 载入 :213。
5. **边界（如实声明）**：若 chunk 在阶段完成前被 abort/unload（thenApply 未跑），标脏不发生——但该行为 vanilla 自身同构（vanilla 的块写同样不标脏），属管线取消语义而非 CoreSwap 偏差，无对齐缺口。

## 结论

- **CP-6 结案：不立项**。bulk 原地替换不落盘的担忧被一手源否定——`needsSaving` 门控在，但置位机制是「阶段完成 setStatus 统一标脏」，CoreSwap 与 vanilla 在同一标脏面上（均为块写不标脏、阶段完成标脏）。
- 置信度：candidate（静态一手源核对，验证分层 = Degraded（静态审查），如实声明）。

**范围边界（judge C4 补充）**：
- `ChunkStatus:361 isAtLeast(this)` 已达标路径（重入/跳跃推进）不再置位——良性，chunk 早已标脏或已序列化过。
- 本核对覆盖面 = **生成期**（ProtoChunk 管线内）的 bulk 写回；**非生成期**（已 LEVELCHUNK 后）的全量 section 替换不在覆盖面内，若未来出现该形态需另核（届时标脏面走 WorldChunk:303/World.java:798 域，不同链）。
- WrapperProtoChunk（:181-186 转发 setNeedsSaving）不进生成任务链（是 worlds 边界视图），无独立缺口。
- 判据沉淀候选（交知识库 subagent 评估）：「MC 生成期 chunk 持久化标脏 = status 阶段完成统一置位（ChunkStatus:362→ProtoChunk:221），不随块写发生；审查生成期写回路径落盘完整性时核对阶段推进链，不核对 setBlockState 链」。

## 错误记录

无运行时错误。核对前置 = 候选池「TACR:797-802」行号引用与一手源实际行号一致（797-802 确为 save() 门控），转抄无漂移（#90 家族通过例）。
