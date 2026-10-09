# 构造数组负控制：classfile Code 变体设计

## 范围

只读检查现有 classfile patch helpers、reader API、CFG block 规则，并为 frozen `sequence()[Ljava/lang/CharSequence;` 设计 test-only byte variants。未改产品、测试或 fixture；未运行 Cargo、Git、rustfmt、Javac 或 Java。现有控件状态以 root 给出的最新冻结回放结果为背景：direct canonical 两腿成功，NestedControls 两个原 JDK legs 成功；本报告只补 fresh canonical index、真实 stack reader 与 handler 控件的构造方案。

## 最低成本的 Code 定位

不用在产品中新增 classfile 改写 API。`jarde_reader::classfile::class_facts(bytes, budget)` 返回 `ClassFacts.methods`；按 raw name/descriptor 选 `sequence()[Ljava/lang/CharSequence;` 的 `MemberHeader` 后，`method_code_facts(bytes, method, budget)` 返回 `MethodCodeFacts.code_span`。`code_span.start` 是原 class bytes 中首 opcode 的偏移，`code_span.length` 是 code array 长度。按 BCI 改 opcode 时直接读写 `bytes[code_span.start + bci]`，并先断言 BCI、opcode 与两套 javac 变体期望相符。这既定位到唯一 method，也避免在 constant pool/别的方法中误改同字节。

repo 已有的 test-only 例子有两种：

- `tests/p3_twr_return_tail.rs::code_span` 手写最小 constant-pool/member/Code 定位器，并在 exception-table byte range 插一行、同步 `Code.attribute_length`。可借其“只解析当前 fixture 需要的字段”的做法。
- `tests/p3_special_refusal.rs::unique` 对 frozen opcode pattern 做唯一命中断言，只改目标 byte；需要增长 Code 时更新 code length/attribute length。
- `tests/fixtures/p3-boolean-array-initializers/patch_class.py` 展示按 classfile envelope 找指定 method/descriptor/Code，不全局搜任意 `0xbc/0x4f`。

对本文前两类 index 反例，reader API 给出的 `code_span` 比手写 parser 更短；对添加指令/改 exception table，仍可由同一个 `code_span` 找 Code array 与紧随其后的 exception table，完全不需要增加产品 API。`MethodCodeFacts.instructions` 可给实际 opcode/width，必须以它断言 patched BCI 的语义，而不只对裸字节索引做信任。

## ① fresh canonical index：只改 Code 单字节

两腿 fixture `sequence` 的关键 layout 由当前测试 `TARGETS` 锚定：array `anewarray@1`；first element `new StringBuilder@6 / dup@9 / init@15 / aastore@18`；second element `new StringBuffer@21 / dup@24 / init@30 / aastore@33`。相应的 index producers 是 `iconst_0@5` 与 `iconst_1@20`。应在变体构造时逐条断言 opcode `0x03` 和 `0x04`，再复制原 class bytes修改。

| 控制 | Code byte 变更 | 栈/验证影响 | 被测拒绝 |
|---|---|---|---|
| 重复 index | `code[5]` 保持 `0x03`；将 `code[20]` 从 `0x04` 改为 `0x03` | `iconst_0` 与 `iconst_1` 都推 category-1 int，Code 长度、max_stack、StackMap、attribute length 不变；JVM verifier 合法。第二次写 index 0 覆盖前值是合法字节码行为。 | 第二 element 不再是 expected physical index 1；整个 fresh initializer 不闭合。应没有 pending construction Site/initializer，且原始 producer BCI 仍可回退。 |
| 逆序 index | 交换 `code[5]` 与 `code[20]` 两字节，即 first store 取 1、second store 取 0 | 同样都是 category-1 int，Code 长度/stack shape 不变。两个 store 的值反序写入，classfile 仍可结构读取并通过 verifier。 | array candidate 不能把物理顺序当作升序 initializer；应在 expected-index/physical-order 边界拒绝，而非把元素重排生成 Java initializer。 |

