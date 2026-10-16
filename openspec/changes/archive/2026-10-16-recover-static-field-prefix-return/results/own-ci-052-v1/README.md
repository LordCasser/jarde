# 052 own CI actual acceptance

产品 `052c82484bb20ca8b7e22ffe9943503c961280f9`、分支 `codex/static-field-prefix-return` 自身 CI [38859590941](https://github.com/LordCasser/jarde/actions/runs/38859590941) 实际 completed/success。4 jobs、52 steps均成功；133次API观察/266 raw，原ZIP851262B SHA256 fad67f6f7e5ac242d2cff6e5a175dc37bff43d1d0b99683204bc835b2918ba88，60安全唯一成员。

两个固定seed各370 summaries、3649 passed、0 failed、98 ignored。139具名门每seed显式ok；严格OpenSpec364/0，六个JDK/full-class命令实际9 tests成功，公开例子、feature/dependency、tracked-clean、MSRV、fuzz、supply-chain均按原workflow实际完成。root审计860 checks/0 errors，独审1667/0；按API step号/名称/时间窗核原日志，不造逐step执行结果。

portable包2813958B，SHA256 `6010a925be0101a657a3af273d730bb83e12e2e480d620bb40044a66fb7c8a47`，293 payload + manifest=294安全regular成员，root重开逐字核验；保留原API/ZIP、实际源Git树/CLI绑定及审计脚本/raw。独审v1因新日志布局未识别而fail-closed，原拒绝保留；v2按实际4 aggregate与step日志重新绑定后通过。这是审计解析器边界，不是产品CI失败。c075/a89两次真实产品失败在各自目录保留。

本结果仅绑定052产品；后续spec同步/归档/main为文档集成，不宣称文档head另有本次CI。FO完整原样双javac及8次隔离rebuilt-only运行见census-local-v1，不据此扩称FY/FX父计划或全局覆盖完成。

portable包另由独立逐成员/原cache/源文件/CLI核验通过；报告SHA256 `2a7ec11387db97b774713a79cea119dd9df3a0cd3f3f3b8fb3ade5ab46e71a3b`。
