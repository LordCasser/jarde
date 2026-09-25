## 1. 冻结三方输入与直接接收者边界

- [x] 1.1 保存即时数组构造器、即时普通方法引用和先存局部的 Java 8 完整类/runner，记录 class SHA、真实 Code、原/JADX/Jarde 完整源码、编译及 `-Xverify:all` 执行；root 用冻结 CLI 在独立输出目录重放，`summary.json` 字节相同。
- [x] 1.2 将三份受控原 class 与 source-only runner 纳入永久回归输入，逐字节核对证据 SHA 和原始执行；更新并运行 reader census/corpus fingerprint，不能把机制对照当作产品输出。

## 2. 复用现有表达式目标类型

- [x] 2.1 在既有实例调用接收者构造处，仅给工厂目标类型与调用 owner 同型且可拼写的直接 Lambda/MethodReference 加现有 Cast；定向 Rust 测试证明即时数组与方法引用能生成目标类型，普通有类型局部正文不增加 cast，也没有新 AST/重新恢复。
- [x] 2.2 对目标类型缺失、不同型及不受支持的 owner 形状明确拒绝，保留工厂/调用来源；定向 Rust 测试核对真实 BCI、essential/all 正文相同、来源与输出/IR预算及取消不发布部分正常正文。

## 3. 完整类执行与 root 验收

- [x] 3.1 用实施后 CLI 原样恢复并重编三份完整类，原 class/JADX/Jarde 在 0、正数、负数上的输出和异常逐行对照；两份即时调用须无引用且可编译，局部对照须保持原行为，不允许手改输出或删除合成成员。`post-fix/` 与 root 独立输出的 `summary.json` 逐字节一致，三类均编译并验证执行，输出与原 class 相同。
- [ ] 3.2 root 独立审查 receiver 类型证明、来源和一次求值，复跑已有 lambda、方法引用、调用参数、数组构造和预算相邻测试，以及 reader census/fingerprint、fmt、严格 Clippy 与 `openspec validate type-immediate-functional-receivers --strict`；把合成 helper 命名冲突与函数体内参数适配另列，不扩大本项。
