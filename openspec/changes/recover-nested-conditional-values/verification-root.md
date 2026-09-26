# 独立验收：共同 join 的嵌套条件值

2026-09-26。以冻结的 Java 8 原 class、JADX 1.5.6 结果及 runner 为基准；完整基线与哈希见 `../../evidence/java-syntax-2026-09-26/nested-conditional-value/`。实现验收从当前工作树单独构建 `jarde-cli`，未复用代理生成的输出。

| 检查 | 结果 |
| --- | --- |
| 完整类 Java 源码 | `class-source` 输出的 `nested` 与 `nestedEffects` 都成为一个嵌套 `?:` 值；整类没有 `@bytecode` 标记。 |
| Java 8 编译与运行 | `javac --release 8 -g:none` 重编整个 jarde 类及冻结 runner 成功；`java -Xverify:all` 的 15 行结果与原 class、JADX 重编版逐行相同。边界值、外/内测试效果、叶子调用次数和三个可区分异常路径均在 runner 中。 |
| 默认与完整证据 | 两种模式目标方法正文相同，`quality=structured`、`representation=java`。报告 `syntax_status=unchecked` 是引擎自身未做语法编译的状态；上面的外部 `javac` 已验证输出。 |
| 来源映射 | `nested(I)I` 覆盖期望的 14/14 个指令 BCI，`nestedEffects(ILjava/lang/StringBuilder;)I` 覆盖 30/30 个；无多余或缺失 BCI。 |
| 正反例 | 三叶真臂、四叶假臂、效果与异常正例恢复；外部入边、独立效果、未知公共引用类型和重复消费者保持保守结果。四个新增 fixture 的 class 文件均为 Java 8 major 52。 |
| 回归 | `cargo test -p jarde-java --tests` 全部通过，含 203 个库测试以及条件值、短路、循环、异常等集成测试；定向相邻四套测试为 10/10。`cargo fmt --all -- --check` 与 `openspec validate recover-nested-conditional-values --strict` 通过。 |

本证明只处理所有直线叶子直接流向同一个最终 join、且最终栈 Phi 有一个真实消费者的条件树。中途汇合后再参与另一汇合、不能证明的类型或其他独立语句仍按现有回退规则处理。
