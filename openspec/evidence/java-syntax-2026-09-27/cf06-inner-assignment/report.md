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

`--out` 必须为空目录。当前脚本固定原 class/JADX 的七行成功结果，保留 Jarde 编译失败作为修前事实；修后再提高三方门槛。
