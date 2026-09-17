# K2 驱动台 console 输出归档（judge B5 SHOULD 条件补档，260917-03）

> 来源 = 驱动 pwsh 后台 job 的 stdout 转录（驱动脚本 console 未落盘的缺陷已识别）；权威原始证据仍在各 .log 与
> _light_after.json（judge 已独立复验：三日志 sha 行 :254 / lightInit ok / hook armed / target+jtmp dll sha256 重算一致）。

```
[K2-D4 第一次·VOID] （K2-D4-VOID1.log）
[K2-D4] changed=81/2025 = 4.0000%  onlyPrev=0 onlyNow=0
[SELFCERT] {"lightInit_ok": 0, "fallback": 1, "done": 1, "domain_hook": 0, "domain_sealed_min": 0, "dll_sha": "(none)"}
[GATE] VOID

[K2-W]
[K2-W] changed=0/2025 = 0.0000%  onlyPrev=0 onlyNow=0
[SELFCERT] {"lightInit_ok": 1, "fallback": 0, "done": 1, "domain_hook": 1, "domain_sealed_min": 2, "dll_sha": "6f7fa3ae"}
[GATE] PASS

[K2-D4]
[K2-D4] changed=0/2025 = 0.0000%  onlyPrev=0 onlyNow=0
[SELFCERT] {"lightInit_ok": 1, "fallback": 0, "done": 1, "domain_hook": 1, "domain_sealed_min": 1, "dll_sha": "6f7fa3ae"}
[GATE] PASS

[K2-D5]
[K2-D5] changed=0/2025 = 0.0000%  onlyPrev=0 onlyNow=0
[SELFCERT] {"lightInit_ok": 1, "fallback": 0, "done": 1, "domain_hook": 1, "domain_sealed_min": 1, "dll_sha": "6f7fa3ae"}
[GATE] PASS
```
