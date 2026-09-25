## Context

证据见 `../../evidence/java-syntax-2026-09-22/initialization-final/expanded/`。四个普通类的五次原 class/JADX 执行一致；jarde 完整类被 javac 拒绝，错误集中在限定的 final 静态字段赋值。`FinalLocalCollision` 同时声明 `local0` 与 `local0_2`，恢复正文恰好把原 seed 叫作 `local0`。

当前 field plan 已证明指令字段身份与读写操作数；build::field_write 总把静态 owner 写成 Path 接收者。FieldAssign 强制接收者，Assign 明确表示局部赋值。NameTable 只避开局部名，free_name 同样如此。MethodIr 已借用同一 Arc<ClassFacts> 的方法头，字段头也在该已有事实里，无需重读。

## Goals / Non-Goals

**Goals:** 将已证明的当前普通类 blank static final 写入原地呈现为简单字段赋值，使局部和合成名称不能改变该字段绑定；保持来源和停止契约。

**Non-Goals:** 不新增 definite-assignment 分析，不修复重复写 final 等无合法 Java 对应的 class；不恢复接口的声明初始化、枚举源码或把初始化效果搬到字段声明。不扩展一般字段解析、名称语法和控制流准入。

## Decisions

1. 从 MethodIr 的同一类头借用字段声明，与已有方法头视图一致；按原始 owner、name、descriptor 与当前声明核对，不复制整个 ClassFacts，不读其它 body。只有已知普通类的 `<clinit>()V`、owner 为该类、唯一且 Java 可拼写的字段名、匹配 descriptor、ACC_STATIC|ACC_FINAL 且无 ConstantValue 壳时适用。缺字段声明、同名歧义、接口/注解/枚举或不匹配事实不产生该新判断；其它字段写保留已有行为。ConstantValue 的缺席可由已经读过的属性外壳确定，不重新解析其值。
2. 在 FieldAssign 内用可选 receiver 表达简单字段名：Some 保持既有带限定符写入，None 是已经证明绑定的当前字段。只为此受限写入生成 None；不使用空 Path、不删字符串前缀、不把字段写改称局部 Assign，也不新建 FieldTarget 层级。既有遍历与 emitter 适配这个有限变化，字段名仍保留字段写的真实 origin。
3. 将本项必须简单命名的字段名交给现有 NameTable 命名决策，在分配局部与 free_name 时一起避让。沿用现有确定性后缀与 alias 契约，`local0` 与 `local0_2` 均预占，不能选一个已占后缀；不先产生 AST 再全树重命名。候选名是当前方法所需的声明约束，不能伪造 LocalVariable 或 debug evidence。未给定这些约束的普通方法继续原路径。
4. 命名预占与字段写判断必须读取一致的前提，不能一个只看字符串名字、另一个看字段身份。可以使用现有字段/声明计划的小查询，把所需名称交给命名；不增加 pass、全图 registry、跨方法缓存或独立 resolver。原指令处的值消费、类型转换与失败生产者追溯沿用 field_write，不移动到声明也不重排多个写入。
5. 复用同一发射入口处理有/无 receiver，文本与 source map replay 一致；节点计数不变，按真实字节与来源收费。默认证据选择不构造来源，来源预算耗尽保留已提交正文。borrowed header 不增加重复解析或 owning 类头副本。
6. 无需引入库。缺口是已有 AST 的一种赋值拼写与既有名称分配的约束输入，外部解析/反编译库不能提供更强的当前类事实；新增依赖反而增加维护、许可和预算成本。受控 javac/java 是验收 oracle，不改变产品 parse、dialect、runtime、verification 或 compilation 平面的承诺。

依据：[JLS 8.3.1.2](https://docs.oracle.com/javase/specs/jls/se8/html/jls-8.html#jls-8.3.1.2)、[JLS 16](https://docs.oracle.com/javase/specs/jls/se8/html/jls-16.html)。接口的 [JLS 9.3](https://docs.oracle.com/javase/specs/jls/se8/html/jls-9.html#jls-9.3) 要求及实际失败单列，不由本项上提初始化器。

## Risks / Trade-offs

- 去限定后写到局部 → 先预占字段名，覆盖无 debug、debug 范围名和已占后缀；重编译原样完整类并比较值/计数。
- 仅比较 owner 文本就选错成员 → 同源声明的 flags、原始名称/descriptor 与唯一性一起核对；缺失声明和同名歧义为负面对照。
- 为编译而移动初始化 → 只变左侧 spelling；顺序、分支及先赋后读样例验证调用与写入位置。
- 名称约束漏掉合成名或来源不同步 → 使用同一个 NameTable 查询入口及同一个 FieldAssign emitter；相邻 accessor/field/lambda 和预算测试验收。
- 接口失败混入普通类修复 → 接口保留原样证据并另列后续任务，不把不能写 static 块的问题当作去限定可解决。
- 与 throw/numeric 修改同文件 → 生产实现等 root 明确移交；夹具可独立准备。
