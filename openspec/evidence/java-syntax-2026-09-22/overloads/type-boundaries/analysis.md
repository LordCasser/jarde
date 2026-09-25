# 边界结论

## 泛型调用与 Object 重载

| 版本 | `genericObjectCast` |
| --- | ---: |
| 原 class（源码有 `(Object)`） | 1 |
| jarde 生成文本重新编译 | 2 |
| jadx 生成文本重新编译 | 2 |

`javap` 确认原调用的池项为 `choose(Ljava/lang/Object;)I`，而 jarde/jadx 文本都是
`choose(GenericFactory.make())`。这里调用表达式的擦除返回类型也是 Object，但 Java 8 泛型目标
类型推断仍会把未限定的 `make()` 作为 String 候选来做重载选择。故不能用
`presented == required` 作为“无需静态类型约束”的充分条件。

Object 上溯本身不会失败，补 `(Object)` 可以保留原重载选择；这只说明 Object 这个安全目标的
特例，不能作为所有 reference descriptor 的通用 cast 规则。没有 resolver 或 callee body 也不
能从这个 class 单独知道任意外部泛型声明的完整 source type。

## Interface 参数与 verifier

原始 caller 的 `checkcast Runnable` 被脚本按 BCI 1 的三字节指令替换为 `nop nop nop`。实测：

```text
original: java -Xverify:all -> exit 1, ClassCastException
patched:  java -Xverify:all -> exit 0, object=7
```

patched `javap` 是 `aload_0; nop; nop; nop; invokestatic take:(Runnable)I; ireturn`。因此仅有
调用目标 descriptor 不足以证明恢复层应补 `(Runnable)`：补出的 cast 会把 patched class 的合法
执行从 7 改为 CCE。对未知 reference subtype 应保守拒绝该调用或保留字节码 fallback；
`checkcast` 作为原 class 的显式证据时才可恢复检查语义。

当前 debug jarde 对 patched class 的文本是：

```java
// @bytecode 1
// the instruction at BCI 1 is not part of the provable subset
// @bytecode 2
// the instruction at BCI 2 is not part of the provable subset
// @bytecode 3
// the instruction at BCI 3 is not part of the provable subset
return take(arg0);
```

该文本不能以 Java 8 编译，因为 `Object` 不能直接传给 `Runnable`；这应记录为当前 fallback
包围结构的恢复限制。jadx 对同一 patched class 也输出 `return take(obj);`，并标出
`Multi-variable type inference failed`，同样无法编译。没有用手写 cast 或替代方法体进行执行
对照。

## 对参数恢复架构的限定

1. 参数 descriptor 仍必须用于匹配池项调用目标，但“能通过 JVM verifier”与“生成 Java 源码的
   overload resolution 会选同一目标”是两件事。
2. 可失败的具体类/接口 cast 不能只因 descriptor 需要就合成；应优先读取原字节码的
   `checkcast` 证据，缺证据时拒绝或保留 fallback。
3. Object 这类不会失败的上溯可作为独立安全特例，用于固定泛型 poly expression 的重载选择；
   该特例不能扩展到 Runnable 等未知 reference subtype。
4. 这两个探针不要求读取 callee body，也没有证明可用类型 resolver 推断外部泛型边界；后续
   实现应保持该读取预算边界。
