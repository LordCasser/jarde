# GC09 raw-64 候选回放：固定 CLI4

本目录只记录 GC09 的 64 项 raw-64 回放，**不是整个 OpenSpec change 的验收结论**；完整变更仍需结合其他矩阵和门禁结果。

## 固定对象与输入

- 候选标签：`gc09-raw-64-candidate-v4-cli4`。
- 固定候选 CLI：`/private/tmp/jarde-generic-calls-candidate-v4-cli`，SHA-256：`f6fe20c6dfb4be1673ba58e4536b4f8b1da38efe1ee991285b77dbd429717525`。
- 固定源码快照：v33，归档 SHA-256：`36768f4551ec2836b9e81a661df3571906bc7acec304a659bb4964b4899e0082`。
- 回放外层 adapter 是**版本 2**（SHA-256：`e170ad879448687dfe972f615dfd78a56406d5b5bc1b9a841a526fdee2633fb3`）；这表示 adapter 版本，候选本身仍是 CLI4。实际运行的冻结 runner `replay-executed.py` SHA-256：`2c9b8233f41f13a483fa252a898f81e4602b9c7b3a0308cbc95bcd42162dc3a2`。
- 输入为 16 个 fixture 家族在 JDK 8/JDK 23、debug/no-debug 下的 64 个组合。冻结输入清单和哈希见 `versions/frozen-input-jar-hashes.tsv`；外层 preflight 的逐项快照见 `outer-preflight-metadata.json`。
- 18 个 runner Java 源、23 个 fixture Java 源和 64 个输入 JAR 的运行前后哈希均未变化。工具版本及 59 个工具/运行时文件哈希见 `versions/tool-hashes.tsv`。

## 回放结果

| 阶段 | 通过 / 总数 | 说明 |
| --- | ---: | --- |
| 原始 fixture 编译 | 64/64 | 两个 JDK、两种 debug 设置 |
| 原始 Probe 编译与 `-Xverify:all` 运行 | 64/64、64/64 | 原始行为对照 |
| CLI4 完整类编译 | 64/64 | 空 classpath/sourcepath 隔离 |
| CLI4 `-Xverify:all` 运行 | 64/64 | 行为与原始 Probe 匹配 64/64 |
| JADX 完整类编译 | 60/64 | 4 项编译失败，诊断保存在 `failures/` |
| JADX `-Xverify:all` 运行 | 60/60 | 仅统计成功编译的输入 |

API 反射比较与运行行为分别统计。字段比较检查泛型类型及变量身份，方法比较检查声明方法和方法变量，类比较检查类变量及 bounds。原始对照自身为 64/64。候选 CLI4 的字段 API 为 60/64、方法 API 为 52/64、类 API/bounds 为 60/64；JADX 分别为 60/64、60/64、60/64（4 个未成功编译的输入按 unavailable 处理）。候选 API 结果没有相对已验收 baseline 新增的回退；逐行对照见 `runtime-comparisons.tsv`，独立核验记录见仓库内 `results/root-input-audit/` 下的 v4 verifier JSON。baseline 路径固定为 `gc09-raw-64-baseline-v1`，其原始报告保留在同级目录，candidate-v3 结果也未覆盖。

## 可复核记录

- `commands.tsv` 有 842 行命令记录：710 个实际子进程以及 132 个 source-header 合成检查。实际子进程的完整 argv、退出码和 stdout/stderr 哈希见 `subprocesses.jsonl`；每个实际子进程的两个流都保存到 `subprocess-logs/`，包括空流。
- `jdk-version-logs/` 保存 8 个版本查询工具在两个 JDK 上的 stdout/stderr；JADX、CLI 和工具文件哈希见 `versions/`。
- `result-file-hashes.tsv` 列出本目录结果文件的 SHA-256；`outer-run-metadata.json` 保留固定 argv、adapter 哈希、退出码和文件哈希。
- root 独立 verifier 对 64 项、842 条命令、710 个实际进程、132 个合成检查和 59 个工具哈希完成核对，结果为通过。临时编译产物未保留，本目录 `.class` 文件数为 0。
