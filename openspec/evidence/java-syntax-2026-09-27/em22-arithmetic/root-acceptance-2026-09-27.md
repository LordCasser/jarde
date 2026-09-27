# EM-22 布尔异或常量主线验收

Luna 修订提交 `fee83c3ea67edabbda814193429fcdc241d33f46` 合并到主线 `a95d26ca` 后，root 构建 `jarde-cli` SHA-256 `b6584a693d03266fc3ddddc7c5506d273b38acd4b0f7e0a9ae355c7e20f3fdea` 并独立执行固定三方重放，结果在 `/tmp/jarde-em22-root-accept/`。摘要除 CLI SHA 外与[实现证据](implementation-replay/summary.json)逐字段一致，JADX/Jarde 源码逐字节一致。原/JADX/Jarde 全类 Java 8 重编及 `java -Xverify:all` 均通过；八行输出及 eager `|`、布尔调用次数一致。

只在 `ixor` 的完整 SSA 双读单写、左值独立布尔证据、右值为唯一消费的 `0`/`1` 常量时化简。`^ true` 写已有 `Not` AST，`^ false` 保留原左表达式及其类型证据；XOR 和省略常量的 BCI 均保留来源。root 审核发现最初 `^ false` 重建表达式会丢掉 `presented` 类型，子代理在提交前修正，并增加嵌套布尔 `&` 消费者测试。数值 XOR、变量 XOR、证据不足、eager `|` 与预算/取消走原有路径。

合并态 `class_source` 82/82、EM-22 定向测试 3/3、`p3_long_field_assignment_result` 3/3、`p3_bitwise` 常规 3/3 与需 JDK 的 2/2 均通过；workspace check、格式检查、严格 OpenSpec 校验和 diff 检查通过。算术其它 lowering 与 Smali 专有取反断言仍待单独验收。
