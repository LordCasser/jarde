## Why

固定 JADX 的泛型 `Function`/`Supplier` 方法引用在 Jarde 中仍失去声明的目标类型，且绑定实例引用因创建期空值语义不能安全退化为 lambda 而被拒绝。当前同布局完整类无法重编；[固定基线](../../evidence/java-syntax-2026-09-28/dt27-typed-functional/README.md)已把问题定位为方法 `Signature` 的正文来源证明与 LambdaMetafactory 适配判定之间的断点。

## What Changes

- 对无方法类型变量、直接返回已验证 LambdaMetafactory 站点的普通方法，证明参数化 `Function<String,Integer>` 或 `Supplier<String>` 目标与 bootstrap instantiated SAM 的相容性，再投影完整声明。
- 在该已证明目标下，区分 erased SAM 到 instantiated 参数的泛型适配与 implementation 实际需要的转换；只有 Java 8 方法引用源码能表达相同成员、调用阶段、装箱/返回结果时输出 `::`。
- 对绑定实例引用保留创建时求值、空值检查和所捕获对象；证据不足继续明确拒绝，不把空值失败推迟到调用时。
- 固定静态、绑定实例、零参 Supplier 的原 class/JADX/Jarde 完整类三方重编与验证运行，并加入不匹配 Signature、非直接返回和可观察捕获近邻。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：完整且相容的参数化函数目标与方法引用应保持 Java 8 可编译的泛型声明、成员引用和捕获语义；不相容证据应安全拒绝。

## Impact

影响 `jarde-java` 的同轮泛型返回候选、class-source 方法声明投影、现有 LambdaMetafactory Plan 和 AST/Builder 源码构建；优先复用既有 Signature parser、SSA、Lambda Plan、表达式节点、预算与停止通道。不新增 crate、通用泛型类型求解器、全局 nullness 分析或反编译 pass。前置是已完成的 descriptor 适配与 class-source Signature 机制；本 change 不覆盖 unbound `Object::toString`、任意重载/继承、altMetafactory 或 DT-27 全部形态。
