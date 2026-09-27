# DT-26：受证明的参数捕获 lambda

固定 JADX revision `2fb1b16386941660fda07e9017285aec40fcb37f` 的三个原始测试说明编译器生成的捕获 lambda body 不应作为源代码 helper 调用暴露。本切片单独冻结两个 Java 8 形态：[CaptureCases.java](CaptureCases.java) 中的 `x -> x + base` 捕获一个 `int` 参数，`() -> this.number() + delta` 捕获 `this` 和一个 `int` 参数；[Runner.java](Runner.java) 检查结果。

运行 [replay.py](replay.py) 会用 `javac --release 8 -g:none` 编译原始 fixture，以固定 JADX 和 Jarde 输出完整类源码，并分别与同一 Runner 一起重新编译。三方源码都通过 Java 8 编译与 `java -Xverify:all`，输出逐字为 `7:-1`。Jarde 类源码没有 `lambda$` helper 调用或声明，物理 helper 仍在类报告的方法记录中。固定的 `javap -c -p`、源码、三方文本、class/source/output SHA-256 和断言结果保存在 [outputs](outputs/)；连续两次运行的输出哈希相同。

捕获内联复用了 DT-25 的 helper 候选、body 证书、类级完整 use census 和原子提交。站点 capture operand 必须是当前完整 SSA 中直接加载的 `this`/`int` 参数；对该参数整个 enclosing Code 扫描 Store/iinc，写入、计算表达式、phi、未知来源和不完整 scan 都拒绝。静态 synthetic helper 将捕获参数先映射、SAM 参数后映射；实例 helper 要求 `REF_invokeSpecial`、`private synthetic` 非 static helper、直接 `this` receiver 和一个 `int` 参数。实例 body 只接受一个同类虚调用 `this.number()I` 与该参数的加法，因此调用仍在 SAM 被调用时执行。

类级诊断保留被拒绝的物理 helper 与站点位置；所有相关 helper 和站点成功 staging 以前不隐藏任何 helper。捕获来源证书的入口为 `report.rs::prove_lambda_helper_captures`，类级原子隐藏继续走 `facade.rs::lambda_helper_census_refusal` 及既有统一提交；只允许捕获站点对应的 static/instance helper flags。DT-26 之外的局部变量生命周期推断、重赋值/效果表达式、额外 helper 调用、复杂或异常 body 均保持拒绝；DT-27 方法引用、泛型 target 和其它捕获类型仍是独立差距。

验证：`cargo fmt --all --check`、`cargo check --workspace --all-features --locked`、`cargo test -p jarde-java --lib --locked`（220/220）、`cargo test -p jarde-java --test anonymous_allocation_candidates --locked`（5/5）、`cargo test --test p3_immediate_functional_receivers --locked`（16/16）和 `openspec validate recover-proved-captured-lambda-bodies --strict` 通过。三方 replay 连续两次输出 SHA 相同。当前基线的全 workspace test 在 `generic_constructor_projection::generic_constructor_projects_atomically_for_debug_and_no_debug_classes` 失败，独立复跑仍复现；root 已在后续 main commit `fdfc5ef0` 单独修复该债务，本变更没有触碰 generic constructor 路径。

root 将实现摘取到 `b4d554ad` 后，以 SHA-256 `6485f11da9316846e9eaa90dc8dae5e28cabe99e3aeecda5999895093bc4785c` 的本线 CLI 独立重放；`outputs/results.json` 及两份反编译源码哈希均与代理冻结结果相同。当前主线 `p3_immediate_functional_receivers` 聚焦测试复跑为 16/16，`cargo fmt --check` 与 OpenSpec strict 通过。
