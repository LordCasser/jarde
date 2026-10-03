# `recover-ctor-reorder-dispatch-guard` 的验收证据（2026-10-04）

修复[构造期虚分派回归](../README.md)：重排判据加单档前置——仅当 super 目标恰为
`java/lang/Object.<init>()V`（owner 与 descriptor 双匹配）时才把 pre-super 合成存组移到
super 之后呈现；任何其它 super 目标一律保持逐字节序与其诊断。落点
`crates/jarde-java/src/ctor_order.rs::present_prologue_first`（读 `operations.get(prologue.bci)`
的 `Operation::Invoke(CallTarget)`，不扩展 `Prologue` 结构），预算计费路径与判据收紧前一致。

## 三向验收（spec scenarios 对照）

| 向 | 证据 | 结果 |
| --- | --- | --- |
| (a) dispatch 反例不重排，且不产生"可编译且行为不同" | [repro-before-after.txt](repro-before-after.txt)；测试 `tests/ctor_reorder_dispatch_guard.rs::the_dispatch_fixture_keeps_the_capture_write_before_the_constructor_call`（断言 `val$captured` 写序早于 `super(`；若 javac 接受则重编运行输出必须等于原类输出） | 前腿（主线 9eaf7b2c）：`super();` 在前 + 重编运行 `observed=null`/`visibleDuringSuper=false`；后腿：verbatim 序 + javac 拒绝（灵活构造器，exit 1） |
| (b) Object super 正例逐字不变 | [c1c2-zero-regression.txt](c1c2-zero-regression.txt)；既有测试未改一字全绿：`tests/recover_synthetic_ctor_super_order.rs`（`before(child,"super();",write)` 序断言 + 家族重编运行基线 `25/6`、`10`）、`crates/jarde-java/tests/p3_patterns.rs` 合成捕获/enclosing/双捕获三正例 | C1/C1$1/C1$2/C1$Op/C2/C2$Inner 六类渲染逐字节一致（SHA 表）；javap 实证三者 super 均为 `Object."<init>":()V` |
| (c) 非 Object super 不重排 | 测试 `tests/ctor_reorder_dispatch_guard.rs::user_class_super_fixtures_return_to_the_compiler_s_byte_order`（anonymous-super-args、anonymous-capture 两冻结 fixture）+ `crates/jarde-java/tests/p3_patterns.rs::a_synthetic_capture_before_a_user_class_constructor_call_stays_where_the_bytecode_made_it`（生成器构造 `p/Base` super，无新 fixture 文件）；[verbatim-revert-behavior.txt](verbatim-revert-behavior.txt) | 三者均退回 verbatim，原类运行输出与各自登记一致，渲染重编 exit 1（响亮失败，与 2026-09-25/09-27 既有登记状态相同） |

## corpus 双腿扫描

[corpus-two-leg-scan.txt](corpus-two-leg-scan.txt)：`tests/fixtures/**/*.class` 共 465 文件，
基线二进制（9eaf7b2c）与守卫二进制各渲染一遍（stdout + exit status），`diff -r` 全量对照：
**仅 4 处差异，且全部是构造器内 `super(...)` 与捕获写入的相对次序**。
三处与 ../README.md 普查表一致；第四处 `anonymous-top-level/AnonymousTopLevel$1`（super
`Base(J)V`）是普查表遗漏的同形，其自身 2026-09-25 证据登记的正是 verbatim 序 + 重编退出 1，
守卫使其恢复登记状态。Object super 对照组（`AnonymousCaptureCases$Outer$1` 双捕获等）与其余
459 文件逐字节一致。

## 呈现文本对照

[renders-before/](renders-before/) 与 [renders-after/](renders-after/)：三处非 Object super
形（dispatch、super-args、capture $1）+ 一处 Object super 对照形（`Outer$1`）。前三处
`super(...)` 移回捕获写入之后；对照形保持 `super();` 首句不动。

## 遗留边界

不可编译是这些独立二进制名类视图的如实登记状态（`present-proved-java-structure` 2.10 的
立场）：本片只做"响亮失败优于静默错误"的过渡收敛；让该形可编译的终局解是 5.3 的类级匿名
语法（`new Base(...) { ... }` 由 javac 自行生成前置合成写入），不在本片（design 决策 5）。
