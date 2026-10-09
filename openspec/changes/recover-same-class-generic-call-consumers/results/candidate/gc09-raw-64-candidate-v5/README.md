# GC09 raw-64 候选回放：固定 CLI5

本目录记录 GC09 的 64 项 raw-64 回放；它不替代 OpenSpec change 的其余矩阵和门禁。

## 固定候选与输入

- 候选标签：`gc09-raw-64-candidate-v5-cli5`。
- 固定 CLI：`/private/tmp/jarde-generic-calls-candidate-v5-cli`，SHA-256：`010e9f1ecd82a1f2c7c4dcd49010afc787d3980a0194e9fc6a1d12994132dac5`。
- 源码快照：v36，归档 SHA-256：`870589b655d855c7eedf6951c230bd8a69158f98b4ea54f54c14da008d9dc9a2`。
- 回放 adapter 为 v2（SHA-256：`e170ad879448687dfe972f615dfd78a56406d5b5bc1b9a841a526fdee2633fb3`）；实际 runner 快照 SHA-256：`2c9b8233f41f13a483fa252a898f81e4602b9c7b3a0308cbc95bcd42162dc3a2`。
- 输入覆盖 16 个 fixture 家族、JDK 8/JDK 23、debug/no-debug，共 64 个 JAR。运行前后核对了 18 个 runner Java 源、23 个 fixture Java 源及 64 个输入 JAR；哈希均未变化。
- 使用的 baseline 固定为同级 `gc09-raw-64-baseline-v1`；候选 v1–v4 的结果未覆盖。

## 编译、行为与 API 对照

| Flavor | 完整类编译 | `-Xverify:all` 行为匹配 | 字段泛型 API | 方法 API | 类 API/bounds |
| --- | ---: | ---: | ---: | ---: | ---: |
| 原始源码/Probe | 64/64 | 64/64 | 64/64（参考） | 64/64（参考） | 64/64（参考） |
| 接受的 baseline | 64/64 | 64/64 | 60/64 | 52/64 | 60/64 |
| CLI5 候选 | 64/64 | 64/64 | 60/64 | 52/64 | 60/64 |
| JADX | 60/64 | 60/60 已编译项 | 60/60 已编译项 | 60/60 已编译项 | 60/60 已编译项 |

API 对照检查字段的泛型类型及变量身份、声明方法及方法变量、类变量及 bounds。JADX 的比较去除了反编译包名前缀 `defpackage.`；四个未成功编译的 JADX 输入按 unavailable 处理，不计为匹配。候选对比接受的 baseline 没有新增 API 回退；独立 verifier 会重新计算该结论。

原始源码、Probe 编译及原始运行均为 64/64。CLI5 的生成类使用空 classpath 和空 sourcepath 编译，再仅用新生成的 classes 目录执行 `-Xverify:all`，分别为 64/64。JADX 有 4 个 `InstanceRawLocal` 编译失败，stderr 保存在 `failures/`；其余 60 项均通过 `-Xverify:all`，60 项行为与原始 Probe 匹配。候选行为 64 项均与原始 Probe 匹配。

## 可复核记录

- `commands.tsv` 共 842 行：710 个实际子进程和 132 个 source-header 合成检查。合成检查是 runner 内检查，不计作子进程。
- `subprocesses.jsonl` 保存每个实际命令的 argv、退出码及 stdout/stderr 路径和哈希；`subprocess-logs/` 保存两个输出流，包括空流。编译命令使用隔离的空 classpath/sourcepath，运行命令只使用各自新生成的 classes。
- `outer-preflight-metadata.json` 和 `outer-run-metadata.json` 保存固定 CLI/runner/adapter 身份、JDK 与工具信息、输入哈希前后核验及实际命令记录。工具清单记录了 59 个 CLI/JADX 工具文件和 8 个 JDK 工具版本查询。
- `versions/` 保存工具与冻结 JAR 哈希；`result-file-hashes.tsv` 索引本目录结果文件。
- 运行结束后结果目录没有 `.class` 文件。独立核验脚本副本位于仓库 `results/root-input-audit/verify-raw-replay-v5.py`；核验 JSON 与其结果哈希另存于同目录。
