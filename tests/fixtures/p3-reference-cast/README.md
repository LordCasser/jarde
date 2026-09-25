# p3-reference-cast

这是 `recover-explicit-reference-casts` 的独立 Java 8 fixture。源码只使用自写的
`ReferenceCastProbe` 类，覆盖对象、原始数组、引用数组和多维引用数组转换，以及返回、
接收者、局部、静态调用参数、带计数调用和两层引用检查。

`nested` 使用 `(Number) (Runnable) value`：`Number` 不是 final，因此该转换在 Java 中合法；
传入 `Integer` 时必须先在 `Runnable` 检查处失败。`nestedCall` 把同一两层转换交给
`instanceof Comparable` 消费，恢复正文保留 `source()`、两个 `checkcast` 和类型测试。
JDK runner 单独调用 `nestedCall`，确认原始和恢复类都抛出 `ClassCastException`，并且
`source()` 只调用一次。`source()` 返回 `Object` 并递增计数。Rust 回归会在内存中把
`discardedCastBeforeCall` 的同宽 `astore_1` 改成 `pop`；这个否定路径不产生或提交手写
class 文件。

编译环境与命令：

```text
javac 23.0.1
javac --release 8 -g:none -d v8 ReferenceCastProbe.java
```

提交的 class 是上述命令生成的 `v8/ReferenceCastProbe.class`。测试只运行这份自写 class；
JDK 对照中的 `ReferenceCastProbe.java` 是 Engine 生成的恢复正文，runner 写在临时目录，
不会作为 fixture 提交。
