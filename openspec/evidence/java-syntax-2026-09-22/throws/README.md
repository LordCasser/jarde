# 裸 throw：独立三方审计

2026-09-23，`javac 23.0.1 --release 8 -g:none`，当前 debug jarde，jadx 1.5.6。
这是 source-only 审计，不新增永久 class，不计入 corpus census。

`ThrowAudit.java` 包含 null、参数异常、新对象、调用返回异常、显式 cast、checked exception、
catch、finally、synchronized、条件双臂共十种方法。`Runner.java` 检查异常类型、对象身份及
调用次数，原始 class 的 13 行输出保存在 `original.txt`。临时编译目录为
`/tmp/jarde-throw-audit`，所有 JVM 执行使用 `-Xverify:all`。

实际结果：

- jadx 原始完整输出（`jadx/ThrowAudit.java`）重编译成功，13/13 行等于原始 class。
  输出自带 `package defpackage;`，未改其文本；仅给临时 runner/helper 加相同 package。
- jarde 完整实际输出保存在 `jarde/ThrowAudit.java`；javac 在 `caught` 报缺少返回语句，
  见 `jarde/javac.log`。没有删掉这个方法来宣称整类通过。
- 直线裸 throw 均未恢复。调用返回异常会先输出独立 `ThrowEffects.problem();` 再引用
  `athrow`；新异常引用 new/dup/构造调用/athrow；cast 也保留引用而没有可呈现消费者。
- `checked` 的方法声明已经正确带 `throws java.io.IOException`，无需另建声明机制。
- `caught` 的现有 try/catch 结构已恢复，只缺 try 中的 throw。`finally` 和
  `synchronizedBody` 在 guard 的独立证明边界拒绝，不能把所有失败归到缺一个语句节点。

## 架构边界

当前 `Operation::Throw` 已是忠实字节码事实，region 也已经认识终止控制流。因此直线及
已接受区域中的 throw 不需要新 pass、额外控制流索引或异常模拟器。`StmtKind` 目前没有
Throw，增加一个持有 `Expr` 的忠实语句节点是必要表达能力，不能用字符串表达式伪装。

需要配套检查现有消费规则：`init::renders_its_reads` 目前不含 Throw，new 异常因此没有
可认领的唯一消费者；build 中调用、checkcast 的延迟呈现也必须和 Throw 的实际可写证明
一致。失败要沿既有 `quoted_bcis` 保留整条生产者来源，避免只留下末尾 `athrow`。
沿用 `render_value(..., at, ...)`，保证嵌套生产者在 throw 的求值位置接受校验。

guard 已认领的 synthetic rethrow 必须仍归 guard；不得把本项扩成 finally/synchronized
路径证明放宽。帧层对athrow目前只检查已初始化reference形状，不是Throwable层级验证；
不能把它记作完整verifier证明，也不应为源码恢复引入新的类层级resolver。
`throw null` 必须保留抛 NPE 的效果。引用类型未知、重复消费、跨写入、消费无法呈现等
拒绝场景需先固定再实现。源码中的 checked exception 声明已可复用。

独立改动`recover-throw-statements`的proposal/design/spec/tasks已完成并通过strict；实施等待数值比较生产窗口移交，当前尚未实现。

## 被athrow丢弃的较低栈值

`ThrowStack.java`先调用mark再抛参数异常。原方法Code为6字节，最大栈1；在临时class中将
BCI3的pop等宽改成nop，并将该Code的max_stack改为2，异常下方因此还留有mark结果。
原class与patched class均通过`java -Xverify:all`，输出`identity=true:calls=1`。
证据在`discarded-stack/`，临时class在`/tmp/jarde-throw-stack/`，没有新增永久fixture。

这是消费语义的重要边界：`ssa.rs`在按SlotTouch登记实际读操作后，才按stack_after清掉
剩余栈定义；清栈本身不产生read。因此未来Throw只消费真正弹出的异常值，mark仍需作为
独立效果保留。现有SSA事实足以区分，不需要额外的“异常栈清理”恢复机制。

规范依据：[JVMS athrow](https://docs.oracle.com/javase/specs/jvms/se23/html/jvms-6.html#jvms-6.5.athrow)
和[JLS throw](https://docs.oracle.com/javase/specs/jls/se23/html/jls-14.html#jls-14.18)。
null会产生NPE；原始异常对象、表达式先求值的顺序以及栈中被丢弃值已经执行的效果都应保留。
