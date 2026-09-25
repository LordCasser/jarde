## Why

Java 8 方法的普通参数化 `Signature` 已在 classfile 中完整存在，但完整类源码目前只写 descriptor 的擦除类型。冻结样本中原类与 JADX 的 `Iterable<String>`、通配符、嵌套泛型和参数化数组能保留反射签名；Jarde 输出为 raw 类型，使依赖泛型返回的调用方源码无法重编。

## What Changes

- 从同一物理方法的 `Signature` 与真实 descriptor 出发，复用 reader 的完整语法和逐位置擦除证明，为普通参数化参数/返回构造 Java 8 声明候选。
- 在现有类源码方法装配中核对类型名称、参数槽、注解/varargs/异常位置及方法体和受影响调用的源级绑定；仅在整项可证明时原子替换声明，保留物理成员与独立方法恢复结果。无正文的合法成员也按其声明事实参与投影。
- JVM 可验证但擦除冲突、类上下文未呈现或源码绑定无法证明时保留 descriptor 声明与明确拒绝原因；以反射、完整类及调用方 Java 8 重编、`-Xverify:all` 对照验收。
- 本项不把增强 `for` 的 `Object` 元素头改成 `String`，也不混入类/字段泛型、任意类型推断或另一套 Signature parser。泛型方法自有 `<T>` 由独立的 `recover-generic-method-signatures` change 处理，完成其声明接缝后本项再接入。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：完整类源码对已证明的普通参数化方法 `Signature` 保留参数与返回的泛型结构和反射信息；未证明时不得输出改变绑定或无法重编的声明。

## Impact

复用 `jarde-reader::signature`；修改 `src/class_source.rs` 的方法头候选与类级发布、必要的同轮方法体侧证据和定向测试。无新依赖、无公开报告格式扩张。基线见 [三方证据](../../evidence/java-syntax-2026-09-24/ordinary-parameterized-signatures/analysis.md) 与 [架构核查](../../evidence/java-syntax-2026-09-24/ordinary-parameterized-signatures/architecture-root.md)。
