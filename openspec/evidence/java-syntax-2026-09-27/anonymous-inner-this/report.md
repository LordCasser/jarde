# DT-04: lexical enclosing-instance receiver `Inner.this`

本证据用顶级 `Inner` 与一个匿名 `Runnable` 隔离词法外围实例读取：匿名体执行 `observed = Inner.this` 和 `Inner.this.f++`，入口通过未覆写的 `observed.equals(inner)` 核对身份并输出字段值。fixture 与两个冻结物理 class 位于 [anonymous-inner-this fixture](../../../../tests/fixtures/proved-java-structure/anonymous-inner-this/)，完整三方重放由本目录的 [replay.py](replay.py) 完成；脚本在 `TemporaryDirectory` 中编译源代码并使用临时 `CARGO_TARGET_DIR`，退出时自动清理。SHA 和 BCI 证据分别见 [SHA256SUMS](../../../../tests/fixtures/proved-java-structure/anonymous-inner-this/SHA256SUMS) 与 [original-javap.txt](original-javap.txt)。

原始源码在 `javac --release 8 -g:none` 下编译、匹配两个冻结 class 的 SHA，且 `java -Xverify:all` 成功，输出：

```text
true
38
```

JADX（本地 checkout `2fb1b16386941660fda07e9017285aec40fcb37f`）生成一份完整源文件。它在匿名 Runnable 体中输出 `Inner.this`，整份源码 Java 8 重编成功，`-Xverify:all` 输出与原 class 一致。完整源码与结果见 [jadx-source](jadx-source/)、[jadx-javac.log](jadx-javac.log) 和 [jadx-run.log](jadx-run.log)。

Jarde 分别输出物理 `Inner` 与 `Inner$1` 两份源码。`Inner.make()` 保留 `new Inner$1(this)`；匿名类仍声明 `final Inner this$0`，方法体为 `Inner.observed = this.this$0` 和 `this.this$0.f += 1`，没有恢复 `Inner.this`。family 事实报告拒绝 `$1`，原因为 `selected root has EnclosingMethod identity`。完整物理源码集重编失败，`jarde-javac.log` 的唯一错误落在 `$1` 构造器的 `this.this$0 = arg1;`：JDK 23 的 `--release 8` javac 报 `flexible constructors is a preview feature and is disabled by default`。因此基线 Jarde 源码未通过 Java 8 重编，未运行；此处证明了源语法/可编译性缺口，不声称基线 Jarde 运行时语义不等价。源码、CLI 结果与诊断见 [jarde-source](jarde-source/)、[jarde-family-facts.txt](jarde-family-facts.txt)、[jarde-javac.log](jarde-javac.log) 和 [jarde-run.log](jarde-run.log)。

fixed 回放只把有证明的匿名 `$1` 原子内联进 `Inner.make()`：输出体保留两处 `Inner.this`，根源码不含 `$1`、`this$0` 或物理构造器；独立物理 child 报告仍可见 `this$0`。根源码使用 `javac --release 8 -g:none` 完整重编，并由 `java -Xverify:all Inner` 输出 `true\n38\n`。fixed 日志和源码位于 [fixed](fixed/)。更早的 `observed == inner` 变体在 Jarde 主方法触发了独立布尔比较/多消费者 SSA fallback，未用于冻结；DT-04 不覆盖该表达式恢复债务。

关键 classfile 事实：`Inner.make()` 在 BCI 0 分配 `Inner$1`，BCI 4 加载 `this` 作为唯一构造参数，BCI 5 调用 `$1.<init>(LInner;)V`；匿名构造器在 BCI 2 把参数存入 `this$0`，随后才在 BCI 6 调用 `Object.<init>()V`。匿名 `run()` 在 BCI 1/8 分别读取 `this$0`：第一次把同一个 `Inner` 身份写入 `observed`，第二次读取 `Inner.f` 并在 BCI 17 写回。顺序不能靠移动物理 pre-super 字段写入解决；成功的匿名源码投影必须完整隐藏已证明的 `$1` 构造器脚手架。

JADX 的集成测试 [TestAnonymousClass2.java](/Users/lordcasser/workspace/testzone/jadx/jadx-core/src/test/java/jadx/tests/integration/inner/TestAnonymousClass2.java) 在 `test2()` 中精确含 `Object obj = Inner.this;`。测试只对生成文本作包含/不包含断言：排除 `synthetic`、`AnonymousClass_` 和 `Inner obj = ;`，并检查 `f = 1`、`f = i`、`Inner.this;`；它没有自己执行生成源码或原 class。JADX 相关生产路径为 `ClassModifier.removeSyntheticFields` (`jadx-core/src/main/java/jadx/core/dex/visitors/ClassModifier.java:79-149`)，它删除匿名 synthetic 外围字段构造写入并为父类实例字段设置 replacement；`InsnGen.instanceField` (`jadx-core/src/main/java/jadx/core/codegen/InsnGen.java:186-202`) 把 replacement 写成 `Inner.this`；`InsnGen.makeConstructor` / `inlineAnonymousConstructor` (`.../InsnGen.java:727-832`) 将匿名类体合到创建点。

Jarde 已有 `ExprKind::QualifiedThis` 及 emitter，但只在 `build.rs` 收到 `ProvedCapturedOuterRead` 后生成该节点；`src/facade.rs` 的既有具名成员 family capture handoff提供该证明。`src/member_inner.rs::prove_target` 对 `EnclosingMethod` child 返回无候选，所以本例的匿名类无法进入这一投影路径，普通字段表达式遂保留 `this.this$0`。这适合建立一个受限匿名 family 证明与单分配点投影，而不是增加无证据的全局字段替换。

本切片仅覆盖词法 `Inner.this` 和匿名类外围实例字段。它不实现 DT-06a 的无捕获匿名父类构造实参映射：本例目标是 `Runnable`，唯一参数是 `Inner` 捕获对象，不存在父类构造参数恢复。
