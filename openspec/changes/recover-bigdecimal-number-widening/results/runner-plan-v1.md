# BigDecimal 候选回放与独立核验计划

本计划对应同目录两个新脚本。脚本只准备、尚未执行；CLI 与 metadata 等 root 冻结后再通过参数传入。

## 调用与输出

候选回放：

```text
python3 replay-candidate-root-v1.py <FROZEN_CLI> <CLI_METADATA_JSON> <ABSOLUTE_NEW_OUTPUT_DIR>
```

输出目录必须是绝对路径且事先不存在。脚本先核验 CLI metadata 的二进制 hash 和精确五个源码身份（`init.rs`、`report.rs`、`build.rs`、`src/class_source.rs`、`Cargo.lock`），再确认所有历史输入、真实 JDK 工具、冻结 class/source、原始双流和 wildcard 完成记录未变。只把这次 CLI 的每个 class-source JSON 原始流写入新目录；全部成员源码在显式空 classpath/sourcepath 下编译，运行时仅使用该腿新生成的 classes 并带 `-Xverify:all`。original/JADX 结果只按冻结哈希引用，不重新执行、不称 fresh。

独立验收：

```text
python3 verify-candidate-root-v1.py <ABSOLUTE_REPLAY_OUTPUT_DIR> <ABSOLUTE_NEW_VERIFICATION_JSON>
```

核验器独立校验回放闭集、所有命令双流 hash、24 个输入 jar/class 闭集、报告成员闭集、全源编译命令、隔离 runtime、实际 candidate source-map 和 frozen classfile。它不执行 Java/CLI，也不信任 runner 的 `candidate_success` 摘要。

## 分母与新判据

固定分母为既有 22 条旧腿（完整 factory/direct 4 腿与 legacy 18 矩阵）加 2 条 BigDecimal 腿，共 24 条。额外 NumberArgument fixture 本轮不加入分母，待 root 冻结输入后另行安排。

两条 BigDecimal 腿沿用已归档 Main-only jar、真实 javac8/javac23 工具及原始运行流。除完整源码无 fallback、空 CP/SP 编译成功、只新 classes 验证运行且 exit/stdout/stderr 与原始字节一致外，还要求：

- 独立解析输入 `Main.main([Ljava/lang/String;)V` 的物理 BCI 1/6/9/12/15/16/17/28/29/34/40/43/46：`Number[]` 分配、`BigDecimal` 分配/dup/构造/aastore、局部写、`System.out`、arraylength、三次 StringBuilder.append、toString 和 println 的常量池绑定保持准确。
- 候选 main 必须是 `structured` / `java`，没有 `@bytecode` 或 `jarde_refused_body`；source-map primary 与 derived 来源合并后包含上述全部 BCI。
- BigDecimal `NewRecord` 恰一条，presented 且对应物理 `(head, dup, constructor) = (6, 9, 12)`。正文实际呈现 `Number[]`、一次 BigDecimal 构造、输出字段及数组长度/索引使用、三次 append 和一次 toString。concat 优化可以保留 refusal warning，只要完整 Java 正文和行为/来源证据通过。
- 两条 wildcard `collectionGridDirect` 仍须保留原成功 marker、精确声明与 raw array body；其它 22 条沿旧完整家族/来源/运行边界复验。

只有独立核验 24/24 均成功才输出完成标记。若 BigDecimal 或旧腿失败，runner 仍保留所有实际 raw 记录与真实分母，核验器拒绝把它记成通过。此轮不运行目标、Java、CLI、Cargo、Git 或其他门禁。
