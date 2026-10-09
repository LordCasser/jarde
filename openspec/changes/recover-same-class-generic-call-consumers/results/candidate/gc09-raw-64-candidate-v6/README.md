# GC09 raw-64 候选回放：固定 CLI6

本目录保存 GC09 的 64 项 raw-64 回放。它只覆盖此矩阵，不替代 OpenSpec change 的其他矩阵和门禁。

## 固定候选与输入

- 候选标签：`gc09-raw-64-candidate-v6-cli6`。
- 固定 CLI：`/private/tmp/jarde-generic-calls-candidate-v6-cli`，SHA-256：`c42c2f64df21fd74e22710e6887eac0040850d2e03906a202e051b65af1dc6d8`。
- 源码快照：v38，归档 SHA-256：`83b1da417d94bcbcfd83d8871a053be93aa31cb2d2e6e3e05f215699db5a5f11`；八个源码文件哈希见 `results/local-gates/candidate-cli-v6.json`。
- 原始 runner 快照 SHA-256：`2c9b8233f41f13a483fa252a898f81e4602b9c7b3a0308cbc95bcd42162dc3a2`。未修改的回放 adapter v2 SHA-256：`e170ad879448687dfe972f615dfd78a56406d5b5bc1b9a841a526fdee2633fb3`。
- 输入覆盖 16 个 fixture 家族、JDK 8/JDK 23、debug/no-debug，共 64 个冻结 JAR。运行前后核对 18 个 runner Java 源、23 个 fixture Java 源和 64 个输入 JAR；哈希保持不变。
- 接受的 baseline 固定为同级 `gc09-raw-64-baseline-v1`；之前 v3、v4、v5 结果均保留。

## 编译、行为与 API 对照

| Flavor | 完整类编译 | `-Xverify:all` 行为匹配 | 字段泛型 API | 方法 API | 类 API/bounds |
| --- | ---: | ---: | ---: | ---: | ---: |
| 原始源码/Probe | 64/64 | 64/64 | 64/64（参考） | 64/64（参考） | 64/64（参考） |
| 接受的 baseline | 64/64 | 64/64 | 60/64 | 52/64 | 60/64 |
| CLI6 候选 | 64/64 | 64/64 | 60/64 | 52/64 | 60/64 |
| JADX | 60/64 | 60/60 已编译项 | 60/60 已编译项 | 60/60 已编译项 | 60/60 已编译项 |

API 对照检查字段泛型类型及变量身份、声明方法及方法变量、类变量及 bounds；JADX 对照先去除反编译包名前缀 `defpackage.`。JADX 的四个 `InstanceRawLocal` 输入未能编译，故 API 只比较 60 个可用项。CLI6 与接受的 baseline 在 64 个输入的这些 API 分类上没有差异。原始源码、Probe 编译及运行均为 64/64。CLI6 输出使用显式空 classpath 和空 sourcepath 编译，再只以每项新生成的 classes 目录运行 `-Xverify:all`，编译和运行均为 64/64；候选行为与原始 Probe 的 7 行行为记录一致。

## 可复核记录

- `commands.tsv` 有 842 行：710 个实际子进程和 132 个 source-header 合成检查。合成检查是 runner 内部检查，不计入实际子进程。
- `subprocesses.jsonl` 保存每个实际命令的 argv、退出码、stdout/stderr 路径和哈希；`subprocess-logs/` 保存两条输出流，包括空流。工具版本探测也保留双流；工具清单包括 59 个 CLI/JADX 工具文件和 8 个 JDK 工具版本查询。
- CLI6 编译显式指定空 classpath/sourcepath；候选运行只用新生成 classes，并带 `-Xverify:all`，不借用输入 JAR 或既有类文件。回放后没有遗留 `.class` 文件。
- `outer-preflight-metadata.json`、`outer-run-metadata.json`、`versions/` 和 `result-file-hashes.tsv` 记录工具身份、输入哈希前后核验、所有实际子进程及结果文件哈希。只读独立核验脚本副本为仓库 `results/root-input-audit/verify-raw-replay-v6.py`；其输出由 root 另行生成。
