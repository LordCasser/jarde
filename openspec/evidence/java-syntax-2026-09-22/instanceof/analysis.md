# instanceof 独立巡查

自写 InstanceOfAudit 的10个方法覆盖对象/类/接口/原始数组/引用数组/多维数组、一次性调用、boolean局部、分支消费和否定值。javac23.0.1 --release8 -g:none生成同一class，driver对8类输入（含null）逐项比较，共80行；class仅放/tmp/jarde-instanceof-audit。

当前debug jarde整类输出有22处bytecode引用，实际javac拒绝（日志保留）。jadx1.5.6整类实际编译并通过80行对照，其自动defpackage以反射runner参数选择，无手改恢复正文。

opcode instanceof当前未收成独立可读的Operation，AST也无类型测试。它与checkcast含义不同，不能靠现有Cast伪装；后续可能需要一个忠实事实和一个boolean表达式节点，但不需要新恢复pass或类型层次resolver。类型拼写可沿用显式cast的spell_reference，consumer和quoted生产者路径应复用。

boolean局部与return还必须扩展现有boolean证明的事实叶子，避免把JVM int形状直接写成Java int；条件的0比较要按已有boolean条件极性处理。negated编译成0/1汇合，另受boolean merge缺口影响，不能因支持instanceof opcode就宣称这一个方法能恢复。保护块/循环准入不应随该小项扩大。来源和单次调用需逐BCI核查，失败时保留被延期操作数；先完成正在实施的调用参数/初始化以及已规划数值比较，再为此项建独立spec。

后续 `type-boundaries/` 又固定了六个测试方法、11行原class输出：Object加宽在字节码中消失，直接恢复会产生不合法的 String instanceof Integer 等文本；实际jadx也有四处javac错误。命名方法引用还需保留已有函数式工厂目标。方案必须保留可成立的Java操作数静态类型上下文，可复用现有Cast安全加宽Object，无需继承resolver；不能把类型测试折叠为常量而丢掉调用。此前80行仅覆盖Object参数，不能替代此边界。`recover-instanceof-expressions` 四份规划工件已经完成并通过strict，生产仍等待已排队修改交接。
