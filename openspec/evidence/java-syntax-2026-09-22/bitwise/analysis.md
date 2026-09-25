# 位运算及非短路 boolean 运算

2026-09-23，用当前数值比较实现后的 debug CLI 独立巡查。自写 BitwiseAudit 有15个语法方法和构造器；只在临时目录编译class，没有扩大永久语料。`run_audit.py` 保存完整三方路径，JADX的包名及方法正文保持原样，只为source-only helper/runner加相同包名。

## 已验证结果

原class通过 `java -Xverify:all`。int/long边界、混合分组、补码、三个boolean运算、与true异或、boolean分支及两侧调用顺序/抛错共211行。JADX完整输出javac成功，211行完全一致；jarde有32处引用，整类javac失败。helper左/右抛错分别保留trace1/12；左值false时普通`&`仍执行右侧。

`mixed-types/`另有合法字节码反例：把唯一的 `(II)I` descriptor 精确改成 `(ZI)I`，不改iand。原patched class通过JVM校验，8行输出已归档。朴素地写成 `boolean a & int b` 被javac拒绝；`naive-hypothesis.java.txt`是独立假设验证，绝非jarde实际输出。未来恢复必须拒绝这种Java无法直接表示的混合类型，不能因frame都为Int便统一输出整数位运算。

## 与架构的对应

- decode当前未把0x7e..0x83交接为受支持事实，因而build看见Other。现有Binary表达式可承载`& ^ |`，无需新建整类AST或引入外部求解器。
- `ast::binary_type`当前把所有非算术BinaryOp都判为boolean。这只适用于当前集合；增加位运算必须显式分开整数与boolean结果，不能只在enum和spell中加三个字符。
- emitter已有统一优先级和操作数分组。扩展这个表即可保留`(a | b) ^ (a & (b + 1))`，不应逐运算符另写括号规则。
- `boolean_proof`目前是descriptor/参数/字段/数组读取的叶子判据；局部声明和最终boolean消费均依赖它。直接boolean位运算、嵌套值和存局部后的读取必须在同一类型判断上闭合，不能只令return渲染成功却让预先声明的局部类型仍为int。
- 0/1不是独立的boolean证据；只有某一侧已被证明boolean，或已有合法消费上下文要求boolean时，才能讨论另一侧literal的拼写。混合descriptor负例不能被上下文强行改类型。共享判据如何有界传播须在实施前继续审查；这里没有授权创建无界递归类型求解或第二套全图索引。
- 补码源码在本例编译为与-1异或。忠实恢复`a ^ -1`即可表达语义，无需为了复原`~a`再增加优化机制。与true异或也不应在此强制化简为`!a`。
- 两侧调用沿用既有最终消费位置和失败生产者来源，不能生成两个独立调用后又在表达式中重复调用。移位、0/1控制流汇合、自增、一般纯值优化均独立处理。

整数与boolean类型、操作数求值及三个不同优先级依据[JLS 15.22](https://docs.oracle.com/javase/specs/jls/se23/html/jls-15.html#jls-15.22)与[JLS 15.7.2](https://docs.oracle.com/javase/specs/jls/se23/html/jls-15.html#jls-15.7.2)。

补充的`type-flow/`包含nested/copied/hoisted/passed、boolean数组元素及byte/char提升，共56行；原class与JADX完整类一致，jarde有24处引用且javac失败。架构复核后，选择扩展现有共享boolean查询及decide_types工作队列的组合依赖，避免另建类型pass；局部依赖获证后必须重新证明整个表达式，不能沿一条边就把混合类型认成boolean。具体约束及预算验收已写入`recover-bitwise-expressions`，仍未实施。
