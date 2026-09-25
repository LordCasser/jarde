## Why

普通 `ToIntFunction<String>` 方法引用在当前完整类输出中失去目标泛型，重编译后选中 Object 重载，并漏掉应有的 String 检查。独立 6 项执行中有 3 项错值，生成正文却没有任何引用；这是已有 lambda 呈现的正确性问题。

## What Changes

- 让已识别的 LambdaMetafactory 形状保留擦除 SAM、动态 instantiated 类型和 implementation descriptor 的各自职责；不能把“同为引用类型”当作 Java 文本等价证明。
- 需要动态参数检查或精确实现调用类型时，复用现有 lambda、cast、call/new 表达式显式表达；只有可证明等价时才保留方法引用。
- 保持捕获与调用阶段、检查顺序、次数及异常；超出受限转换证明者明确拒绝并保留来源。
- 用普通 javac 样例与合法内存变体固定正反例，完整比较原 class、JADX 和 jarde；候选手写 lambda 不作为验收结果。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：已验证函数式工厂的源码呈现保留 descriptor 所述动态参数限制和实现目标。

## Impact

主要影响 `jarde-java::lambda` 的现有 Plan、Builder 的 lambda 表达式构造与相邻回归。不新增 AST 节点、pass、类型层次 resolver、泛型 Signature 系统或依赖库；不调整 parsing、runtime selection、verification 状态。

前置为已经验收的显式引用 cast、调用实参类型恢复。生产排在 `preserve-deferred-value-order` 后以避免共同 Builder 冲突；本项不扩展捕获 producer 的可重放范围、不内联合成 helper、不修复其整类命名冲突，不实现 boxing/unboxing、任意 primitive widening 或 altMetafactory bridges/markers/serializable。
