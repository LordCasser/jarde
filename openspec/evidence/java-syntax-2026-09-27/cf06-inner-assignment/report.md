# CF-06：条件内局部赋值与中间指令

固定队列项为 [CF-06](../../jadx-feature-inventory-2026-09-27/control-flow.md)。[replay.py](replay.py) 固定 JADX 提交及 `TestInnerAssign`、`TestInnerAssign2`、`TestIfElseAndConditionIntermediateInstruction` 和对应条件/变量处理代码的哈希。前两项从 Java 源码构造并含运行断言；第三项是允许 warning 的 Smali 测试，断言 JADX 在不能合并两个 IF 块时仍生成两处 `else` 与 `nothing2()` 文本，不能当作完整 Java 8 源码正向能力。

[baseline/summary.json](baseline/summary.json)记录一个去掉字符串拼接、日志等旁支的隔离 `InnerAssignCases`。原 class 与固定 JADX 的**完整类源码**均通过 `javac --release 8 -g:none`、`java -Xverify:all`，七行输出 `-1,4,-1,true,true,false,false` 一致。`lengthBranch` 在短路条件的第二条路径计算 `text.length()` 并把同一结果赋给局部、再与 5 比较；`assignedAndChecked` 先调用会写字段的 `call`，只有其返回 false 才把字段读值赋给局部、判空并调用 `isEmpty`。两处均要求赋值恰发生在可达分支、不能挪到短路前或重复读取字段。

Jarde 完整类源码不能重编：`lengthBranch` 只恢复 `if (!arg0.isEmpty())` 后将 `length()` 作为独立无用调用，BCI 11 的 `dup` 和后续值使用保留引用，最后错误地表面输出 `return -1`；`assignedAndChecked` 因短路测试链含共享值消费者但缺该形状的 SSA 证明而整方法引用，导致缺少返回。两者均带物理 BCI 和拒绝诊断，不能算已恢复。[物理 `javap`](baseline/javap.log)与两侧完整源码可复核原始赋值和求值路径。

架构判断：现有 `StmtKind::Assign/Declare` 可表达等价的有序语句，但 `ExprKind` 没有赋值表达式。若目标只是运行等价，可在已证 CFG/SSA 下把短路测试拆成早退、赋值语句、剩余测试；若要追平 JADX 对 `length = text.length()`、`value = this.field` 的**原位语法**，需要最小的局部赋值表达式表示及其 Java 类型/优先级/来源规则。不能先把一条有副作用的 `dup` 当作普通无副作用值复制，也不能仅为让文本看似完整而跳过共享 consumer 的证明。后续 OpenSpec 应在这两种呈现之间以完整来源、变量作用域、预算和真实运行结果选择，不与 CF-03 共享尾或 CF-05 数值窄化合并。

重放：

```sh
python3 openspec/evidence/java-syntax-2026-09-27/cf06-inner-assignment/replay.py \
  --jarde /tmp/jarde-root-cli-cf04-791641c2 \
  --jadx /Users/lordcasser/workspace/testzone/jadx/jadx-cli/build/install/jadx/bin/jadx \
  --jadx-checkout /Users/lordcasser/workspace/testzone/jadx \
  --out /tmp/jarde-cf06-fixed-replay-20260927
```

`--out` 必须为空目录。上述基线记录了修前 Jarde 编译失败；以下后验收使用提高了三方硬门槛的同一脚本。

## 实施后验收（2026-09-27）

[post-local-assignment/summary.json](post-local-assignment/summary.json) 记录三方完整类源码的 SHA-256、`javac --release 8 -g:none` 和 `java -Xverify:all` 结果。原始源码、固定 JADX 和 Jarde 全部重编成功，七行均为 `-1,4,-1,true,true,false,false`。脚本现以此为硬门槛，还检查 `lengthBranch` 仅调用一次 `length()`、`assignedAndChecked` 仅调用一次 `call()` 并读取一次 `field`，两个条件内赋值及后续局部读取均存在；任何 `@bytecode` 或缺失方法都会失败。Jarde 源码 SHA-256 为 `fda8596207c3348174824239d6fc066afce93421628b1f80cc57a7f78b34c59f`。

局部赋值仅认 Region 已拥有的条件测试 BCI。证书要求源值、`dup`、局部 store 和测试在同一 SSA 块内，两份复制各仅有 store 与测试一个消费者；中间只允许常量入栈，且无异常 handler。目标变量的名字、类型与词法声明来自原计划。最终 AST 必须实际发布每个已隐藏的赋值表达式，否则整个方法回退为 BCI quote；低预算或取消不发布部分正文及来源映射。[定向夹具](../../../tests/fixtures/p3-inner-assignment/README.md) 覆盖额外复制消费者、Java 类型不符、交错副作用、异常边，以及带有同块 `dup; store; test` 但未被此证书接管的循环条件。负例均保留 quote，Java 8 负例 class 也通过验证器并运行。Rust 定向测试检查调用次数、来源 BCI、预算和取消；`jarde-java` 全套测试通过。

范围边界：字段/数组赋值左值尚未覆盖；JADX 的 `TestIfElseAndConditionIntermediateInstruction` 仍是允许 warning 的 Smali 弱断言，不能作为完整 Java 8 源码正向证据。循环区域所有权与 Frame 不在本变更内。
