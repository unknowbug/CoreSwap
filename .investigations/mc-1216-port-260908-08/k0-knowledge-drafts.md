# k0 知识库草稿（260908-08，subagent 产出，主会话应用前先过 INDEX/价值门核对）

> 价值门判定：#38 高价值详写 / #39 中价值简写 / 「diff 四路并行 + 条件清偿」候选低价值不写（无新判据，#3/#73/#76/#77 既有制度覆盖）。
> ⚠️ #38 回填钩子：403→镜像切换的具体复现日志在 .investigations/mc-1216-port-260908-08/ 内未检索到（仅 scout-1a 记录 9199 代理配置 + 联网依赖），按 candidate 标注；主会话应用前若有构建日志请补入来源定位。

---

## 草稿一：追加到 knowledge/discovered/build-tooling.md 末尾

```markdown
## 发现 #38: mavenCentral 经本地代理 9199 全 403 → build.gradle 换阿里云镜像；pwsh/curl 出网 SSL 全挂是常态、gradle JVM 网络栈另算——宿主网络分层不互相代表（260908-08）

- **时间/置信度/module**：260908-08；candidate（scout-1a 代理配置一手 + 本轮构建记录；镜像切换的具体 403 复现日志如另存应回填链接）；build-tooling / 网络通道。
- **来源定位**：`.investigations/mc-1216-port-260908-08/scout-1a-reference-source.md`（gradle.properties 本地代理 127.0.0.1:9199，loom 官方 jar/yarn 依赖 maven.fabricmc.net/mojang 通道）+ 本工作块 1.21.6 参照链搭建记录。
- **现象**：1.21.6 参照工程首建 `gradlew build` 需联网拉取 jar/yarn/loom 依赖；mavenCentral(repo1) 走本地代理 9199 时全部请求 403（代理对该上游拒绝）；同时 pwsh/curl 直接出网 SSL 握手全挂（本沙箱环境常态）。
- **根因（机制）**：① 代理对 repo1.maven.org 上游的拒绝是代理策略问题，不是本机网络不可用；② pwsh/curl 的 SSL 失败与 gradle 失败分属**不同进程的网络栈**（curl 用 schannel/OpenSSL，gradle 用 JVM truststore + 自己的代理设置）——一个通道挂不推出另一个通道挂，「工具级出网失败」极易被误读成「本机无法联网」从而放弃构建。
- **定位**：分层排除——先看 gradle 报错里具体 upstream URL 与状态码（403 = 代理侧拒绝而非 DNS/SSL 失败），再对比 curl 同 URL 表现，确认两层独立。
- **修复**：build.gradle/settings.gradle 仓库列表把 mavenCentral 置于阿里云镜像（`https://maven.aliyun.com/repository/public` 等）之后/替换，绕开代理对 repo1 的 403；gradle 自身走 JVM 网络栈（gradle.properties 代理配置）正常通。
- **教训/判据**：① gradle 联网构建失败先按「仓库源 × 代理策略」矩阵排查，镜像换源是 mavenCentral 403 的首选低成本修复；② **「pwsh/curl 出网 SSL 全挂」在本环境是常态基线，不作「网络不可用」判据**——判断某构建通道可用性必须用该通道自己的进程（gradle = JVM 栈）实测，宿主网络分层不互相代表；③ 新版本探针工程首建前预留联网需求声明（scout-1a G-62 已预置），依赖落 `.gradle-home/caches/fabric-loom/` 后即可离线复建。
- **证据**：scout-1a-reference-source.md §gradle.properties/要点/G-62。
```

## 草稿二：追加到 knowledge/discovered/build-tooling.md 末尾（紧随 #38）

```markdown
## 发现 #39 简记: 多版本 dll 同名冲突裁决——薄壳 cdylib 包名带版本（worldgen1216→worldgen1216.dll）+ processResources rename 回 worldgen.dll，两版本输入互不覆盖（260908-08）

workspace 多版本薄壳并存时 cdylib 产物同名（都叫 worldgen.dll）会互相覆盖 target 产物；裁决 = 每版本薄壳包名内嵌版本号（`worldgen1216` → 产物 `worldgen1216.dll`），version-specific 的 build.gradle `processResources` 再 rename 回运行时固定名 `worldgen.dll`——版本隔离在构建产物层，运行时契约名不变。下个版本 = 新薄壳包名 + 对应 rename，模式照抄。（260905-01 workspace 拆分 §13a 的多版本延伸。）来源：`.investigations/mc-1216-port-260908-08/port-list-260908-08.md` P1。**置信度**：本轮已定稿裁决（清单 judge PASS-with-conditions 已过）。
```

---

## INDEX.md 追加行（两条，追加到分类表 build-tooling 行末尾）

```
 + mavenCentral 经代理 9199 全 403 → build.gradle 阿里云镜像换源 + pwsh/curl SSL 全挂是常态、gradle JVM 网络栈另算——宿主网络分层不互相代表（发现 #38，260908-08）
 + 多版本 dll 同名冲突裁决——薄壳包名带版本（worldgen1216.dll）+ processResources rename 回 worldgen.dll，版本隔离在构建产物层（发现 #39，260908-08）
```

> 编号核对：build-tooling.md 现至 #37，本两案依序占 #38/#39。