两种变体仅改变 method 的 Code 指令字节，常量池、descriptor、class attribute、method source 均不变；无需重算 branch target 或 exception-table BCI。测试只验证候选 refusal 和来源保留，负例输出顺序已故意改变，不要求拿它与原程序输出相同。

可复用的 test helper 形状：

```rust
let mut bytes = SEQUENCE_CLASS.to_vec();
let facts = class_facts(&bytes, &mut budget()).expect("frozen class facts");
let member = facts.methods.iter().find(|m| {
    m.name.raw().0 == b"sequence" && m.descriptor.raw().0 == b"()[Ljava/lang/CharSequence;"
}).expect("one exact method");
let code = method_code_facts(&bytes, member, &mut budget()).expect("Code facts");
let start = usize::try_from(code.code_span.start).unwrap();
assert_eq!(code.instructions.iter().find(|i| i.bci == 5).unwrap().opcode, 0x03);
assert_eq!(code.instructions.iter().find(|i| i.bci == 20).unwrap().opcode, 0x04);
// duplicate-index control: bytes[start + 20] = 0x03;
// descending control: bytes.swap(start + 5, start + 20);
```

该片段是设计草稿，不是已编译代码。不同 JDK leg 必须基于各自原始 class bytes 重新解析并做同样的精确断言；不能假定某个 JDK 的 BCI 可直接套给另一腿而不检查。

## ② 真实额外 stack reader：合法 `dup_x2` 栈形

在 first `aastore@18` 前，array element 的 operand stack 为：

```text
[array_retained, array_store_copy, index:int, completed:reference]
```

将单字节 `aastore@18 (0x53)` 替换为以下线性字节码：

```text
dup_x2  (0x5b)
aastore (0x53)
pop     (0x57)
```

JVMS `dup_x2` form 1 的输入 `..., value3, value2, value1`（三项均 category-1）输出 `..., value1, value3, value2, value1`。这里 `value1=completed reference`、`value2=index`、`value3=array_store_copy`，因此精确栈变换是：

```text
before:   [array_retained, array_store_copy, index, completed]
dup_x2:   [array_retained, completed_copy, array_store_copy, index, completed_copy]
aastore:  [array_retained, completed_copy]
pop:      [array_retained]
```

`aastore` 的三個 top operands 仍依次是 reference array、int index、reference value，且其類別與原始版本相同；第二个 `completed_copy` 由 pop 消费，array 的 retained copy 仍在原栈位由之后第二 element 的 `dup` 消费。**不要**在 `[array,index,ref]` 上简单插入 `dup`：那会变成 `[array,index,ref,ref]`，`aastore` 会把 ref 当 arrayref，栈不合法。也不要写成 `dup_x2; pop; aastore`，它会丢失原 store operands。

expected SSA use 关系应以 root 对真实 IR 的 inspect 为准，报告不捏造 ValueId 数字。按 SSA 的正常 stack-output 建模，待核对形状是：

- invokespecial 初始化后完成的那个 `ValueId` 被 `dup_x2@18` 读取；所以该对象的 site-value consumer 是 `dup_x2`，不再是 paired `aastore`。
- `dup_x2` 产生两个对象别名：上方 copy 作为 `aastore` 的 value operand；留在 retained array copy 上方的 copy 被 `pop@20` 读取。
- `aastore` 的 array/index operands 仍是 original array copy 和 original index ValueId；`aastore` value operand 是 `dup_x2` 写出的 alias，不是 invokespecial 写出的 exact completed ValueId。
- array retained ValueId 的消费与原始 sequence 一样仍由下一元素的 array `dup` 完成。

