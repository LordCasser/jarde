# 任务 2.2 验收：已证枚举的类级源码投影

投影只发生在类源码正文：对已证两常量组，额外要求原始 `<clinit>` 在完整前缀后只有 BCI 对齐的正常 `return`，同次结构化侧车恰为三项字段写加该 `return`，且构造器 `Signature`（若存在）在同次解析后恰为源码尾参 `(I)V`。`Stage` 写成 `START(4), FINISH(9);`，构造器只声明一次 `int` 字段存储。物理字段、方法、outcome 和 Signature 拒绝 marker 仍留在 report/JSON；只有同次结构化检查证实的 enum 注入参数 erasure marker 不复制到投影正文。构造器的其它 marker 会拒绝投影。`Measure` 有用户静态后缀，保持原正文完整。

Root 用最终 CLI（SHA-256 `0803924735e2e6c7c1726820e33b7c1c70cf5886a321456b8a234e70c0c53c33`）独立验收：Stage 默认和 `--evidence all` 的文本 SHA 相同（`edc4e0bf9d78654f3b16247a709d43be717261edeaaf4362c81f80c583a5d5d7`）；Stage 的完整类和 `StageRunner` 以 `javac --release 8` 编译，原 class、JADX 重编类和 Jarde 投影 Stage 均经 `java -Xverify:all`，Runner 每行输出相同的 `enum declaration checks: PASS`。JSON 仍有原始 4 个字段和 6 个方法，保留 `$VALUES`、`$values()`、`<clinit>` 的物理身份。Measure 正文与冻结 CLI 相同，仍保留 `totalUnits = sumUnits()`；name/ordinal、构造器/前缀、values/valueOf 变体和额外 `$VALUES` 读取均未误投影。当前普通 `StageRunner` 源码中的 `main` fallback 与冻结 2.1 CLI 相同，属于已有方法恢复结果。

本地复验通过：

- `cargo test -p jarde --lib enum_constants::tests --target-dir /tmp/jarde-enum-proof-target`：9/9。
- `cargo test -p jarde --test class_source --target-dir /tmp/jarde-enum-proof-target`：47/47。
- `cargo test -p jarde --test interface_initializer_proof --target-dir /tmp/jarde-enum-proof-target`：4/4。
- `cargo fmt --all -- --check`、`cargo clippy -p jarde --lib --no-deps --target-dir /tmp/jarde-enum-proof-target`、`openspec validate --strict recover-proved-enum-constants` 均通过。Clippy 只报告已有的 10 条 `jarde` 告警；2.2 未增加告警。
