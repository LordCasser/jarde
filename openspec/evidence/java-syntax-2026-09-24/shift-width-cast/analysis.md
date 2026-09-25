# 移位左值宽度与被擦除的转换

`ShiftWidth.java` 是自写 Java 8 输入，以 `javac --release 8 -g:none` 编成冻结 class（SHA-256 `0ead0ef6d8d26e594f9498e716630b4697bea11d78198d021c892e9d5f33e43e`）。`wide(int)` 写 `((long)x) << 32`，`narrow(int)` 写 `x << 32` 再隐式转为 long，`mixed(int,int)` 写 `((long)x) >>> d`。两者对 `x=1` 分别是 `4294967296` 和 `1`；Java/JVM 的距离低位规则在 `d=33` 及负数上也可见。原 class 的 `java -Xverify:all` 对 `1,-1,7` 输出三行：

```text
4294967296,1,0
-4294967296,-1,2147483647
30064771072,7,0
```

本地 JADX 1.5.6 的原始输出在 `jadx.java.txt`，保留 `((long) i) << 32`、`i << 32` 与 `((long) i) >>> i2` 的区别；仅去掉自动附加的 `package defpackage;` 后，完整类通过 Java 8 重编与 `-Xverify:all`，上述三行逐字相同。对应源码实现可见本地 `jadx-core/.../SimplifyVisitor.java::isArithWideUpCast`：它特意保留会增加宽度的算术上溯，因为删掉后 Java 会选另一种移位宽度。这是值得复用的**约束**，不是让 Jarde 照搬 JADX 的寄存器宽度推断。

Jarde CLI `/tmp/jarde-instanceof-replay/jarde-cli`（SHA-256 `336fda92b93df11a33299cb015df7b925b4ade22970727a21ff09b4b32156b71`）的冻结类源码在 `jarde.java.txt`。`wide` 和 `mixed` 首先引用 BCI 1 的数值转换：其消费者移位仍为未支持操作，导致转换值无可呈现位置；`narrow` 引用 BCI 3 的移位。三个方法均非完整 Java 实现，故不对 Jarde 编译或执行作等价声明。问题属于 [recover-shift-expressions](../../../changes/recover-shift-expressions/tasks.md) 的六个 opcode 与左值结果宽度，不需要新的 CFG 区域；后续准入应从原 opcode 与 SSA 左操作数确定 int/long 结果，并保留原有 Cast 节点，不能按右操作数宽度或普通二元提升猜测。
