## Context

`narrow-locals/` 的 NarrowLocals 为297 bytes、5 Code、SHA `833415dd5400ba5e477e659379b0bdadad8df0bc5c903cb7a9706467a9946368`。20项原/JADX一致，jarde三个返回引用、完整javac失败。方法没有显式转换opcode。decide_types 将非boolean局部按frame呈现为int；return_expr只调用普通Java位置的widening规则。

root 的 `return-sinks-core/` 为411 bytes、4 Code、SHA `dbb7b8cecab3df4d6fcb207cf4db7bbe9337af73d8ceeba8df9f9926087002d5`，仅将唯一CP descriptor的返回 I 等长改为 B，Code不变。原class经过验证执行19项，包含int字段前/后自增及同步返回。当前jarde字段自增零引用却输出不合法的窄返回，同步返回引用；JADX也无法完整重编译。`return-sinks/` 另有条件stack phi边界，不能把其31项全部作为本项必须恢复的正例。

Operation::Return 合并了返回家族，但既有 SsaInstruction 已提供 effective opcode；不必新增事实表才能识别实际0xac。本方法return_type已同源读取descriptor。普通return、synchronized_return、switch_returns和直接生成Stmt的increment_statement都必须审读；只改一个普通路径不闭合。

本地 JADX `2fb1b163` 的 `InsnGen` 对 RETURN 直接打印 `return ` 与已有参数表达式；它的类型推断可确定返回值候选，但不会据 JVM `ireturn` 加上 B/C/S 的源码窄化。下文 verifier-valid 的三方对照中，JADX 完整类因此无法重编；这个结论限定于这些输入。Jarde 应在返回消费入口保留 descriptor 与真实指令的组合证明，不把 producer 的通用类型改成窄类型，也不照搬无转换的输出步骤。

Luna补查的`return-narrowing/`仅将四个极小类的唯一UTF8 `(I)I`改为`(I)B/C/S/Z`，四份Code前后hash不变、均通过JVM验证并各执行13个边界输入。B/C/S合计39项纳入正面验收，Z的13项固定独立最低位边界。JADX四类完整源码均javac失败；jarde四类各1引用、完整javac失败。此次CLI为`feed5c377a439490efb12a2a46bdfa0e94c32dffd3a83bd1583b70fe89317350`，不与先前7527基线混称同一构建。

## Goals / Non-Goals

**Goals:** 在返回位置表达已由指令和descriptor说明的B/C/S窄化，覆盖当前可呈现的值和返回结构，复用Cast/Type/来源与停止通道。

**Non-Goals:** 不恢复原窄局部声明、不做范围推导或转换优化、不改普通赋值/字段存储/调用参数规则、不扩展stack phi或三元表达式、不在本项支持Z返回的任意整数最低位转换。显式15条conversion opcode另案；本项无需等待它们才能处理无conversion指令的返回。生产在共享求值顺序实现完成后串行接入。

## Decisions

1. 返回专属适配以同源真实ireturn、方法返回Type和已呈现操作数Type为前提。B/C/S目标接受byte/char/short/int的整数值，使用现有Cast表达必要的窄化；同型、合法widening及既有可表示常量拼写可保留。禁止把一般meeting_position放宽为任意narrowing，否则赋值与调用会获得字节码未陈述的转换。
2. 不重解释原始Code，不扩充SSA栈种类。读取现有SsaInstruction有效opcode和原MethodFacts返回类型即可；ireturn的frame仍为JVM Int。没有可呈现类型、boolean或不相容输入不得靠覆盖presented标签通过。
3. 将已呈现Expr的返回位置适配作为一个小型共享消费步骤。普通render_value仍使用实际呈现位置；switch_returns当前传入arm位置，真正返回在join_bci，必须显式区分两者，不能把返回来源冒充操作数求值点，也不能用arm opcode判定是否ireturn。同步路径保留原锁区域，不改变monitor/异常处理。
4. 字段自增快捷路径已构造了带field_type和原anchors的update表达式；让该表达式经过同一返回适配，不重读字段、不把值拆成第二次求值、不修改字段存储类型。`return (byte) this.value++`应先按原int字段更新，再窄化返回的旧值；前置形态同理。无需趁机建立新的increment AST体系。
5. 新Cast来源关联实际ireturn并保留操作数原anchors；switch下推仍保留join的真实return BCI。节点、来源及任何新增查询受既有深度/IR/输出/取消预算；同一已决定AST供默认、all与replay使用，不加独立缓存或预求值器。
6. Z返回仍执行既有boolean证明。JVMS对一般整数Z返回规定value最低位，不能写成value!=0，也不能当作数值cast；该能力与位运算表达支持另行闭合。合法boolean操作数返回B/C/S的字节码也不在本项强行转数值。未知phi保持已知拒绝，不能以返回签名代替值证明。
7. 不引入依赖。既有Cast、类型拼写、表达式来源和SSA事实足够；外部求解/字节码库不能补充这里缺少的简单消费规则，维护与许可成本无收益。自写fixture执行是验收授权，不改变产品parse/dialect/runtime/verification或项目级可编译承诺。

依据：[JVMS ireturn](https://docs.oracle.com/javase/specs/jvms/se23/html/jvms-6.html#jvms-6.5.ireturn) 与 [JLS narrowing primitive conversions](https://docs.oracle.com/javase/specs/jls/se23/html/jls-5.html#jls-5.1.3)。

## Risks / Trade-offs

- 只补普通return漏掉快捷结构 → 字段自增、同步及已有switch下推分别固定实测/回归，不用手工修补输出通过javac。
- 原局部声明被扩大成类型推理 → 保留int局部，转换只在实际返回位置陈述；无需声明源代码原本就是这种写法。
- Cast提前改变字段更新 → 同时比较返回值与更新后完整int字段值，覆盖127/128、-129及大值。
- 把任意Z返回当非零 → 合法descriptor变体作为独立边界，不能用不合法bytecode假证保守性。
- 同步/stack phi债务导致验收范围失控 → Core完整类去掉条件phi；原边界输入和失败日志另存，不能删掉失败方法后声称原类验收成功。

## 永久 fixture 与 root 基线

NarrowIntegerReturns为1288B/14Code，SHA `90219c1ac7c93b53bee50da43ac792cd7e628dba4bff7b0f697a8b64a9b5d14c`。独立重编译并追加UTF8 descriptor、按method_info定点修改12个返回descriptor；全部Code逐字节不变。49项原class实际执行覆盖direct B/C/S、窄局部回读、int control、字段前后更新、3个同步返回。root与worker各自验证生成class相等，JADX与jarde均停在完整javac失败阶段；Rust预实现RED确认在directByte的Mixed，而非测试编译错误。该49项与既有20/19/39项分别保留，恢复后都需真实运行。另有 source-only 的真实 stack-join 阶段证据：`actual-stack-join` 通过 major-49、无 StackMapTable 的合法 Code patch 证明三臂共用单条 join `ireturn`，原 `(I)I` 的 jarde/JADX/JVM 全通过；B/C/S 单 descriptor 变体各为379B，JVM/JADX通过而当前 jarde保留1个真实quote并因缺少返回语句失败。该证据确认 switch 的 arm 呈现位置与 join return anchor 必须分开，但尚未完成永久 Rust 测试，因此 task1.2 仍保持未勾选。
