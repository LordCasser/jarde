# CF11 local reference-write baseline diagnosis

本报告只读取已完成的 debug 与 no-debug full-source captures、checker observations、javac 原始 stderr、Jarde 生成源和 javap 输出。没有运行 Java/JADX/Jarde/Rust 工具，也没有改共享 checkout。两份 execution receipt 均为 `cf11-target-full-source-capture-v3 / observed`，source/tool pins unchanged；checker 均为 observation-only 的 `verified_observation_capture_with_quality_gaps`。

| 目标 class | debug：JDK8/23 × default/all | no-debug：JDK8/23 × default/all | 结果 |
|---|---:|---:|---|
| `CF11OuterAssignmentTarget` | fail | fail | `noCastAssign` 把 `Iterator.next()` 的 Object 写进生成的 `String target`（或 `local1`）；javac 8 原始 stderr：`错误: 不兼容的类型: Object无法转换为String`。 |
| `CF11CastReadControl` | fail | fail | 生成源将 `values.get(0)` 的 Object 直接写入 `String target/local1`；后面的显式 String cast 在读取点，不能修复此前的非法赋值。javac 8 原始 stderr：`错误: 不兼容的类型: Object无法转换为String`。 |
| `CF11CharSequenceControl` | fail | pass | debug 源把 `builderLength` 写成一个 `String value`，随后赋 `StringBuilder`；javac 8 原始 stderr：`错误: 不兼容的类型: StringBuilder无法转换为String`。no-debug 源分成 String 与 StringBuilder 两个生成局部，四个编译/运行组合均通过。 |
| `CF11ReferenceArrayControl` | fail | fail | 生成源将声明为 `String[] target/local1` 的数组重赋为 `CharSequence[]`；javac 8 原始 stderr：`错误: 不兼容的类型: CharSequence[]无法转换为String[]`。 |
| `CF11ParameterControl` | pass | pass | 两 profile 的四组 isolated compile 均成功，runtime 与同 JDK 的原始 Runner oracle 一致。参数保持 Object，生成源保留从 List.get() 写回参数。 |
| `CF11SlotReuseControl` | fail | fail | 生成源把已结束的 String 局部和 Object 目标都表示成 `String local2`；`List.get()` 的 Object 值无法赋给 String。 |
| `CF11HandlerWriteControl` | fail | fail | 方法输出只有解释注释，没有可执行语句或 return；javac 8 原始 stderr：`错误: 缺少返回语句`。Jarde 对该方法明确标记未恢复：跨 protected region 的读取路径无法证明每条路径都先有展示写入。 |

这些 compile 状态来自每个 class 单独编译的 isolated 命令；每个 profile 下同时核了 JDK 8、JDK 23 与 Jarde `default`、`all` 四种组合。除表中特别指出的 `CF11CharSequenceControl` no-debug 例外外，失败信息在同类的四组组合中一致。JDK 23 stderr 另含 source/target 8 过时警告，不改变上述 javac 错误。

## CharSequence 的 profile 差异

原始 `builderLength()` 的 Code 字节逻辑在两个 profile 下都是：先把 `"seed"` 存入 slot 0，再构造 `StringBuilder` 并写回 slot 0，最后经 `CharSequence.length()` 返回。debug class 的真实 LVT 将 slot 0 的 `value` 描述为 `Ljava/lang/CharSequence;`，范围从 BCI 3 开始、长度 17；debug 的 Jarde 源却声明 `java.lang.String value` 并把 StringBuilder 赋给它，因此 debug isolated 编译失败。见 pins 中两份原始 javap 和失败 stderr。

no-debug class 不含 LVT/行号属性。Jarde 输出改成 `String local0 = "seed"` 和 `StringBuilder local0_2 = ...`，不再生成跨两种具体类型的赋值；JDK8/JDK23 × default/all 的四组 compile 与 Runner runtime 都成功且匹配同 JDK 原始 oracle。这是观察到的输出差异；它不单独证明底层 SSA 身份如何拆分，也不代表整个类集都可编译。

## Slot 物理证据

两 JDK 的 debug javap 对 `afterStringLifetime(Z,List)Object` 都记录同一组相关指令：`2: astore_2`、`3: aload_2`、`10: astore_2`、`15: aload_1`、`22: astore_2`、`23: aload_2`。实际 LVT 中 `first:String` 是 slot 2、`start_pc=3,length=5`（范围 `[3,8)`）；`target:Object` 也是 slot 2、`start_pc=11,length=14`（范围 `[11,25)`）。参数 `replace`/`values` 分别在 slot 0/1。no-debug 没有 LVT，但 javap 的 astore/aload 指令仍是相同序列。

因此物理 classfile 证据支持两个不重叠的 LVT 范围使用同一 slot index；它不证明 SSA value identity、LVT/SSA 的 source binding 是否 split，也不代表 Jarde 已正确恢复。事实上 Jarde 源把后续 Object 写入生成的 String 局部，两个 profile 的该目标都编译失败。

## 总体边界

两份原始 Runner oracle 都为 29 个唯一输出行，JDK8/JDK23 stdout 字节相同（SHA-256 `06d5983027988a0b07e3ac14de4f983688d448c4235e4f3cdae8819eed835f7b`）。Checker 记录的 `behavior_mismatches` 为 0，意思是所有实际编译成功并运行的生成结果与 oracle 一致；编译失败的 case 没有 runtime，因此不能据此推断其行为。debug 有 28 条 Jarde generated compile failure，no-debug 有 24 条；no-debug 有 8 个成功 isolated compile：CharSequence 与 ParameterControl 各四组。所有失败 stderr、生成源、原始 javap、receipt 与 observation 的 SHA/path 都在同目录 `pins.json`。

两 profile source closure 都是 4,744。checker 输出的 source map gaps 分别为 4，且 default/all 的方法报告存在差异；这些是捕获质量观察。该结果没有把 baseline 解释成产品修复，也没有据此声称接受产品行为。
