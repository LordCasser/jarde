# R9 拒绝消费者回归

普通 checkcast 已可恢复，因此原测试不能继续把它当作拒绝消费者。`tests/p3_eval_context.rs::cast_result_is_popped` 对三个唯一完整 Code 末尾的 areturn，插入 pop、aconst_null，保留方法 String 返回描述符，并给 Code 和 attribute_length 加2。各原始读取/转换 BCI 不变，永久 class 未修改。

主代理复跑 10 项 eval-context 测试通过；三个内存变体继续保留静态字段初始化、实例 NPE 和两级字段生产者的 quote 与 source map。临时同时应用三个 patch 的 class 在 `/tmp/jarde-refused-pop.d8m6dq`，2026-09-23 主代理用 `java -Xverify:all -cp /tmp/jarde-refused-pop.d8m6dq Run` 复核，输出见 java-verify.txt；runner 为本目录 Run.java。永久来源为 tests/fixtures/p3-refused-cast/v8/*.class，精确 patch 与长度检查由 Rust 测试记录。
