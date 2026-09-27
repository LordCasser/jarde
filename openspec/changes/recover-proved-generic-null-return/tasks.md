## 1. 同轮 null 返回证明

- [x] 1.1 在既有 `GenericReturnCandidate` 接缝中证明单一、无参数、无效果 `aconst_null; areturn` 与同轮 AST/SSA/BCI 一致；任何额外指令、handler、phi、写入或操作不完整拒绝
- [x] 1.2 对真实编译正例及 verifier-valid 负例做候选/拒绝测试，避免只比较最终文本

## 2. 类源码泛型头

- [x] 2.1 复用 reader Signature 擦除及既有 class-source 原子投影，仅准入顶级普通 `Object` 子类的公有无参实例 `<T extends Number> T` null 返回；保存物理方法来源
- [x] 2.2 测试错误界/descriptor、类级变量、类型注解、同名重载/Methodref、正文副作用、预算及取消整项拒绝；独立方法恢复和物理报告保持可查

## 3. 三方验收

- [x] 3.1 将 `dt16-generic-null-return/replay.py` 的 `fixed` 模式跑通，要求原/JADX/Jarde 完整类型源码与外部 API consumer 均 Java 8 重编、`-Xverify:all` 运行和反射逐字一致
- [x] 3.2 运行相关 jarde-java/class-source/reader 回归、Rust fmt/check 与 OpenSpec strict；清理临时 Cargo target 并记录局部限制
