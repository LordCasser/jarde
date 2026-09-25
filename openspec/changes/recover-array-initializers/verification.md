# 数组初始化器验证记录

## 实现与定向验证

数组初始化证明位于 `crates/jarde-java/src/build.rs`，只接受一维、正的常量长度，以及同一基本块中按顺序完整覆盖 `0..N-1` 的 `dup; constant-index; value; arraystore`。SSA 约束分配身份、每个 copy/index 的单次消费和元素表达式来源；表达式生产者也要求单次消费，避免递归渲染同一 SSA 值时重复执行。类型不确定、控制流/别名/逃逸不符合形状或异常处理器集合不一致时，证明不认领该链。`chained_pair` 未作修改。

现有 `ExprKind::NewArray` 增加可选初始化元素；只有已证明的 rank-1 链由 emitter 写为 `new T[]{…}`，普通和空分配继续写为带长度的 `new T[n]`。最终消费的 SSA dup 别名只在证明成功后映射回分配值。完整链及元素 BCIs 进入同一表达式来源；元素生产者不能被 deferred-binding 提前提升到分配之前。

定向验证通过：

```text
CARGO_INCREMENTAL=0 cargo check -p jarde-java
CARGO_INCREMENTAL=0 cargo test -p jarde --test p3_array_access --test p3_array_initializers
```

11 个普通数组访问回归和 4 个数组初始化集成回归均通过。测试对永久 class 的四种非空初始化核对正文、默认/all 正文一致性，以及完整来源表中的长度、分配、dup、索引、元素、store 和最终消费者 BCIs；同时确认空数组仍使用原来的普通分配形式，并确认零输出预算不发布整类结果。两个紧预算相邻回归分别锁定：无数组方法 `ArrayTypes.text(String)` 在 84 个 IR 项上恰好完成；`DeferredValueOrder.multiArray(I)[[I` 的 `multianewarray` 在 104 个 IR 项上恰好完成。预检只读取已发布 effects 中的 `newarray`/`anewarray` opcode；没有一维候选时直接返回空计划，不扫描并计费 effects 映射，多维部分分配的 `multianewarray` 不触发候选收费。

## 补充边界重放

以 `openspec/evidence/java-syntax-2026-09-22/array-initializers/boundaries/ArrayInitializerBoundaries.java` 和 runner 重新执行：

```sh
javac --release 8 -g:none -d "$tmp/original" ArrayInitializerBoundaries.java
javac --release 8 -g:none -cp "$tmp/original" -d "$tmp/original" ArrayInitializerBoundariesRunner.java
jarde-cli class-source --input "$tmp/original/ArrayInitializerBoundaries.class" \
  --class ArrayInitializerBoundaries --policy single-class --release 8 \
  --format text --evidence all > "$tmp/jarde/ArrayInitializerBoundaries.java"
javac --release 8 -g:none -d "$tmp/jarde" \
  "$tmp/jarde/ArrayInitializerBoundaries.java" ArrayInitializerBoundariesRunner.java
java -Xverify:all -cp "$tmp/original" ArrayInitializerBoundariesRunner
java -Xverify:all -cp "$tmp/jarde" ArrayInitializerBoundariesRunner
```

原 class 与恢复类的非协变 runner 行逐字节一致，SHA-256 均为 `0f22c3339d91a9d074200291948e4e089dd185cabb17ad667333839f293a59c4`。因此 `mark('a'..'c', value)` 元素各执行一次且保持 `abc` 顺序；动态长度、重复/跳跃写、提前逃逸和跨分支情形继续走普通数组路径，返回值和逃逸时观察值一致。

协变负例的原行为为 `ArrayStoreException`。恢复正文没有把它改写为 `new String[]{new Object()}` 或插入会改变异常类型的 cast，而是在该方法保留 `@bytecode` 引用，且完整证据的 source map 保留原分配、未支持对象构造、`aastore` 及返回 BCIs。带 fallback 注释的混合正文即使可被 `javac` 接受，也不作为可执行语义等价物；这条负例只验证保守拒绝，不计入正例运行结果。

## 永久 fixture 的原先阻塞

