# DT-04 窄切片独立验收

根验收基于主线 `2061473f`（DT-04 两个实现提交已 cherry-pick），对照 `recover-proved-anonymous-inner-this` 的封闭证明范围。原始 fixture 由 `javac --release 8 -g:none` 重建并与冻结的两个 class SHA 一致；`replay.py --expect-jarde fixed` 在主线重新运行成功，生成的完整原始、JADX 与 Jarde 根源码都以 Java 8 编译，并在 `java -Xverify:all` 下输出相同的 `true\n38\n`。重放后工作树无证据漂移。

验收的 Jarde 根源码在唯一 `new Runnable() { ... }` 体内两次写出 `Inner.this`，不包含 `Inner$1` 或 `this$0`。独立的物理 `Inner$1` class-source 仍保留真实捕获字段和构造器，因此源码投影没有改写字节码事实或物理查询。证明链从准确 `EnclosingMethod`/匿名 self row、唯一 caller allocation/constructor XRef、synthetic-final `this$0` 字段、构造器 pre-super 写入和两次 SSA 字段读取，落到同次 AST 的 `QualifiedThis`；额外 owner 用途会拒绝整次匿名投影。

主线独立运行 `cargo fmt --all -- --check`、`cargo test -p jarde --lib --locked`（105/105）、`cargo test -p jarde-java --lib --locked`（219/219）、`cargo check --workspace --locked` 和 `openspec validate recover-proved-anonymous-inner-this --strict` 均通过。`cargo test -p jarde --test class_source anonymous_inner_this --locked` 六个真实 classfile 负例全通过，涵盖同 owner 第二分配、跨类构造引用、`EnclosingMethod` 身份错误、方法 fallback、两个额外局部捕获，以及预算/取消无部分发布。`jarde-java` 的 AST 单测进一步检查准确读取只命中一次且 read BCI 不改变。

本次只验收该 Java 8 单分配、一个外围实例字段、匿名 Runnable 的词法接收者形态；不推断 DT-04 的所有 `Outer.this`、局部变量捕获或 DT-06a 的父类实参已追平。冻结源码为隔离 DT-04，使用 `observed.equals(inner)`；先前 `observed == inner` 遇到独立的比较/多消费者 SSA fallback，留在单独语法单元巡查。
