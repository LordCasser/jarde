# 非泛型成员内部类：调用点负控

本目录用 `javac --release 8 -g:none -Xlint:-options` 编译 [`fixture`](fixture/)，然后在私有临时目录以 `java -Xverify:all` 执行。已验证的类文件副本、`javap -c -p` 与原始输出在本目录；JADX 版本为 [`1.5.6`](jadx-version.txt)。临时编译目录在验证后删除。

[`nestedEffects`](javap-NegativeUse.txt) 是一条 verifier-valid 的标准限定构造调用：BCI 0/3 分配并复制 `NegativeOuter$Inner`，BCI 4 读取限定接收者，BCI 5 复制外层实例、BCI 6 执行 `Objects.requireNonNull`，BCI 9 弹掉检查结果。两个普通实参效果从 BCI 10 开始，内层 `mark("B", value)` 在 BCI 15 调用，外层 `mark("A", ...)` 在 BCI 18 调用，BCI 21 才调用物理 `(NegativeOuter;I)V` 构造器。运行结果 `ok:BA` 证明实参效果顺序；对 null 接收者结果为 `null:`，证明 A、B 均未执行。拒绝边界建议：必须证明接收者 null-check 完成早于首个普通实参副作用，并保留普通实参间原顺序；不能因调用点包含多个有序副作用就一概拒绝，也不能把这组调用折叠成一个实参。

[`preEffect`](javap-NegativeUse.txt) 在 BCI 0–6 先执行 `mark("P", value)` 并存入局部变量，之后才在 BCI 7 开始分配，BCI 13/16 检查接收者，BCI 17 起计算普通实参。null 接收者运行结果为 `pre-null:P`：构造表达式之前的效果应保留，而构造表达式内的 A 效果仍不发生。拒绝边界建议：只将半开区间 `[receiver evaluation, ordinary argument evaluation)` 中已经被证明属于接收者的 null-check 作为排序约束；不要把方法中更早的可观察效果误判为可移除或重排。

JADX 对两方法都输出 `Objects.requireNonNull(negativeOuter)` 后接 `negativeOuter.new Inner(...)`；输出用 Java 8 重编，且 `-Xverify:all` 的运行输出与原类逐字一致（见 [`jadx-run.txt`](jadx-run.txt) 与 [`original-run.txt`](original-run.txt)）。Jarde 对 `nestedEffects` 与 `preEffect` 均保留前置效果，但对分配/复制/检查值在 BCI 0、3、5 等处给出未证明形状标记，没有生成可编译的调用表达式；完整结果见 [`jarde-NegativeUse.txt`](jarde-NegativeUse.txt)。这说明当前安全边界仍是拒绝该形状，而不是已支持副作用排序。

限定接收者与物理首参身份不一致的控制未收录：普通 Java 源的 `receiver.new Inner(...)` 由 `javac` 固定把同一个接收者用作合成首参，无法产生该反例；要得到 verifier-valid 但身份不同的调用，需独立生成或变换字节码（例如 ASM），再用运行行为证明两个身份。当前证据没有构造这种字节码，因此不能把类名、描述符相同或上述源码正例当作身份一致的普遍证明，也不据此提出该边界的已验证拒绝规则。错误 outer class 关系同样未伪造：本轮类均由 javac 生成，不能作为其拒绝证据。

SHA-256（类文件）：

| 文件 | SHA-256 |
|---|---|
| `NegativeOuter.class` | `61c2b72668099e5ddb43cfd674d2d7a8383644fa843fe2c1cf15f982ab3582f2` |
| `NegativeOuter$Inner.class` | `1477ecf9f206eb33a854af05e39b1860fe8faa69248980b495c5e13acc71bca0` |
| `NegativeUse.class` | `5ca924a9ba8fb7fbfa86c042e43aa3b54a5c563e01ddc394b1e46419728c38e7` |
| `NegativeRunner.class` | `f7a15440ff705fa59c4403557e0d055fd5bd50e5ed48bd2398e1ffcece556cb3` |
