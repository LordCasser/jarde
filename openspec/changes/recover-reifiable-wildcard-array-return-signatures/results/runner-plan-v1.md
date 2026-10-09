# 无界 wildcard 数组候选回放记录

候选 runner 与 root gate 最初只为 root 审阅而准备，准备阶段没有运行候选回放、原程序/JADX、Java、Cargo 或 Git。此后 root 已用冻结 CLI 实际完成候选回放和独立 verifier；下文分开记录这两阶段，避免把准备时的边界误写成当前状态。

候选 runner 接收冻结 CLI、对应 metadata JSON 和一个必须为绝对路径且尚不存在的输出目录。开始前会核对 CLI 路径及 SHA，并要求 metadata 精确列出五个源码身份且其 hash 与当前 workspace 相同：`crates/jarde-java/src/init.rs`、`crates/jarde-java/src/report.rs`、`crates/jarde-java/src/build.rs`、仓库根目录 `src/class_source.rs` 和 `Cargo.lock`。

候选回放分母为 24 腿：完整 factory/direct 六类 fixture 4 腿、旧 legacy 矩阵 18 腿、numeric 两类 2 腿。每腿保留完整原 class 集；runner 校验并复制冻结输入 JAR，对每个类完整渲染并保存 JSON stream 和生成源码，用冻结 compiler flags、显式空 classpath/sourcepath 编译全部生成源码，然后只用新编译的 classes 目录执行 `-Xverify:all`。numeric 入口类取自冻结 case：`ConstructorPrimitiveConversionControls`。记录包含实际命令、原始 stream 和 exit、工具身份、输入 class entries、生成源码 hash、markers、方法身份/descriptor/access flags、NewRecord 与 source map 摘要。旧原程序和旧 JADX 结果只复用已保存历史证据，本轮没有重新运行它们。

每条 direct 腿对 `collectionGridDirect` 单独记录原物理 descriptor 与 Signature、完整生成成员声明、marker/refusal 状态、恢复正文及 hash、quality/representation、source map、NewRecord 和完整生成源码路径/hash。完整 source 与原始 report JSON 同时保留在普通 case 证据中。runner 的 source/runtime replay success 只表示完整源码可重编、运行并与旧输出一致，不代表泛型 Signature 已恢复；泛型结论必须检查原 Signature、生成声明、marker 和正文。

本 change 的历史摘要 `baseline-root-v1.json` 指向 child v4 candidate manifest 和 root verification 文件，runner 会逐一核对它们的 SHA。摘要状态为 `baseline_frozen_signature_pending` 且 `fresh_execution: false`，只描述旧基线。child slice 的六类原程序/JADX audit 以及其余 24 腿的永久 Jarde/JADX 原始记录也都是历史证据。本轮历史分母为 factory 2/2、旧 18 腿矩阵 16/18、六类 direct 家族旧候选 0/2；两条已知 BigDecimal 失败保留在矩阵中。这些旧计数不代替新候选的结果。

root gate 沿用已有 20 GiB 进程组磁盘保护。启动前记录五个源码身份及 `tests/p3_heterogeneous_array_initializers.rs`、`tests/ordinary_generic_projection.rs`，并保存 argv、stdout/stderr、exit 和磁盘空间数据。root 可用唯一 label 传入 CLI build 命令；wrapper 为每个 label 新建目录并拒绝覆盖。

root 后续实际生成的 [候选回放 manifest](candidate-root-v1/manifest.json) 记录 24 腿全部完成、22/24 候选成功；使用的 candidate CLI SHA-256 为 `196bb3e1e1bd074bb851f363938951a6d2dea81d8e895cd67e1a2560b3c2f761`，两条 BigDecimal 已知失败仍在分母中。[独立 root verification](candidate-root-verification-v2.json) 通过 1166 项检查，状态为 `semantic_and_wildcard_signature_passed`，并将本 change 的 task 3.1 标记为 complete。它逐腿核对了原始 `Main.class` 的 `collectionGridDirect` Signature、完整参数化声明、成功证明 marker、保留 raw 类型的创建正文、来源 BCI 和行为输出。此处只记录该 slice 的 root 验收；没有据此宣称全仓 CI 已完成。
