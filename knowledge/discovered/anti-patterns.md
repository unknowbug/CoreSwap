# discovered/anti-patterns — 混淆/反逆向手法

**空置**（2026-08-08 初始化）。

CoreSwap 逆向对象是 Java 版 Minecraft 算法源码（yarn mappings + sources jar），**非混淆二进制**——本分类一般用不上。若未来涉及：
- 混淆 jar（MCP 反混淆映射流程）→ 记录 ProGuard/R8 模式
- 崩溃调试（用户 0x34001 二进制定位）→ 可能用到 anti-re 知识（按需从 RE-Framework knowledge-builtin/anti-re.md 参考）
