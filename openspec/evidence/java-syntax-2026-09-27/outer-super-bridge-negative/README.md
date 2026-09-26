# Outer `super` bridge negative evidence

本目录记录 `project-proved-outer-super-bridges` task 1.2 的小型 Java 8 夹具。复现命令为 `python3 run_evidence.py`；命令用 `javac --release 8 -g` 编译，执行 `java -Xverify:all`，保存 `javap -v -c -p` 输出，并通过临时目录自动清理所有 class 文件。反汇编首行的临时编译路径替换为 `<base>`、`<patched>` 或 `<direct>`，连续重放得到相同文件哈希。夹具由当前 JDK 23.0.1 编译为 class-file major version 52。它们不改冻结的 `OuterReceiverCases`，不代表 Jarde 已实现或接受这些拒绝。

## verifier-valid 夹具与结果

- `src/OuterSuperCases.java` 同时包含 `explicitOther(OuterSuperCases other)` 的普通动态调用、捕获 Outer 的两个 super 调用、带同型参数的 super 桥候选、lambda 中的 super 调用和 Member 自身 `this.value()`。`BaseA.value()` 返回接收者的 `state`，Outer 覆写返回 `state + 100`，MemberBase 返回 `3`，因此不同分派可由结果观察。
- 未改写 class 的结果为 `120:10:11:10:10:3`，见 `run-baseline.txt`。显式 `other` 保持动态分派并得 120；捕获 Outer 的两次调用、lambda 调用进入 BaseA 并得 10；Member 自身调用得 3。
- `javap-baseline.txt` 中 Member 的 `explicitOther(OuterSuperCases)` 在 BCI 0 `aload_1`、BCI 1 `invokevirtual OuterSuperCases.value:()I`。这说明相同静态类型的 `other` 不是 Outer 的词法捕获值，也不是可由类型推得的 super 接收者。
- 为得到实际可执行的“同型 other 被送入桥”负例，`patch_other_call.py` 只改编译结果的 `OuterSuperCases$Member.otherBridgeCandidate(OuterSuperCases)`：保留 4 字节接收者序列长度，将 BCI 0..3 的 `aload_0; getfield this$0` 改成 `aload_1; nop; nop; nop`，并把 BCI 4 的 `invokestatic` 常量池引用改为 `OuterSuperCases.access$101:(LOuterSuperCases;)I`。这是 verifier-valid 字节码修改；两个值的验证类型相同，调用栈形状与指令长度不变。`run-patched.txt` 在 `-Xverify:all` 下输出 `120:10:11:20:10:3`，可见第二调用点传入 `other` 后桥仍执行 BaseA 实现，但读取的是 `other.state`。系统应因 BCI 0 的数据流来自显式参数而拒绝把该调用投影为 `Outer.super`。
- 同一个物理桥 `OuterSuperCases.access$101:(LOuterSuperCases;)I` 因上述改写拥有两个调用点：`secondCapturedCall()` BCI 4（接收者来自 `this$0`，可成为候选）和 `otherBridgeCandidate(OuterSuperCases)` BCI 4（接收者来自 slot 1，必须拒绝）。两者在 `javap-patched-member.txt` 都指向 `access$101`；完整闭包不能只计入第一个调用点，也不能因第二个失败而删除整个桥。
- 两个桥体调用 `BaseA.value:()I`，并在桥方法自己的 BCI 1 执行 `invokespecial`；桥 BCI 0 为 `aload_0`、BCI 4 为 `ireturn`。`javap-baseline.txt` 中四个 access helper（`access$001`、`$101`、`$201`、`$301`）均是 `ACC_STATIC, ACC_SYNTHETIC`，descriptor 均为 `(LOuterSuperCases;)I`，目标都是 `BaseA.value:()I`。这些是不同物理方法定义，不能只凭目标相同合并。
- `lambdaOuterCall()` 在 Member BCI 1 执行 `invokedynamic #29`。`BootstrapMethods[0]` 的实现参数是 `REF_invokeSpecial OuterSuperCases$Member.lambda$lambdaOuterCall$0:()I`；该 synthetic 方法 BCI 4 再 `invokestatic OuterSuperCases.access$301`。这是一条 javac lambda/bootstrap 到桥的可执行间接使用链，不是 bootstrap 直接引用桥。源码夹具和实际引用链都在 `javap-baseline.txt` 中。
- `src/DirectParentTargetCases.java` 是独立的 verifier-valid target 夹具。Outer 直接继承 `TargetBaseB`，Member 调用 Outer super；`javap-direct-parent.txt` 中 Outer `access$001` BCI 1 实际引用 `TargetBaseB.value:()I`，执行结果为 `102`（`run-direct-parent.txt`）。若候选证明来自冻结样本并预期直接父类目标 `ReceiverBase.value:()I`，此不同物理目标必须拒绝该预期证明；通用实现仍须按各自 Outer 的准确直接父类目标重新验证，不能硬编码 `ReceiverBase` 或仅看方法名。

## proof-unit 负例（不是 verifier-valid class）

以下情形无需扩张夹具或编造可运行字节码；它们只定义证明单元的拒绝输入。它们没有实际 class-file BCI，不能称为 `javac` 产物、不能声称经 verifier 验证；预期拒绝在进入源码投影前发生：

1. **桥内额外效果**：在上述单返回体的 `invokespecial` 前插入可观察静态写入或调用；即使最终仍返回父类结果，也不等价于单个 `Outer.super.method(args)`。预期拒绝原因：桥体包含额外效果。
2. **另一出口/异常覆盖**：桥体含条件分支，一条路径先返回常量或抛异常，另一条路径才执行父类 `invokespecial`；或异常表覆盖调用但投影表达式不覆盖相同范围。预期拒绝原因：返回/异常路径与单表达式不等价。
3. **直接 method-handle/bootstrap 桥引用**：常量池 `MethodHandle` 直接指向 `REF_invokeStatic OuterSuperCases.access$101:(LOuterSuperCases;)I`，或 bootstrap argument 指向该句柄。当前 javac 源码没有生成这种直接 accessor 句柄，因此此项是 proof-unit；预期闭包枚举必须识别此引用并拒绝 helper 文本删除。它与已实际观察到的 lambda 实现方法间接调用（上一节）分开记录。
4. **错误父类目标**：以 `invokespecial` 目标不等于已解析直接父类方法的桥体摘要作为输入，拒绝原因应包含物理目标不匹配。将其改成非直接父类目标是否仍能通过 JVM verifier 取决于具体类文件形状；本证据不作 verifier-valid 声称。实际、可验证的其他直接父类目标见 `DirectParentTargetCases`。

## 覆盖界限

已覆盖：同类型普通 `other` 动态调用；经 verifier-valid 改写把 `other` 作为桥实参；同一准确桥的第二调用者；lambda 的 `invokedynamic`/bootstrap 间接使用；不同直接父类的准确目标；Member 自身动态分派；桥内额外效果、另一出口/异常覆盖和直接桥 method-handle 引用的明确 proof-unit 拒绝输入。

未覆盖：直接指向桥的 method-handle/bootstrap class 文件、带额外效果/多出口/异常表的真实 verifier-valid 变异 class、接口 default 与跨层父类目标、Jarde 对这些拒绝的实际输出。它们没有被冒充为已执行负例；后续实现测试应据此补充或维持拒绝，任务 1.2 本记录不代表完整实现验收。
