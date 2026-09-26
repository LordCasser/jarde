# 2.3c 常量体整组合证验收

2026-09-26，root 审查 `enum_constants.rs`、`facade.rs` 与现有投影调用的类型分离，并独立执行 `CARGO_TARGET_DIR=/tmp/jarde-enum-root-23c-target cargo test -p jarde --lib enum_constant_body_relation_tests`：11/11 通过。代理执行 `cargo test -p jarde --lib`：70/70 通过；格式与 diff 检查通过。`openspec validate recover-proved-enum-constant-bodies --strict` 通过。

整组门复用已冻结的常量前缀、子类独占使用、构造链和结构化正文证明，再核对 `values`/`valueOf` 的精确 Code、`<clinit>` 在三次已证字段写入后的唯一 `return`、两常量与选定子类的一一对应，以及每个无 Code 抽象声明在各带体常量中的完整实现。`Op` 和 `Mixed` 的 `-g`/`-g:none` 正例得到私有 `Body` 组证明；缺抽象实现、改写标准 `values` 助手、字段/异常/额外使用等负例保持 `Refused` 与物理成员记录。普通组仍由原有 `Ordinary` 投影消费，`Plain` 不因本步骤放宽。

本步骤只发布私有整组证明，3.x 尚未把带体方法写入枚举常量的 `{ ... }`，所以不能把 `Body` 的 `Proved` 当作已交付完整 Java 类源码。验收 target 已清理。
