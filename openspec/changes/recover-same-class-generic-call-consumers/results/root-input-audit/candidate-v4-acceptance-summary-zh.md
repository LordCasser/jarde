# 固定 CLI4 的 140+4 全量重放记录

本报告按固定验收 runner 的原输入与原判据运行；它记录这次重放的实际结果，不代表泛型调用片已全部完成。

| 项目 | 本次证据 |
| --- | --- |
| CLI | `/private/tmp/jarde-generic-calls-candidate-v4-cli`，SHA-256 `f6fe20c6dfb4be1673ba58e4536b4f8b1da38efe1ee991285b77dbd429717525` |
| 固定验收脚本 | `candidate-acceptance-replay.py` SHA-256 `a19895e70ef306a3024132a3da7c07cf891ea87e5ef0e25e01413989514ce836`；实际副本位于 `candidate/candidate-v4/candidate-acceptance-replay-executed.py`，重放后 hash 一致 |
| 通用矩阵 / nested runner | 实际副本 hash 分别为 `6b102b7c16e0bc318d42d3fbd21a32d51bd3bbeeeb72c8f66722808c09f28960` 与 `fb0235563fe9cb68df7d362997a1bd601928f9335d78af01326ddf1282d604f6`；通用runner与nested runner的原始脚本hash分别为 `e3a8a3df2953833237fdddc9cd176acb100f60a086e9adad521434fb1261f368`、`fb0235563fe9cb68df7d362997a1bd601928f9335d78af01326ddf1282d604f6` |
| 输入范围 | 冻结主矩阵 140 个唯一输入，加 nested-call 独立四腿，共 144 输入腿；产出 148 份物理 class-source |
| 输出完整性 | 外层 `run-manifest.json` 记录 3,836 个输出文件，重新核验所有记录的大小和 SHA-256 均匹配，integrity errors 为 0；主矩阵 manifest 3,793 个文件、nested manifest 34 个文件也逐项 hash 核验无误；两个runner退出码均为 0 |
| 完整编译 / Probe / 行为 | 候选均为 144/144；Probe 按 `-Xverify:all` 使用独立生成 classes，原输入 jar 未作为 classpath。保存了输入源、输入jar、原/候选Probe源码及其stdout/stderr与hash |
| 完整反射匹配 | 96/144；其余差异没有被编译或行为通过覆盖 |
| 现有固定判据分类 | feature-recovery pass 100 条；拒绝控制 pass 8 条；boundary-control pass 8 条；另外七项控制按原 runner 仍为 `needs-root-review` |
| 主矩阵 gate | GC-01、02、04、06、07 为 pass；GC-03、05、08 为 needs-review（包含七项待 root 裁定控制）。GC-09 和 GC-10 仍需独立回归/非矩阵门禁 |
| 汇总裁定 | `matrix_scope_decision=needs-review`；overall acceptance 仍 pending GC-09 / GC-10。没有把本次144行结果当作全片完成 |

固定输入出处为 [source-snapshot-v33.json](../local-gates/source-snapshot-v33.json)（SHA-256 `39759cccd93ac50ebff0fc6c3406d0ac46b596b2a07899a0bc1443701f10650c`）及 [source-snapshot-v33.tar.gz](../local-gates/source-snapshot-v33.tar.gz)（SHA-256 `36768f4551ec2836b9e81a661df3571906bc7acec304a659bb4964b4899e0082`）。其中 `src/class_source.rs`、`src/facade.rs`、`crates/jarde-java/src/report.rs`、`tests/same_class_generic_call_consumers.rs`、`tests/class_scope_constructor_projection.rs`、`tests/same_class_generic_binding.rs` 六个文件的当前 SHA-256 均与 v33 JSON 中记录相符。两条JDK腿及debug/no-debug参数按冻结 runner 执行；JDK的javac/java路径和hash、每条命令及其 stdout/stderr 路径和hash保存在主矩阵 manifest，所有输入与输出源、Probe、编译 classes 的hash也在对应记录中。

CLI4 主矩阵结果与先前 candidate-v3 主矩阵逐输入比较：candidate-v3 [`manifest.json`](../candidate/candidate-v3/gc01-08-140/manifest.json) SHA-256 `c8c4fcb4cc0d9d87f0975d406806c04dddb29e688534f16017029b0143acbe5b`；candidate-v4 [`manifest.json`](../candidate/candidate-v4/gc01-08-140/manifest.json) SHA-256 `e3a8a3df2953833237fdddc9cd176acb100f60a086e9adad521434fb1261f368`。140/140 输入 jar hash、输入源码 hash、CLI 发射源码 hash、候选编译 class hash、候选 Probe stdout hash、行为行及反射匹配标志全部一致。因此这组140个主矩阵样例未体现 v3→v4 结果差异。`VoidDirect` 和 `BoundOverload` 本轮在原固定判据下各为4/4 feature pass，完整反射各4/4；该结果只记录当前CLI4固定矩阵事实，不外推到其他输入或所有非矩阵门禁。

逐项验收汇总为 [acceptance-summary.md](../candidate/candidate-v4/acceptance-summary.md)（SHA-256 `36d01b2a3c4e2b8fd9a4266ba41b616b0b7732fc5a14026a23e30c22ced2a3c2`），原始结构化结果为 [acceptance-summary.json](../candidate/candidate-v4/acceptance-summary.json)。完整控制事实另存为 [candidate-v4-control-facts.json](candidate-v4-control-facts.json) 和 [candidate-v4-control-facts-zh.md](candidate-v4-control-facts-zh.md)；七项 CONTROL_REVIEW_ONLY 保持待 root 裁定，BridgeUnknown 与 IncompleteSite 仍作为正控制单列。