这同时建立真实额外对象读取，并使栈验证有效。最短 patch 会将一字节 store 替换为三字节序列：Code 长度 `+2`、`Code.attribute_length +2`。原方法在字符串参数求值时的峰值栈为 6 个 category-1 slots（array copy、index、未初始化 receiver 的两份 dup、string argument），新增 `dup_x2` 峰值是 5；所以这条具体变体通常不需要提高 `max_stack`，但 patch 必须从 `MethodCodeFacts.max_stack` 核对 `>= 5`，不要盲目改头。sequence 无 branch/switch，且无 exception table，后续 BCI 顺延即可。先通过 `MethodCodeFacts` 验证该方法确实无 handler/branch；Code 的 nested attributes 应断言为空（冻结 fixture为 `-g:none`），若非空需按类别更新含 BCI 的 debug/stack-map offset 或退回不增长长度的方案。`Code` 字段相对 `code_span.start` 的位置为 `max_stack = start-8`、`code_length = start-4`、Code attribute length = `start-12`（前提是 classfile shape 断言通过）。

**需区分高层路径与特定 guard：**在完整 `prove_with_composition` 上，此 `dup_x2` 位于构造完成与 store 之间，现有 `init::verify` 的 statement-free/未知 interval 扫描也可能先拒绝它；而元素存值的 SSA 定义改成 `dup_x2`，builder array walk 也可能不再把其 definition 识别成直接 `Allocate`，从而不进入 `verify_array_store` 的深层身份分支。全路径 refusal 仍是正确负向结果，但不能据此声称“paired reader guard 被击中”。若 task 要单独证明 exact reader guard，在 `init.rs` 同模块 unit test 中从变体 `MethodIrAnalysis` 取得真实 `head=6 / store=19 / stored ValueId`，直接调用私有 `verify_array_store`，并期待拒绝；再核对 refusal 是 duplicate reader/identity 边界。完整 array proof 另断言 no candidate/no pending Site。必要时用 `array_store_consumes_site_value` 的直接定向调用把 store operand 和 expected exact completion 绑定，从而避免前置 statement-free refusal遮蔽该 assertion。

如果想让 store 后还出现**另一条命名 reader 指令**，可在 `aastore` 后调用 `Object.getClass()Ljava/lang/Class;` 再 `pop`：完成上述栈变换后 `[array_retained, completed_copy]`，`invokevirtual Object.getClass` 消耗对象并留下 Class，`pop` 清掉 Class，后续栈仍 `[array_retained]`。须从 frozen Main constant pool 精确查到现有 `Methodref java/lang/Object.getClass:()Ljava/lang/Class;`，不可猜 cp index；这比单一 `pop` 额外增加 4 个 Code bytes、另需把 max_stack 加到足够值，还使 getClass 成为可能抛 NPE 的 effect。用于基本额外读取控制时，`dup_x2; aastore; pop` 更小、更纯。

## ③ handler 范围修改不必拆分 source block

`MethodCodeFacts::control_flow_targets` 会把异常表的 `handler_pc` 作为 `ControlFlowTargetKind::Handler`；当前 `crates/jarde-jvm/src/cfg.rs::leader_flags` 会将这些目标设为 block leader。但 handler 的 `start_pc`/`end_pc` 只是已验证的 protected-range 边界，并未加入 `leader_flags`；它们只要求落在合法 instruction boundary。故：

- **保护范围差异不必然把 array allocation、mark、`<init>`、aastore 切成不同 source block。**handler target（catch block）自身是 leader，正常 source run 可以仍处于单一 straight-line block。
- 只要构造、store、array 都在同一普通 block，组合 candidate 不会仅因 handler range 开始/结束而在更早的 “same block” 条件失败；`ssa.effects()` 对每个 may-throw instruction 分别携带 handlers 集，array effect closure 可看到分歧。
- 若测试源在 array run 中加入 `if`、conditional expression 或 branch，`leader_flags` 会按真实控制流切 block，组合更可能先因 block/闭合条件失败；那不是 handler effect gate 覆盖。
- 注意异常表 handler **target** 与 protected range **start/end** 是两类 BCI：target 是 CFG 边目标/leader，范围端点不是 source block leaders。不要从“新增异常边”推成“protected body 按每个 exception range 切块”。

最小可用 handler patch需要一个原生 javac handler target/StackMap frame，不能凭空在无 handler 的 `Main.sequence` 上增加一行 catch-all 而不配套合法 handler code。低成本方式是在 test-only 源/一次编译输入中准备单一方法：

