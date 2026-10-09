# Child-array 对抗控制构造方案

## 范围与依据

本文是只读规划，不是测试结果。未运行 Cargo、Git、Java，也未改产品、fixture 或规划文档。控制变体均从现有 direct `Main.class` 的 `ownGridDirect()` 代码派生，优先复用 [p3_heterogeneous_array_initializers.rs](/Users/lordcasser/workspace/projects/jarde/tests/p3_heterogeneous_array_initializers.rs) 里的 `DIRECT_JAVAC8`、`DIRECT_JAVAC23`、`archive`、`opened`、`class_source` 与 body 查询；只需在该 integration test 文件新增测试侧 classfile Code-span patch helper，不需要产品 API 或新 fixture 家族。

输入主类位于 `tests/fixtures/p3-heterogeneous-array-initializers-v3/direct/{javac8,javac23}/classes/Main.class`。本地 frozen javap 记录 `openspec/changes/recover-heterogeneous-array-init/evidence/heterogeneous-array-initializers-v3-pathfix/direct/javac8/logs/direct_javac8_javap-main.stdout` 给出的 `ownGridDirect()` 指令序列如下；javac23 对应序列相同：

```text
0 iconst_2
1 anewarray #55 ([LBase;)
4 dup
5 iconst_0                 // 父数组第一个 index
6 iconst_1                 // child 长度
7 anewarray #30 (DerivedA)
10 dup
11 iconst_0                 // child index
12 new #30 (DerivedA)
15 dup
16 iconst_1
17 invokestatic #11 (Main.mark:(I)I)
20 invokespecial #31 (DerivedA.<init>:(I)V)
23 aastore                  // child element store
24 aastore                  // 父数组 store / child consumer
25 dup
26 iconst_1                 // 父数组第二个 index
27 iconst_1                 // child 长度
28 anewarray #32 (DerivedB)
31 dup
32 iconst_0
33 new #32 (DerivedB)
36 dup
37 iconst_2
38 invokestatic #11 (Main.mark:(I)I)
41 invokespecial #33 (DerivedB.<init>:(I)V)
44 aastore                  // child element store
45 aastore                  // 父数组 store / child consumer
46 areturn
```

只读解析 javac8 `Main.class` 的 Code 属性得到 `ownGridDirect`: `max_stack=9`、`max_locals=0`、`code_length=47`、异常表 0 项、Code 子属性 0 项。常量池有 `Class java/lang/Object`（#13）、`Class DerivedA`（#30）、`Class DerivedB`（#32）、`Methodref Main.mark:(I)I`（#11）；父数组的类常量为 #55。数字只作为 javac8 原件的锚点；javac23 变体应从其自身 pool 查对应项，不假定跨编译器常量池编号相同。已有 Boolean 变体 helper 已展示从 `CodeAttribute.content_span` 定位 Code、调整 `code_length` 与 Code attribute length 的测试侧做法（同文件 `tests/p3_constructor_primitive_conversion_arguments.rs:532–608`）。

实现前应以目标 leg 的 class facts 再核对 Code 内容与 CP 索引，确保只修改上述方法；不要用历史行号代替当前原始 class 的 Code BCI。

## 最小变体

| 控制 | `ownGridDirect` 改动 | 预计拒绝层与能证明的边界 |
|---|---|---|
| 父元素乱序 | 原地交换父 index 常量：BCI 5 的 `iconst_0` 改 `iconst_1`；BCI 26 的 `iconst_1` 改 `iconst_0`。不可误改 child 长度 BCI 6/27。Code 长度与 CP 不变。 | 父初始化器按递增 index 消费；预期第一个 parent store（BCI 24）遇到 index 1 时结构 proof 失败，child 链不得独立提交。该控制验证父 store 顺序约束，拒绝发生在协变类型呈现前，不应声称命中 Builder assignability。`aastore` 索引值变化不影响 class verifier；不执行语义已变的类。 |
| child 最终值的非认可读取 | 在 child element store BCI 23 后、父 store BCI 24 前插入 `dup; pop`（`0x59,0x57`）。当时栈为 `[parentArray, parentIndex, childArray]`；插入后回到同一栈形，随后父 `aastore` 仍有正确的三个操作数。`max_stack=9` 已高于该位置新增峰值；Code 与 Code attribute 长度各加 2，异常表和子属性保持 0。 | child retained array ValueId 的下一条读取是新增 `dup`。已有 `array_initializer_reader` 的“紧邻下一条且 sole-use”快路之后，调用方只接受普通 initializer consumer 或明确的父 `aastore` 第三个栈操作数；`dup` 不满足，child 候选应被拒绝，闭合 parent chain 不得提交。这里不能表述为“同一个 ValueId 有两个 uses”：JVM `dup` 读取输入一次并产生两个不同 SSA 输出；原 retained ValueId 的 reader 是这条不受支持的 `dup`。 |
| child-parent 区间内插入副作用 | 在 child element store BCI 23 后、原父 store BCI 24 前插入 `iconst_1; invokestatic #11; pop`。此时栈为 `[parentArray, parentIndex, childArray]`；调用将参数压入、消费并弹出返回 int，结束后栈不变。该字节序列复用原有 `Main.mark:(I)I` Methodref，不需新 CP 项。序列长 4 字节；Code/code-attribute length 各加 4；最大新增栈深不超过旧 `max_stack=9`。 | child retained value 的唯一实际读取仍是父 `aastore`，但它不再是紧邻 child 最后一次 element store 的 reader。`array_initializer_reader` 的备用区间证明必须拒绝未被父 reader 操作数生产所解释的 mark 调用；若 operand-interval 收集先失败，也属于同一 reader closure 门。必须从报告确认实际拒绝点，不把它说成 duplicate reader 或 Java 类型拒绝。 |
| child/parent 类型不兼容 | 仅把 BCI 7 `anewarray #30 (DerivedA)` 的两个 CP 操作数字节替换为当前 leg 中真实存在的 `Class java/lang/Object` 项（javac8 #13；javac23 先查本 leg）。保留 BCI 12 的 `new DerivedA` 和 child 内部 `aastore`，不改其余 pool/descriptor。child 变为 `Object[]`，其中写入 `DerivedA` 引用仍类型兼容；parent 仍是 `Base[][]`，因此其 component 为 `Base[]`。 | 预期 child 结构与 child/parent store 身份闭合，随后 Builder 对 `Object[]` → `Base[]` 的引用赋值兼容性拒绝完整 initializer；完整方法必须 fallback，不能以结构 Site、NewRecord 或 source-map 锚点冒充成功 Java。原类的父 `aastore` 在执行时会因 `Object[]` 不可赋给 `Base[]` 而触发 `ArrayStoreException`；变体不执行。此处测试 class verifier 只确认字节码形状可加载，不能拿该运行时异常当作 recovery 测试结果。 |
| 缺失 Mid | 复用原始 direct leg 文件集合，只在构造 archive 输入时过滤 `Mid.class`；不修改/重编任何 class。查询 `ownGridDirect`，其第一个 child 的类型链 `DerivedA -> Mid -> Base` 不完整。 | 结构事实仍可闭合，但层级赋值证明缺少 `Mid`，完整 `ownGridDirect` 正文应 fallback。它是 type-proof 缺失控制，不代表 child 所有权/reader 结构不闭合。现有 `missing_mid_header_refuses_only_initializers_whose_derived_path_uses_mid` 只覆盖 factory legs；新测试如需要 direct 覆盖，按此过滤 `DIRECT_JAVAC8/23` 即可。 |