永久 fixture `tests/fixtures/p3-array-initializers/v8/ArrayInitializerProbe.class` SHA-256 仍为 `998bdb54c863d92cb63cc08c654df21bc674961c652fc46a5c3eaf7de2915c86`。当前恢复完整类可用 `javac --release 8 -g:none` 编译，并在 `java -Xverify:all` 下运行。四种非空数组都已输出完整初始化器，空数组保持普通分配。

这份 fixture 的 14 行 runner **尚未通过**逐行等价：10 行 effectful 调用的数组值和异常类别匹配，但 trace 丢失。helper `intElement`/`stringElement` 在 BCI 4 使用 `i2c`；当前解码把该转换视为 `Other`，导致后续 `append(char)` 调用被保守引用，重编译类自然没有 trace 副作用。此缺陷属于独立的 `recover-primitive-conversions` change（15 种转换），没有混入本 change。故任务 3.1 保持未勾；待该独立转换实现合入后，须重放此永久 fixture 的全部 14 行，不能以本次补充样本代替。

本次最终 CLI 构建 SHA-256 为 `27d265b16c648dd3c0e7adc0fbfdf35d47d0f7a1e1e10c63ec7262b02cd625f4`；这是本 agent 的临时验证构建记录，root 将按任务 3.2/3.3 独立审读、重建与重放。

## root 独立验收

root 审查了常量长度、同一 SSA 分配身份、逐索引单次消费、具体 `*astore` opcode、元素类型与异常处理边界，并把无数组及仅有 `multianewarray` 的紧预算方法排除在候选收费之外。最终 CLI `/tmp/jarde-cli-root-final-accepted` SHA-256 为 `d7520a08a37b54ef579d42c770f5b512af17ddb11b05050568458759c78dbf16`。复制脚本与原始 Java 源到 `/tmp/jarde-array-root-postrefactor`、`/tmp/jarde-array-boundaries-root-postrefactor` 后原样重放；原/JADX/Jarde 永久 fixture 均完整编译、验证、运行，Jarde 14 行仅 4 行与原始逐字一致，其余 10 行值与异常一致但 trace 为空，故 3.1 未验收。边界 12 行中 11 行一致；协变 `aastore` 保留 `@bytecode` 和来源，混合正文运行返回 `[null]` 而非原始 `ArrayStoreException`，不能当作等价执行。补充 `mark(char,int)` 正例的 `effects=abc` 和调用次数一致，证明本轮初始化表达式的顺序。

最终定向 Rust：类源码 42、数组访问 11、数组初始化 4、链式赋值 2，以及部分维度/延期值/调用实参相邻回归全部通过；需要 JDK 的三项完整类执行对照也通过。reader 160、类型属性 4、CLI 类源码 16、corpus fingerprint 5 与 reader census `(151, 1086, 98, 346, 8)` 通过。`cargo fmt --all -- --check`、严格 reader Clippy、root lib Clippy（仅既存 `region.rs` 的 `type_complexity` 例外）、`git diff --check` 和本 change 的 OpenSpec strict 均通过。缺失的 `i2c` 属独立 `recover-primitive-conversions`；协变 fallback 语义缺口须与该证据分开追踪，不能改写为本初始化器的成功样本。

## 转换依赖解除后的永久 fixture 终验

`recover-primitive-conversions` 接通 `i2c` 后，root 以重新构建并冻结的 CLI `/tmp/jarde-cli-root-primitive-conversions-accepted`（SHA-256 `8b86c7293387e4be385630884109cb0effa4d044762788ac019abb7cbdd756e9`）在独立 `/tmp/jarde-root-array-after-conversion-20260923` 原样重放永久 fixture。`javac --release 8 -g:none` 生成的原 class 与提交的 1281 B class 逐字节一致；恢复的完整类零 `@bytecode`，再次 javac 成功。两个类都通过 `java -Xverify:all`，14 行值、效果 trace 与异常文本逐字节一致，输出 SHA-256 同为 `f16eb426dc781c7ac365ff0f85b20c1452269145a306297802105f5eeb5a7349`。先前 10 行空 trace 的外部转换阻塞已解除，任务 3.1 现在验收完成；原负例中的协变 `aastore` 仍以拒绝记录，不计入语义等价。
