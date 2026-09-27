# EM-27 主线独立验收

root 在主线合并 `recover-proved-inline-string-char-array` 后重新构建 CLI，SHA-256 `182ff47c9526e32583c4d75a44fe722c310cadadc8d0267e6afa38072a520615`，独立运行固定 [replay.py](replay.py)。固定 JADX `2fb1b16386941660fda07e9017285aec40fcb37f` 的七份测试哈希不变；原 class、JADX、Jarde 完整 Java 8 类源码均重编并通过 `java -Xverify:all`。八行结果中，Jarde 与原 class 逐字一致，最后两行身份比较为 `false/false`；固定 JADX 为 `true/true`，因此其省掉显式 `new String(char[])` 的文本不能作为语义目标。root 生成的 Jarde 源码含 `return new java.lang.String(new char[]{'a', 'b', 'c'});`，先存局部数组的构造仍保持显式 `new`；源码与实现代理证据逐字一致。完整源码、日志和摘要保存于 [root-replay/](root-replay/)。

代码审查确认数组证明在字段计划之后、构造站点之前建立一次，两个消费者读取同一证书；内嵌准入要求 Java 8、准确 String `[C` 构造、常量 char[] 完整区间和直接返回，SSA 值和异常范围闭合。`cargo test -p jarde-java`、`cargo test -p jarde-java --test em27_inline_char_array`、`cargo test -p jarde --test class_source`（82 项）、`cargo check --workspace`、格式检查与 OpenSpec strict 均通过。`byte[]`/charset、数组别名和突变、额外效果、其它构造器及 Java 11 拼接仍在本切片外；EM-27 整单元继续扩验。
