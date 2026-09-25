# 非泛型成员构造调用：Root 独立验收（2026-09-25）

## 裁决

本变更在 class-source 的选定物理环境中，仅对公开、非泛型、可独立重编的成员类调用呈现 `outer.new Inner(args)`。目标类和外层类各自的 `InnerClasses` 条目、构造器的隐式外层首参/捕获字段、调用点同一 SSA 限定值、早于普通实参的准确空值检查与同一异常处理范围必须同时成立。`$` 名称只缩小读取范围；普通 method-only 请求没有目标交接时继续拒绝。实现复用现有 reader、环境选择、`new@1` 和 `New` AST，不新增通用 IR/pass。物理首参仍留在 `NewRecord.arguments`，只从源码参数和 descriptor 位置约束中剔除。

JADX 1.5.6 的 `ConstructorVisitor` 折叠分配/构造，`ClassModifier` 给合成首参加 `SKIP_FIRST_ARG`，`InsnGen` 以该标志和类型相等门拼限定创建。本实现借用其“物理前缀与源码实参分开”顺序，但不把标志、类型或可编译性当作调用点证明。[完整检查形状的身份错形](../../evidence/java-syntax-2026-09-25/inner-generic-instance-constructor/simple-member/invalid-controls/byte-variants/analysis.md)在 `java -Xverify:all` 下原 class 的 `actual-null` 输出为 `null:A`，JADX 重编为 `null:`，而 Jarde 直接命中 `jre_new_member_shape` 的 SSA 副本身份门并保留 fallback。另一个只篡改外层 class 成员条目的变体在 JVM 中照常运行，但以该外层 class 重编 `outer.new Inner(...)` 会找不到 `Inner`；这使外层选定定义的交叉核验成为必要条件。

## 三方运行与重编

冻结的 [`UseInner` 正例](../../evidence/java-syntax-2026-09-25/inner-generic-instance-constructor/simple-member/analysis.md)原/JADX/Jarde 在 Java 8 下均重编成功，`-Xverify:all` 同为 `7:IA`、`null:I`。Jarde 的[整类报告](../../evidence/java-syntax-2026-09-25/inner-generic-instance-constructor/simple-member/invalid-controls/byte-variants/after-jarde-full-all.json)对 `make` 为 `java/structured`，生成 `return arg0.new Inner(nested.SimpleOuter.mark("A", arg1));`；[调用方重编结果](../../evidence/java-syntax-2026-09-25/inner-generic-instance-constructor/simple-member/invalid-controls/byte-variants/after-jarde-full-javac.txt)退出 0，使用原成员类作为依赖的[运行结果](../../evidence/java-syntax-2026-09-25/inner-generic-instance-constructor/simple-member/invalid-controls/byte-variants/after-jarde-full-run.txt)与原 JVM 一致。这里不宣称整个 jar 的成员声明/泛型投影已完成。

[嵌套与前置效果](../../evidence/java-syntax-2026-09-25/inner-generic-instance-constructor/simple-member/negative-controls/after-jarde-NegativeUse.json)的 `nestedEffects`、`preEffect` 同为 `structured`，分别呈现 `arg0.new Inner(mark("A", mark("B", arg1)))` 和先执行 `mark("P", arg1)` 再限定创建；[Java 8 重编](../../evidence/java-syntax-2026-09-25/inner-generic-instance-constructor/simple-member/negative-controls/after-jarde-javac.txt)退出 0，[三行输出](../../evidence/java-syntax-2026-09-25/inner-generic-instance-constructor/simple-member/negative-controls/after-jarde-run.txt)与原/JADX 逐字相同：`ok:BA`、`null:`、`pre-null:P`。身份与迟到检查的两个源码绿侧也各以原依赖重编退出 0、运行逐字对齐；相应 `after-jarde-identity-source*`、`after-jarde-late-source*` 与五个 verifier-valid 错形的 class、SHA、`javap`、原/JADX/Jarde 结果均见[字节负控目录](../../evidence/java-syntax-2026-09-25/inner-generic-instance-constructor/simple-member/invalid-controls/byte-variants/analysis.md)。

缺失目标、目标自身关系错、外层关系错均未交付证明，完整 Jarde 报告是 `fallback`、`new@1` 0 呈现；两个身份错形和晚检查亦拒绝。强身份错形保留准确 `aload; dup; requireNonNull; pop`，因构造器首参不是被检查值的 SSA 副本而拒绝，故不只是早期 opcode 形状门碰巧挡住。额外在直接 proof-unit 中验证了外层静态类型错配、构造站点跨 handler 半开边界与无目标交接的拒绝。

## 报告与停止

同一 `UseInner.make` 的 [essential](../../evidence/java-syntax-2026-09-25/inner-generic-instance-constructor/simple-member/invalid-controls/byte-variants/after-jarde-full.json)、[all](../../evidence/java-syntax-2026-09-25/inner-generic-instance-constructor/simple-member/invalid-controls/byte-variants/after-jarde-full-all.json)、[0..4 range](../../evidence/java-syntax-2026-09-25/inner-generic-instance-constructor/simple-member/invalid-controls/byte-variants/after-jarde-full-range.json) 都完整完成，方法正文和 `structured` 判定逐字相同。all/range 的 `new@1` 记录均为 1 个已呈现站点、物理实参 BCI `[5,13]`；all 的 source map 给限定值 BCI 4、普通参数 BCI 13、构造根 BCI 16 及派生的 0/3/5/6/9。`class_headers=2` 的[目标读取预算控制](../../evidence/java-syntax-2026-09-25/inner-generic-instance-constructor/simple-member/invalid-controls/byte-variants/after-jarde-full-target-budget.json)报告 `partial/budget_exceeded`，无 `new Inner` 正文；取消也由 facade 定向单测覆盖。

## 定向门禁与资源

- `cargo test --locked -q -p jarde-java --lib`：180/180；`cargo test --locked -q -p jarde --lib`：23/23。
- `cargo test --locked -q -p jarde --test class_source`：47/47；`p3_new_value`：4/4；`p3_ordinary_new_invokes`：2/2；`d1_evidence_selection`：8/8。后者还保持普通 `new` 内独立 `void` 效果的拒绝边界。
- `cargo fmt --all -- --check`、`git diff --check`、`openspec validate recover-proved-member-inner-construction --strict` 均通过。
- 严格 Clippy 在当前共享树的 enumswitch、region、report、build、reuse 共 17 条既有告警上失败；本变更新增的两个 `init.rs` 参数数量告警已通过统一只读 `ConstructionFacts` 消除。`jarde-java/tests/p3_patterns.rs:389` 的既有 `recover_for_class_source` 两参/三参不一致阻止该测试目标编译；`jarde-cli --test class_source_cli` 为 15/16，唯一失败是历史 `finallyPath` 回退来源断言期待单独 `@bytecode 9`，当前其它在途 Region 改动给 `@bytecode 9 10 13 14`。这两项不属成员构造实现，未混改。
- 本次 Root 私有 Cargo target `/tmp/jarde-member-root-target` 已由 `cargo clean --target-dir` 清理，Cargo 报告移除 12,475 个文件、4.4 GiB；受控运行使用的 `/tmp/jarde-negative-member.jar` 亦删除。项目目录没有留下 Cargo `target`，最后 `df -h .` 显示约 82 GiB 可用。
