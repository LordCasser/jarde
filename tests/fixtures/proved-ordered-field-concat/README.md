# 有序字段拼接 fixture

正向整类和拒绝形状分别保存；四组 frozen class 来自 javac8/23 的 debug/none，统一 source8/target8。原始 JVM 的六lane对照另包含 javac23 默认 target 的 debug/none，不将它们伪装为 Java8 class。

正向 Runner 记录普通/volatile实例字段、Object转换修改后续字段、null/char[]的Object overload，以及调用 instanceFields(null) 时的NPE。NegativeRunner 记录static、receiver call、void effect及字段值保存到local后复用的行为；最后一种不证明getfield SSA结果有多个直接reader。

manifest记录源码、class和原始执行。运行原始class通过不表示Jarde候选已验收；生成源码必须整类无修改编译执行。null补充首轮期间共享Runner发生修改，闭包不一致已明确拒绝；第二轮源码闭包保持一致并独立核全部raw。永久证据见子变更results。
