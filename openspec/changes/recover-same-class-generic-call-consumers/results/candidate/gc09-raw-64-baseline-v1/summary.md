# GC09 raw-64 accepted-baseline replay

本次只重放已验收 baseline；候选 CLI 未运行。
CLI：`/private/tmp/jarde-raw-receiver-final-v3-cli`；SHA-256：`3f75dab495c9753ec2c3f1bd0ac91665ed42f8b877eea65cb7c69a6879448f70`。标签：`accepted-baseline`。
输入：64 行、16 个族、64 个冻结输入 JAR；JAR 哈希与启动前快照匹配：True。

| 阶段 | 通过 / 总数 | 失败 |
| --- | ---: | ---: |
| 原始源码完整编译 | 64/64 | 0 |
| 原始 Probe 编译 | 64/64 | 0 |
| baseline class-source | 68/68 个物理输出 | 0 |
| baseline self-describing source header | 68/68 | 0 |
| baseline 完整类编译 | 64/64 | 0 |
| JADX 完整类编译 | 60/64 | 4 |
| 原始源码 `-Xverify:all` 运行 | 64/64 | 0 |
| baseline `-Xverify:all` 运行 | 64/64 | 0 |
| JADX `-Xverify:all` 运行 | 60/60 | 0 |

行为与 API 对照分别统计。API 匹配比较字段类型及变量身份、声明方法及其方法变量、类变量及 bounds；JADX 只对类名去掉 `defpackage.` 包装；未生成运行结果的输入计为 unavailable，不算匹配。

| Flavor | 行为匹配 | 字段泛型反射 | 方法 API | 类 API/bounds |
| --- | ---: | ---: | ---: | ---: |
| baseline | 64/64 | 60/64 | 52/64 | 60/64 |
| JADX | 60/64 | 60/64 | 60/64 | 60/64 |

API 差异族：baseline 字段为 `InstanceRawLocal`（四种 JDK/debug 组合）；方法为 `TypedReceiver`、`ShadowMethodT`、`MultiFormalRawParam`（各四种组合）；类 API/bounds 为 `RawOwnerChild`（四种组合）。JADX 在成功编译的 60 行中三类 API 均与原始 Probe 匹配；`InstanceRawLocal` 的四行未生成运行结果。

失败 stderr：JADX 的 `InstanceRawLocal` 在 4 个 JDK/debug 组合全部因 `Object` 无法赋给 `T` 而编译失败，诊断原文见 `failures/`。baseline 和 original 没有失败命令。

输入源哈希与启动前快照一致：True；runtime 命令均带 `-Xverify:all` 且未将 JAR 放入 classpath：True。结果目录中的 `.class` 文件数：0。
实际 runner 快照：`replay-executed.py`；SHA-256：`2c9b8233f41f13a483fa252a898f81e4602b9c7b3a0308cbc95bcd42162dc3a2`。外层命令和工具哈希见 `outer-run-metadata.json`；runner 命令表见 `commands.tsv`；JADX/JDK 工具哈希见 `versions/tool-hashes.tsv`。

这里的 baseline 结果仅作为此次 raw-64 回归的新对照，不以旧历史计数替代本次实测。
