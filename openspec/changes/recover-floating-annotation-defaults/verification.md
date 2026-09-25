# 主代理验收

root 审阅 `MemberDefault` 和 `resolve_default`：F/D 仅按 `ElementConstantTag` 与相同类型的池项构造，字段 `ConstantValue` 路径没有新增 F/D 分支。有限值按原始整数位拆解成精确 Java 十六进制字面值；float 23 位尾数左移一位成为六个十六进制数字，double 52 位尾数用十三位；负零、次正规、最大有限值经真实 Java 8 重编译验证。只准入正 canonical NaN 和正负无穷的 Java 常量表达式，其它 NaN 原始位返回 `None`。数组仍沿用 `collect::<Option<Vec<_>>>()` 整段拒绝，没有新增 reader、方法 IR 或 JSON 默认值树。

冻结 CLI SHA-256 `f2fb24241c2692dff53060576e185eb282617a6881e81900140bb2eb62006544`。root 在原始夹具**复制目录**重新编译并重放三套完整类，四行反射 raw bits 原/JADX/Jarde 完全相同；八项边界与 F/D 数组四项也在修后原/Jarde 执行逐行相等。两份单池项异常 NaN class 均通过 JVM 全校验，Jarde 只省略不可忠实拼写的方法默认值；数组 payload NaN 则整段省略 `floats`、保留 `doubles`。完整证据、重放脚本及日志在 `../../evidence/java-syntax-2026-09-22/annotation-float-defaults/post-fix-root-replay/`，JSON `text` 与默认文本一致。

门禁：`p3_annotation_default` 8/8、`class_source` 16/16、`p5_corpus_fingerprint` 5 通过/1 ignored、reader 160/160；`cargo fmt --all -- --check`、`git diff --check`、`openspec validate --all --strict` 64/64 均通过。严格 Clippy 的根库及注解定向目标在排除既存 `region.rs:1736` 的 `type_complexity` 后通过；全目标 Clippy 受既有测试 feature 门控及无关测试 lint 影响，未称全仓全绿。`p5_bulk_corpus` 三项计数/分类基线漂移见 `../../evidence/java-syntax-2026-09-22/bulk-recovery-pin-drift/analysis.md`，未混入本 change。
