## 1. 类头与父定义证明

- [x] 1.1 复核 `project_generic_signature` 的早退/参数化父类拒绝、reader 擦除与选定环境解析接缝，定义准确 child/parent identity
- [x] 1.2 对无自有形参的 `Child extends Parent<String>` 只在唯一选定 `Parent<T>` 的真实 Signature 与物理 superclass 闭合时复用现有 class 头 writer

## 2. 拒绝和停止

- [x] 2.1 覆盖非泛型父、错 arity、错擦除、缺失/冲突物理父定义、额外接口及错误 Signature 的 verifier-valid 或 proof-unit 控制，并标明证据等级
- [x] 2.2 覆盖预算/取消的原子停止；raw/no-Signature 和已有 `Parent<T>` 类头结果不变，物理类报告可查

## 3. 三方验收

- [x] 3.1 扩展 DT-21 单层 replay 修后模式，原/JADX/Jarde 完整源码与 Runner 以 Java 8 重编、`-Xverify:all` 运行及泛型父类反射逐字一致
- [x] 3.2 运行相关 Rust 测试、fmt/check、OpenSpec strict，清理临时 Cargo target；多层/bridge 保持独立待办
