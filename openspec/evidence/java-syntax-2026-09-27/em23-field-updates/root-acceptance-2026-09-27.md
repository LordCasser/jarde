# EM-23 主线独立验收

root 在合并 EM-21 与 EM-23 后重建 CLI（SHA-256 `c33ed575a4603ff3ffebe9104486df47ea8fec2501525849d7090939179f033d`），对固定 JADX `2fb1b16386941660fda07e9017285aec40fcb37f` 的 `replay.py` 独立三方重放。原 class、JADX、Jarde 完整 Java 8 源码全部重编并通过 `java -Xverify:all`；输出同为 `2:0:A_B_` 和 `5:4`。Jarde 输出已证实例 `this.instanceField++` 与静态 `Updates.staticField--`，字符串拼接链仍按 EM-27 单独处理。生成源码 SHA-256 `b486101f5b7d86403cffd950f20436d2a0587abb6c076e4cb79f7814d0d0fe95`，与实现代理的验收源码逐字节相同；独立摘要在 [root-replay/](root-replay/)。

代码审查确认新证明只接纳同块末尾、无 handler、准确读写同一本类非 volatile/non-final `int` 字段、常量 1、唯一 SSA 消费者的两个字节码窗口；减法证明已限定旧字段值位于左操作数。`cargo test -p jarde-java`、`cargo test -p jarde --test em23_field_unit_updates`、`cargo test -p jarde --test class_source`、`cargo check --workspace`、格式检查及本 OpenSpec 严格校验通过。嵌套接收者、结果被使用的前/后缀和字符串 `+=` 仍待单独验收。
