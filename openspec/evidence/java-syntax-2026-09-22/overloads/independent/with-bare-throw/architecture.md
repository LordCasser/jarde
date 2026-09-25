# 裸 throw 的独立架构缺口

本目录 InvocationAudit.mark 的原class为计数写入、条件、new IllegalArgumentException、dup、invokespecial、athrow。当前输出引用new/dup/构造调用/athrow，另一臂返回String，因此实际javac报告缺少返回语句。这个失败不归调用参数修复。

当前代码的三个边界：

- Operation::Throw 已存在，但 build::instruction 把未被guard认领的Throw与Monitor一起引用，只用vec![at]。
- init::sites 的现有 consumer reader 列表不含Throw；所以即使引入throw语句，也要在现有new规则内允许唯一throw消费，不能绕过new/dup/constructor证据直接拼文本。
- StmtKind 当前没有独立Throw，guard的handler rethrow通过整个try/finally/synchronized形状呈现。本项可能需要一个忠实的语句节点，不能把throw伪装为Expr字符串；不需要新的区域机制。

下一轮先独立构造throw null、参数异常、new异常、返回异常的调用、显式cast和保护块中的throw，对照效果、异常对象身份、先后求值及引用来源。Throw应继续尊重guard已经认领的synthetic rethrow，不扩张保护区准入；消费失败沿既有quoted_bcis保留生产者，不能只引用athrow后丢失其来源。类型与checked-exception声明的限制需要实测后写spec，当前仅登记，不实施。
