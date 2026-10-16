# Design

## Context

动机见proposal。独立控制基于原FX的finReturn/c0完整源码切片，只改外围类名为CallReturnFinally：12条实际命令中两份原class四次运行均匹配217B oracle，两份完整生成Java均missing return（line19），没有rebuilt运行。实际Facts/Region探针为2命令/1测试，非产品实现。

两编译器的方法均127B、60条指令、7 canonical块、8条边，两异常row为[17,30)→88与[88,90)→88。自然循环header7、blocks{7,12,59}、latch59；return块23在循环scope外，terminal return/body为空。23→88为实际异常边，12→88不存在。c0@25 may_throw且handlers[0]；cleanup各调用may_throw但handlers[]。不能借用NoCall的不抛异常体前提。

## Goals / Non-Goals

**Goals:** 为这个固定CFG建立完整私有证书，复用既有Plan/Finally/SavedReturn与loop_header授权；在真实反例、停止、公共入口和全源码运行中验证边界。

**Non-Goals:** 不扩大cleanup_sequence的单调用grammar，不引入循环注册器、通用构造器/effect等价框架、公共enum或依赖；不执行或解析c0体来推测调用不会抛出。不把parse/dialect/runtime selection/verification或Structured标志当源码语义证据。

## Decisions

1. 使用专用固定proof。有限BCI/opcode/拓扑/row admission后，对指令、phi、值和使用逐项charge；完整核对Operation/CP符号、块成员/末端、8条canonical边、loop membership、每指令effects与SSA。沿用原IR所有权与预算，不重跑P2。现有库已提供所需事实与发射能力；外部库不能提供本项目物理身份绑定证书，不新增许可/维护负担。相比放宽通用grammar，专用证明使此次接受边界明确。
2. 证书覆盖lead12..17、body17..30、savedStore28/return58、cleanup30..56；fallthrough59..85之后transfer85至Tail119..125。owned为{12,23,59,88}，join/header7。复用私有loop_header与当前region额外返回授权，handler88只在成功后认领；不把handler加入正常child scope。尾部iinc119/goto122融合于真实block59，不能假设独立tail块。
3. 三份10操作cleanup分别30/59/90起始。按每copy内部SSA映射核对System.out、Allocate(StringBuilder)、Duplicate双未初始化alias、Special构造器初始化写回、"fin:"、append(String)、当前x、append(int)、toString、println。精确核类型/slot/producer/consumers及effect；NewSite按各copy物理分配位置对应，不能把三次分配当同一个对象，也不把全输出单消费者当必要假设（实际dup一个alias被构造器覆盖且uses为空）。saved call→store→load→return链则必须唯一。
4. cleanup读取的x均resolve到store15的值；handler local4 phi输入[x,自身]且replaced_by x，要核实际phi与替换证据。异常stack phi有来自23与88的caught inputs，store88→load116→athrow118保持主异常身份。完整核对自绑定row非抛出、c0受保护、清理均在保护区外，保持清理异常替代pending completion。两row的finally_copy_shape=true仅局部形状；现有cleanup_sequence=false也不等于已观察到旧整体proof的首拒点。
5. 真实class变体反例在重新analyze后的不可伪造Facts中拒绝：副本参数/CP/receiver/slot、saved return、构造器alias、分支/尾更新/保护边界、异常存取等。证明收费中触发晚预算与取消，确保停止传播且未发布ownership，保留已收费量。
6. 最终冻结CLI对每编译器独立完整class恢复；每份report.text原样重编，driver rebuilt-only编译并双JVM运行。固定driver只观察cleanup顺序/计数/异常和返回值，不直接观察纯c0调用次数，也未触达try内异常；这两点由证明/来源覆盖，报告必须限定。原完整FX可仍有finContinue/finBreak失败，保留原样结果，不能删除它们使整类编译过关。

## Risks / Trade-offs

- [构造器别名误配或只比opcode造成伪等价] → 精确内部SSA/类型/NewSite与完整消费者矩阵，副本反例必须拒绝。
- [错误扩大catch-all导致清理递归或异常优先级改变] → 逐指令handler集合、完整两row/8edges与异常值链验证，driver检查清理异常返回/继续路径。
- [scope扩张认领未证区域] → 仅复用已有限定的额外return授权，失败保持coverage/visited回滚与未提交产物原子性。
- [固定样例成功被扩大为通用恢复率] → 保留原FX整类失败及能力边界，不改旧P5计费表。

## Migration Plan

先交付NoCall自身CI/归档主线，再让本分支承接主线文档；产品代码按独立fixture/proof/公共入口顺序提交。验证完成后仅用本片产品exact-head自身CI，正常同步新增规格并归档；回退可回滚本片提交，不影响已有窄规则。
