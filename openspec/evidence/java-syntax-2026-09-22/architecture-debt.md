# 独立架构债务：字段自增的整段文本节点

`crates/jarde-java/src/build.rs::increment_statement` 将已证明的简单 `this.n++` / `++this.n` 整段写进 `ExprKind::Local`。它保留了现有窄形状的行为，却绕过子表达式自身的括号、类型与来源结构；把它直接扩到调用接收者或数组下标会使真实生产者无法按节点追踪。

`recover-postfix-lvalue-values` 只为新增的字段/数组后置旧值返回链使用真实后置表达式，不同时迁移已有简单前置/后置字段规则。后续独立清偿应先冻结已有局部接收者 `n++`、`++n` 的完整类执行、来源和预算，再将这两个已证明形状统一为结构化更新目标，并证明不会与字段/数组复合赋值及新后置规则双重认领。此项尚未实施，也不算当前语法修复的验收条件。

## 部分引用正文可能碰巧可编译但执行不等价

`compound-assignments/post-fix-fixture-replay/` 的六个 JVM 合法 tick-gap 样本触发了保守来源引用，方法正文不再写 `+=`，但仍包含已经恢复的前缀效果。整类恰好通过 javac；例如 `field-gap-before-dup` 原 class 运行得 `value=9:select=1:rhs=2`，Jarde 部分正文运行得 `value=7:select=1:rhs=1`。输出顶部和方法中已有“非完整恢复”标记，故当前报告没有声称行为等价；**javac 成功本身也不能证明恢复完成**。

这一风险属于类源码部分恢复结果的使用/状态契约，不属于复合 `+=` 正例证明。后续应独立审计其他 fallback 与 class-source 组合，决定如何使下游不能把带未恢复效果的偶然可编译正文误认作等价程序；验收需要比较报告质量/coverage、真实 BCI 引用与原 class/JADX 执行，而不能只看编译返回码。本轮只记录，不在 `recover-compound-lvalue-updates` 中扩写通用 fallback 策略。

## 语料指纹清单的独立漂移

2026-09-25 只读盘点按 `tests/p5_corpus_fingerprint.rs` 的实际遍历/排除规则发现 121 个应登记而未登记的 fixture 文件：`p3-conditional-values` 64、`p3-nested-array-initializers` 10、`p3-typed-catch-boundary-return` 4、`proved-java-structure` 43；`fuzz/corpus` 没有漏项。`p3-local-postfix-array-elements` 自己的八项已全部登记，路径/长度与磁盘 8/8 对齐，因此不把其它目录的清单修复混进后置自增语法 change。后续单独核对 121 个文件是否都是预期永久 fixture，再运行仓库既有的 `cargo test --test p5_corpus_fingerprint --locked -- --ignored regenerate_corpus_fingerprint` 并审查仅新增对应条目的 diff，最后运行普通清单测试。遍历算法与排除规则目前无证据需要修改。

## 恢复层严格 Clippy 的共享门禁告警

2026-09-25 在泛型成员调用点独立验收时，`cargo clippy -p jarde -p jarde-java --lib --locked -- -D warnings` 仍报 17 项共享树告警：`enumswitch.rs` 两处无用 `i64::from`；`region.rs` 两处参数过多、一处复杂返回类型；`report.rs` 的泛型构造候选与主恢复函数参数过多、四处 `Option::as_deref_mut` 冗余及一处可合并 `if`；`build.rs` 两处复杂返回类型、两处参数过多；`reuse.rs` 一处复杂返回类型。泛型成员调用点本次增加的 `GenericReturnValue` 体积和 `generic_return_candidate` 参数数告警已在该变更内消除，根验收后的剩余清单为上述 17 项。它们不改 Java 语义但阻止严格门禁通过，应另行归因并按所属变更逐项清偿，不把 enum switch/Region/复用规划器的整理混进当前语法修复。

## reader fixture 人数钉值与跨模块 Clippy

2026-09-25 的接口 `super` 独立验收运行 `cargo test -p jarde-reader --lib --locked`：176 项通过，`repository_class_fixtures_validate_without_false_target_rejections` 一项因 fixture 人数断言失败，实测 `(310,1645,144,880,8)`，旧钉值 `(164,1152,98,381,8)`。其中三类、三个 Code 来自本次新增的多父 default 冲突控制，新增前实测为 `(307,1642,144,880,8)`；其余漂移不能通过修改接口语法代码来掩盖。应先按 fixture 来源核对新增人口与指纹登记，再单独更新 census 断言。

同轮 `cargo clippy -p jarde --lib --locked` 以警告模式可完成，除上述 17 项 jarde-java 共享告警外还显示 jarde 的 11 项旧告警：`class_source.rs` 四个参数过多，`facade.rs` 一处可合并 `if`、三处复杂返回类型及三处参数过多。新增接口闭包/目标证明所在的 `facade.rs` 约 4132–4600 行没有可归因 Clippy 告警。严格 `-D warnings` 因共享告警失败；独立清偿时应按函数归属处理，不把这些重构混入接口 `super` 合法性修复。