类型不兼容变体是本轮最便宜、最贴近实际 child path 的 classfile 负控：新建 child 仍为 reference array，内部 `DerivedA` element store 仍是合法的 `aastore` 形状，parent 与 child 两级消费次序不动。primitive invariant 和 rank 不匹配已经有 `initializer_reference_widens` helper 的独立单测（`crates/jarde-java/src/build.rs:33586–33648`），包括 primitive array 不协变与错误 rank；不要为本片复制类型算法或增加类型表。该 helper 测试不能替代以上真实 child/parent 结构候选抵达 Builder 的 Object[] 变体。

## SSA 多读取的可表达性边界

合法 JVM 指令没有一种“不复制栈值就把同一 operand ValueId 同时交给两个指令”的表示法。为让两个后续操作都看见值，必须使用 `dup`/`dup_x*` 等复制操作，或经局部变量保存再加载；在 SSA 中复制/加载各自成为 producer，不能据此预言原 retained ValueId 的 `uses().len() > 1`。本表的 `dup; pop` 控制只主张：child retained ValueId 的 sole reader 是一条 `dup`，而它不是 child proof 认可的最终 parent-store consumer。若希望覆盖真实“同一 ValueId 多个直接 uses”的数据结构分支，只能使用现有 SSA 单测里的合成 uses 注入方式；那是单测层证据，不能伪装成由此 JVM fixture 产生的两条物理 reader。

应核对 child 证明实际在 `array_initializer_reader` 调用处拒绝，且候选未进入共同提交；不应仅断言父方法最终是 Fallback，否则不支持语法或其它更早门也会让负控变绿。

## 验证边界与最小验收集合

1. **正例分母**：三种 direct grid（`numberGridDirect`、`collectionGridDirect`、`ownGridDirect`）各在 javac8/javac23 两腿，共 6 个 method-leg；目标 class-source family 每腿仍含 `Main/Base/Mid/DerivedA/DerivedB/LocalInterface` 六个原 class。成功标准是完整 Java body、生成六类完整编译，`-Xverify:all` 执行后 stdout/stderr/exit 与对应 frozen original leg 精确相同。不得只计局部 child candidate、单个 Main 或 fixture 中的方法命名。
2. **结构负例**：父 index 乱序、child retained 值的 `dup` 非认可读取、child/parent 区间中的无归属 effect；至少断言报告里的对应 body fallback、目标 child/parent allocations 的结构位置和实际 refusal 原因。若公开 report 将结构 Site 与 Java 呈现分层，分别断言；不得把 NewRecord `presented` 字段单独解释成源码成功。
3. **类型负例**：Object[] 变体和 direct 缺 Mid 变体均要确认 child-parent store/reader 结构没有先被乱序、非法 opcode 或错误 Code 长度掩盖，然后确认完整正文不成功。Object[] 变体先在两条原 compiler leg 上构造。已有 helper 的 primitive/rank negative 继续作为 assignability 子测试，不新增 product 层 type framework。
4. **Verifier-only 确认**：语义改变的 index、Object[] 变体均不得运行。可用测试侧 `VerifyOnly` 启动器在 `java -Xverify:all` 下以隔离 classloader 对变体 `Main.class` 执行 `defineClass`/非初始化加载，并从 unchanged companion classes 解析 `Base` 等引用；启动器不调用 `Main.main` 或任何变体方法。仅当变体 class 能加载、方法字节码被 verifier 接受，才记录该负控抵达后续恢复层。若加载触发 class verification error，控制无效，应停下修正变体，不能把它记为 Builder 拒绝。

上述为拟议验收步骤，本文没有执行 class patch、SSA 分析、`-Xverify:all`、完整 compile/run 或任何变体。当前代码在本片实现前仍可能在旧 component-equality 结构门拒绝类型变体；测试应在计划中的 equality 移除之后验证最终 Builder type gate，不能提前把当前静态实现说成已满足这些预期。
