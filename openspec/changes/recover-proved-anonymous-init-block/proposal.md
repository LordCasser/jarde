## Why

固定 JADX 的 `TestAnonymousClass4` 要求匿名类实例初始化块回到匿名体。DT-09 的隔离 Java 8 样例显示 Jarde 已能恢复物理 child 构造器效果，完整源码也能重编运行，但因为现有匿名投影只接受无效果构造器，根类仍输出 `new Subject$1()`，丢失源级初始化块。

## What Changes

- 对同包、无捕获、唯一直接返回分配点的匿名父类子类，增加有界的“准确 `super()` 后单个已证明静态 `int` 字段赋值”构造形态，并在匿名体中输出实例初始化块。
- 沿用现有物理类与调用点证明、同次 AST、整类 owner 引用普查及根类原子提交；任何形态或预算证明失败时保留物理 child 源码。
- 固定原/JADX/Jarde 完整源码 Java 8 重编和验证运行对照，并加入额外构造效果、错误字段、第二分配点、预算停止等拒绝例。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：在已证明的受限匿名类构造形态下恢复实例初始化块，并保证失败时不发布不完整的匿名体。

## Impact

主要涉及 `crates/jarde-java` 中的同次构造 AST/SSA 证书与匿名体呈现，以及 `src/facade.rs` 的匿名父类整类投影。无需新增 crate、外部依赖或公开 API；物理 child 的独立报告继续可查。此变更不覆盖 JDK `Thread` 父类、外层私有实例字段、非返回分配或嵌套匿名类；这些仍按 DT-09 原测试的其它依赖分别验收。
