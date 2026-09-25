# 构造器委托实参中的条件值

## 冻结的 Java 8 对照

[`ConstructorConditionalProbe.java`](ConstructorConditionalProbe.java) 将 `text == null ? 0 : input` 传给 `this(int)`；class SHA-256 为 `2bf349eb148816d67ad14ef8ffe7ec0639333e9e7c4cec9331a77cc83605e770`。[`ConstructorPairProbe.java`](ConstructorPairProbe.java) 将两个相继求值的条件字符串传给 `this(String, String)`；class SHA-256 为 `4e587e0a6c7f81c7b1e9f2a9b71683427a3d78c689068ae5ad2a381fac29c0d4`。均使用 `javac --release 8 -g:none -Xlint:-options` 编译。

两者都把尚未初始化的 `this` 留在操作数栈上，随后在条件臂的汇合点使用实参值。单实参类在 BCI 10 调用 `this(int)`；双实参类先于 BCI 12 汇合第一个实参，在第二个条件值汇合后于 BCI 22 调用 `this(String, String)`。字节码与 StackMapTable 见 [`javap.txt`](javap.txt)和[`pair-javap.txt`](pair-javap.txt)。

本机 JADX 1.5.6 对两例都输出条件实参，见 [`jadx.java`](jadx.java)和[`pair-jadx.java`](pair-jadx.java)。修正默认包的 `defpackage` 拼写后，完整类可用 Java 8 重编；原 class 和 JADX 输出分别在 `java -Xverify:all` 下通过单实参四条、双实参六条路径，逐行结果相同，见同目录的 `*-run.txt`。这证明这些样例上 JADX 表现正确，不证明它处理任意构造器条件实参。

## Jarde 架构判断

现有 `prove_conditional_value` 允许条件值外侧的栈槽携带相同输入穿过两臂，只要求一个**变化的**栈 Phi。`constructor_call` 通过 `invokespecial` 的真实参数和值描述符渲染实参。因此这里首先应检验现有普通 `If` → 条件值 Phi → 构造器实参消费者链，特别是双实参样例中保留第一个值穿过第二个分支的路径；不应仅因语法出现在 `this(...)` 中就增添构造器专用 Region。JADX 的 `ConstructorVisitor` 会把 invoke 转为构造器指令并内联移动链，可用于理解输出顺序，但 Jarde 已持有 JVM 栈和 SSA 事实，应以真实 Phi、参数顺序和初始化限制为准。

## 独立 CLI 结果与缺口

Root 用共享终点布尔返回验收时独立构建的 CLI（SHA-256 `7e4107e7d8d6c0d3c22bf9d13a46f3105e7c59ed6a18c1b4d6db8cfbf888b984`）处理两个冻结 class；报告与输出在 `root-after-single.{json,java}` 和 `root-after-pair.{json,java}`。单实参构造器为 `structured/java`，完整 Jarde 类 Java 8 重编并在 `java -Xverify:all` 的四条路径与原/JADX 逐行相同，三套记录为 `root-after-{original,jadx,jarde}-ConstructorConditionalProbe-*`。

双实参构造器则为 `fallback/mixed`。Jarde 在第一个条件分支输出空 `if`，随后在 BCI 22 的 `this(String,String)` 引用：“the value at BCI 22 is the entry state of stack depth 1, which no instruction produced”；完整类编译报可能尚未初始化 `first` 字段，见 `root-after-jarde-ConstructorPairProbe-javac.txt`。原源码与 JADX 的双实参完整类各自在六条 `-Xverify:all` 路径与冻结原类逐行一致。这是 Jarde 的真实 parity 缺口，而不是该探针上的 JADX 问题。更严重的是当前输出的 source map 只覆盖 BCI `0,1,2,3,22,25`，漏掉实际指令 BCI `6,7,10,12,13,16,18,21`：第二个条件图已部分标记为折叠，但调用失败后没有随失败一起引用，必须原子修复。

最早的语义边界在构造器实参渲染前：`init::prologue` 已识别委托调用，`constructor_call` 按真实 `invokespecial` 实参顺序渲染；`render_value` 遇到跨第二段条件图携带的栈深度 1 值时拒绝。当前 `prove_conditional_value` 只接纳被**同一个 join 块中的一条指令**立即消费的变化栈 Phi。第一个条件值在 BCI 12 汇合后仍要穿过第二个条件分支，最终才由 BCI 22 消费，故无法发表为条件表达式；第二个条件值与调用同 join 并不足以补救第一个。子代理的 SSA trace 进一步确认：BCI 12 `Stack(1)` Phi ValueId 7 汇合两臂，BCI 22 的同值 Phi ValueId 12 输入为 `[7,7]` 且被折回 7，调用实际读取 7；ValueId 7 的 uses 同时记录真实 BCI 22 指令与同值 Phi 的内部 use，因此现有证明先在 `PhiUseCount` 拒绝，尚未到 join-block 消费检查。所需的是在已有条件值证明中核对跨后续块的精确栈值传递和最终消费次序，同时保持独立效果拒绝，并让调用失败回滚已标折叠的条件图；不需要构造器专用 Region。独立任务见 [`preserve-carried-conditional-invocation-arguments`](../../../changes/preserve-carried-conditional-invocation-arguments/proposal.md)。

## 当前验收

上段“当前/基线”指修复前的冻结 CLI。最终源码由 Root 独立构建的 CLI（SHA-256 `dd5ff212acc464c02960e2c2e387e055195c3c077e4da7480766ce98d76bbff0`）重跑，完整结果见 [`verification-root.md`](../../../changes/preserve-carried-conditional-invocation-arguments/verification-root.md)。双条件构造器现恢复两个按左到右求值的条件实参，0 引用，14/14 指令 BCI 有来源；单条件 8/8。原源码、JADX、Jarde 的完整类分别 Java 8 重编，所得 class SHA 均与冻结输入相同，四路及六路 `java -Xverify:all` 结果逐行相同。四种 verifier-valid 非准入控制保持整体引用且逐 BCI 来源完整，静态与实例调用的 20 行替换执行与原件一致。`root-after-final-*.json`/`.java` 及三方 `*-javac.txt`/`*-run.txt` 为最终产物，旧 `root-after-impl-*` 是实现中途快照。
