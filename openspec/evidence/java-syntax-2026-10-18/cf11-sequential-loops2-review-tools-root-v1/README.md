# Seq2 工具审查与下一固定项静态备忘

归档22成员，SHA256 `042ccb672013f46c1b453c91b9572d9cb310e8eeda4b6eaea4cf36a806f35962`，包含 collector/reviewer 各冻结版本、root-v5 参数遮蔽修复、真实失败历史说明、构建来源绑定及两份下一固定项静态备忘。root-v4 草稿在写脚本前失败，没有可接受脚本。最终 root-v5 实际执行并接受矩阵；旧版本不冒称通过。

静态源码表明当前 foreach helper 已支持 Iterator 的声明/初始化分离和 Elevated owner；永久正例已覆盖，不再据旧 same-IR 首拒发明同类机制。Object foreach 加 String cast 与 typed String header 是不同呈现问题，须看当前实际输出。历史 `TestNestedLoops` 接受的是完整选定转写类，不是原始完整外层 JUnit 源码；备忘中的历史摘要受此范围限制。下一项准备完整上游原输入/三方输出并取当前证明路径，不因历史行为通过而忽略黄金断言的源码特性。

本目录只保存工具和静态分析，不单独构成当前 `TestNestedLoops` 编译、运行或 Reader 首拒证据。
