# EM-01：静态成员声明与泛型父接口首片审计

本文记录实施前的固定基线；`Shape.I`/`Shape.A` 与 `Generic.A<T>` 均已在后续的 [Generic.A 独立验收](../../java-syntax-2026-09-28/em01-generic-member/root-acceptance.md)中通过完整三方重编与运行。下文所说的 Jarde 缺失指当时的 CLI，不代表当前主线。

固定 JADX `2fb1b16386941660fda07e9017285aec40fcb37f` 中，`TestClassGen` 活动断言要求根类内的 `public interface I` 和 `public static abstract class A`，`TestClassImplementsSignature` 的 Java 测试要求 `public static abstract class A<T> implements Comparable<A<T>>`。后者还有独立 Raung 畸形签名测试；`TestIncorrectFieldSignature` 是 Smali 负例，均不能冒充合法 Java 8 重编正例。三份测试哈希固定在 [replay.py](replay.py)。

[input/](input/) 用 Java 8 完整 `Shape`、`Generic` 与共同 Runner 隔离上述两个正例。原 class 和固定 JADX 的两份完整根源码通过 `javac --release 8` 与 `java -Xverify:all`，输出相同：

```text
2:1
1:java.lang.Comparable<em01.Generic$A<T>>
```

固定 Jarde CLI SHA-256 `8f1f0012324350e4fc65c7fef4b3e3835102e6d4fda00df280b466270c98e727` 的根 `Shape`/`Generic` 源码均只含根构造器，不含任何成员声明。把完整根源码与同一 Runner 重编时，`Shape.I`、`Shape.A`、`Generic.A` 四个引用均找不到符号。Jarde 对三个物理 child 的单独请求成功，保留了 `Shape$I`、`Shape$A`、`Generic$A` 的方法事实；`Generic$A` 的 `Signature` 明确被标为不能在单独物理类头投影，因而退为 raw `Comparable`。三方源码、错误日志和摘要在 [baseline/](baseline/)；失败位置是**根类源码家族装配**，不能说 Jarde 不能解析物理 child。

为排除“只是两个 child 才失败”，另以 [input-single/](input-single/) 的唯一无分配点 `SingleAbstract.A` 重放。原/JADX 的完整根源码均重编、验证运行输出 `true:1`；Jarde 根源码仍没有 `A`，完整源码重编失败，物理 child 则可单独请求。[single-baseline/](single-baseline/) 保存同一固定工具链的三方源码与日志。因此单 child 声明型恢复本身就是独立缺口。

已有 `ClassSourceMemberFamily` 及双向 `InnerClasses`、唯一物理定义读取与成员原子投影，无须另建 JVM IR 或按 `$` 名猜嵌套。当前 `prepare_class_source_member_family` 对静态 child 要求 `ProvedStaticMemberTarget`，即根方法里存在已证分配/构造目标；`project_class_source_static_member_family` 又要求非空构造调用站点及完整普通方法体。这里根类只有声明、没有分配点，而接口与抽象成员本来无 Code，因此**声明关系证明被构造使用证明绑死**。应先在既有家族边界把“可证明的单个静态成员声明”和“某使用点的构造/返回改写”分离；无使用点时只需前者，仍要保留完整 root/child 报告、预算和来源。

后续差距必须拆开验收：

1. 单个无分配点的普通静态成员接口/抽象类声明，先证明双向关系、flags、完整成员表和无须改写的 root 使用闭包，允许无 Code 的合法抽象方法；这是当前最小实现任务。
2. `Shape` 的**两个**静态 child 要在同一根文本按关系装配，且任一 child 停止时不发布半份嵌套源码；属于家族基数扩展，不能用第 1 项的单 child 结果冒充。
3. `Generic.A<T> implements Comparable<A<T>>` 的 child class Signature、父接口 type argument、bridge 与根嵌套源码同轮证明；属于泛型投影扩展，不能只把 raw `Comparable` 改成文本参数。

上述样例不代表 EM-01 的全部类头、畸形 Signature 或 DT-01/02/19/21 的全部成员与泛型形态已追平。JADX `RootNode.initInnerClasses` 的先建关系、`SignatureProcessor` 的类型校验和 `ClassGen` 的嵌套写入顺序可参考；Jarde 应继续按选定定义、属性与完整方法事实证明，不复制 JADX 的警告文本或宽松错误恢复。
