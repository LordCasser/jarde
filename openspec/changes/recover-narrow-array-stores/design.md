## Context

`array_write` 已按数组、下标、值的顺序用真实写入 BCI 呈现三个操作数，再以 `array_element` 给出的元素类型调用普通 `meeting_position`。decode 将 int-sized array store 合并为 Int 家族；SSA instruction 仍有实际 opcode，数组自身事实才能区分 bastore 的 byte[] 与 boolean[]。因此缺的是消费位置陈述指令转换，无需另一套类型分析。

本地 JADX `2fb1b163` 的 `TypeUpdate.arrayPutListener` 将数组元素类型传播给写入操作数，`TypeSearch.addUsageTypeCandidates` 也可从写入值反推数组候选；`InsnGen` 的 APUT 分支直接打印 `array[index] = value`。这些步骤有助于确定两端类型，却没有为 JVM 的 `bastore`/`castore`/`sastore` 截断建立源码表达式。`core-bcs` 的 JADX 完整类无法由 `javac` 编译，与这条路径一致；这是针对该样本的归因，不把所有 JADX 数组输出概括为错误。Jarde 复用数组自身的类型事实，只在匹配的真实写入指令消费位置加 Cast，避免把值生产者全局改型。

`narrow-array-stores/` 的13Code主类从 int[] 源码生成，8个 method descriptor 与各自单个 iastore 被精确改为 B/C/S/Z 数组写入，Code长度、栈和局部数量不变。patched SHA `8bbb683045fdc2a85ee376383515920331fd15c53710176e447ba93f0d04004b`。root独立重编译/patch/JVM验证196项，与worker冻结字节相同；CLI948a输出12引用、整类编译并运行196行，其中186行不同。该结果是已标记拒绝导致的部分正文，不能称作零引用的静默误恢复。JADX整类javac失败，未执行。

196项包含Z写入和普通boolean局部回读边界，不能要求本案让这整个类零引用。root在输入源码阶段独立生成 `core-bcs/`，移除Z相关三个方法并重新编译/patch，得到760B/10Code、SHA `a3464f4b62da257e4cd4ff70970a05360475e51502a938c675392b334b359803` 的147项完整类正面输入；当前9处引用、147项全部不同，JADX完整javac失败；原196项不删减，保留边界。

## Goals / Non-Goals

**Goals:** 对实际0x54/0x55/0x56且元素类型分别已证明为Byte/Char/Short的数组写入，恢复已呈现整数值的截断，保持一次求值、异常和来源。

**Non-Goals:** 不从bastore独自猜B/Z，不让JVM Int覆盖源码Boolean类型；不精化局部类型，不推导值域，不折叠数组初始化。Z最低位、字段写入、返回和显式conversion分别闭合，不扩展phi、重复stack消费或已有无法呈现的生产者。

## Decisions

1. 以写入instruction的实际opcode、已有数组元素事实、值的presented类型共同准入。值为Byte/Char/Short/Int时，用现有Cast表达必要窄化；同型、合法widening及已可表示常量保留现有拼写。未能证明元素类型或值类型时继续来源完整拒绝，不凭验证器Int家族改标签。
2. 转换属于此数组写入。保持普通meeting_position的赋值/调用契约；若窄返回已落地且存在同样的整数Cast小函数，可复用其纯表达式构造，但opcode/目标类型的授权仍由各自消费入口给出。不要建立通用ConversionContext枚举、注册器或新pipeline来共享几行代码。
3. 三个操作数仍在array_write的最终at上下文中呈现。Cast只包住值，不提前数组检查、不复制下标或生产者、不把null/越界检查移动到值生产之前。支持值穿过独立语句的情形沿用已验收的deferred binding，不增第二套保存设施。
4. B/C/S强制转换关联真实array store BCI，同时保留数组、下标、值的来源及现有派生链。新增节点和来源使用现有push/预算/取消通道；默认/all与replay使用同一AST决定。
   root的`root-948a/refusal-origins.json`还固定了既有拒绝缺口：三个`store*Produced`的真实call@4/store@7，完整source-map仅有7/8；Z分支则正确保留4/7/8。数字类型不相容分支目前仅fallback(vec![at])，应沿现有quoted_bcis追溯，使本项仍拒绝的合法边界也保留已延期生产者。不得新增另一套来源图。
5. Z保留原boolean证明。一般整数到boolean[]的最低位规则不同于value!=0，也不是Java数值cast。普通boolean局部推导不足单列债务，不能为让全类测试变绿而猜类型。core-bcs作为正例输入的裁剪发生在javac前，绝不编辑恢复输出。
6. 不增加依赖。现有类型事实与Cast/ArrayWrite已足够；外部解析或求解库不能补充这里的指令消费语义，维护和许可成本没有收益。测试执行仅针对自建样本，不改变产品parse/dialect/runtime/verification承诺。

依据：[JVMS bastore](https://docs.oracle.com/javase/specs/jvms/se23/html/jvms-6.html#jvms-6.5.bastore)、[castore](https://docs.oracle.com/javase/specs/jvms/se23/html/jvms-6.html#jvms-6.5.castore)、[sastore](https://docs.oracle.com/javase/specs/jvms/se23/html/jvms-6.html#jvms-6.5.sastore)。

## Risks / Trade-offs

- 把bastore一律当byte → 对B/Z分别固定合法输入，准入依赖数组元素事实。
- 值生产提前或重复 → 对null/越界与生产者异常组合比较调用次数、异常身份和数组原值，并补数组/下标自身有副作用的顺序对照。
- 误把Z和未支持生产者列作必须恢复 → B/C/S完整类独立验收，保留原始196项作为边界档案。
- 通用赋值规则被放宽 → 相邻field/return/invocation测试保持原合同，本项转换只由匹配指令授权。