```java
static Object[] handled() {
    try {
        return new Object[] { new StringBuilder(mark("handler")) };
    } catch (RuntimeException ex) {
        return null;
    }
}
```

这个最小 source 只依赖 Object、StringBuilder、一个静态 mark 和 catch，原 class 可正常编译，javac 会给合法 handler_pc 与 StackMapTable。复制编译出的 class bytes，用 `class_facts`/`method_code_facts` 定位该方法 Code 与唯一 handler record；将这一行 `start_pc` 从 array `anewarray` BCI 改为 `mark` 调用的 BCI，保留原 `end_pc`、`handler_pc`、`catch_type`。只改 exception-table entry 中 start_pc 的两个 byte；Code 长度、Code attribute长度、StackMapTable不变；start BCI 必须由 opcode facts 选取，且 `start < end`。改前 assert 原 row 覆盖 array allocation 与 mark，改后 assert allocation 无 handler、mark 与 invokespecial/aastore 在同 handler（如果 `end_pc` 覆盖它们）。

预期：classfile/IR 保持可解析；source 中 allocation到store是一条普通 straight-line block；array allocation 与 may-throw `mark` 的 handlers 向量不同。只有在 exact reader/index/site/composition checks 已通过、最终 array effect closure 不提交 initializer 时，才算 hit 了 handler gate。生成源的原始行为和 patched 行为在 allocation OOME 时有意不同；该 negative只判断 Jarde保守拒绝/来源，不宣称两 class行为等价。

可用 `code_end = code_span.start + code_span.length` 找异常表长度与 entry 起点：前 2 bytes 是 `exception_table_length`，随后每条固定 8 bytes `(start_pc,end_pc,handler_pc,catch_type)`。在只有一个目标 handler 的专用 method 上断言数量、原 tuple 与 catch type 后只 patch start_pc。若想完全避免手写 offset，在 tests 可从 `ClassFacts.methods[].attributes` 的 `Code` shell 取 content span；若只消费公开 `code_span`，例程中 Code bytes 后的 table offset 足够直接。

## Root 需核对的断言点

1. 两条 direct JDK legs 对 `sequence` 的 index BCIs/opcodes都为 5:`iconst_0`、20:`iconst_1`；canonical mismatch 只改一个/交换两个 Code bytes，frame 与其他 class bytes逐字相同。
2. 对 `dup_x2` 变体，原始 `aastore` operand tuple 是 `(array ref, int index, ref)`；运行 `analyze_method_ir` 后检查上述 alias/copy 形状与 exact use BCI，不记录或杜撰 `ValueId` 数值。该 mutation 栈合法；应检查实际 `max_stack >= 5`，原方法峰值为 6，因此此变体不增加 `max_stack`。Code/attribute length各 `+2`，所有后续BCI偏移。
3. `dup_x2` 可能被更外层 statement-free/definition gate先拒绝；要把“全候选拒绝”与“exact consumer guard测试”分成两个断言，不能把前者报告成后者。
4. handler patch 的 catch target具有正常 verifier frame；保护端点是 instruction starts；`handler_pc`成为leader但 start/end不增加 source leader。用 IR 的每条 effect handler ordinal 证实array allocation与may-throw参数/constructor/store之间确有差别，并确认refusal实际到effect-closure层。
5. `Object.getClass` 扩展变体需按method ref符号定位constant-pool index；不硬编码未核对的index。仅测试已使用 `dup_x2; aastore; pop` 时不必加入额外调用。

## 本报告不声称的证据

以上仅是源码/API和JVMS栈语义推导出的 test-construction plan。没有生成或验证变体 class bytes，没有运行 classfile analyzer/JVM，也没有确认当前 build 的具体 refusal code/SSA uses。后续落地前必须由 focused test 将 bytes/BCI/SSA use和实际 gate逐项断言；root给出的现有两腿通过记录不覆盖本报告新增的三个 controls。
