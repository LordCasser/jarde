## 1. 前置证明与声明边界

- [x] 1.1 确认 DT-16 的同轮 `NullLiteral` 候选已独立验收，消费现有候选而不复制 Code/SSA/AST 扫描
- [x] 1.2 在普通参数化方法声明门中仅准入公有无参实例 `List<String>` null 返回及匹配的唯一 `Signature`/descriptor，保持物理来源和原子投影

## 2. 正反例与停止

- [x] 2.1 对真实 class 测试 `List<String> empty()`，并验证 raw 字段/方法与既有 `id(List<String>)` 不变
- [x] 2.2 测试错误擦除/签名、正文额外效果、同名调用绑定及预算/取消，拒绝时无半个泛型声明

## 3. 三方验收

- [x] 3.1 扩展 DT-18 冻结脚本的修后模式，原/JADX/Jarde 完整类型与同一 Runner 通过 Java 8 重编、`-Xverify:all` 运行，反射和调用结果逐字一致
- [x] 3.2 运行相关 Rust 测试、fmt/check 与 OpenSpec strict，清理临时 Cargo target；将嵌套参数化成员和局部变量推断保持在待测队列
