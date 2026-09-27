# EM-21 主线独立验收

主线同时包含 `inline-proved-this-local-alias` 与 EM-23 字段单位更新后，root 使用本轮源码重新构建 CLI（SHA-256 `c33ed575a4603ff3ffebe9104486df47ea8fec2501525849d7090939179f033d`），重放固定 JADX `2fb1b16386941660fda07e9017285aec40fcb37f` 的 `replay.py fixed`。原 class、JADX、Jarde 完整源码均通过 Java 8 重编和 `java -Xverify:all`，四行输出逐字相同：`123:1`、`123:2`、`true:3`、`true:1`。修后的 Jarde 消去两个已证明只保存 `this` 的局部别名，分支可变来源仍保留局部；`output_bytes=1` 时不发布源码。

独立重放摘要在 [root-replay/](root-replay/)；本次 Jarde 源码 SHA-256 为 `32976e641d22bf442309b2dfc544e82a8cf5bf4707f05d10138e44032bc94f85`。它与实现代理保存的源码只有同批 EM-23 在 `touch()` 中把已证 `this.touches += 1` 写成 `this.touches++` 的预期差异，运行结果不变。`cargo test -p jarde-java`、`cargo test -p jarde --test class_source`、`cargo check --workspace`、格式检查及本 OpenSpec 严格校验通过。验收仅覆盖报告中的当前实例别名闭合形态；其它消费者和槽复用仍需独立证明。
