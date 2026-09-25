# 方法自有泛型 throws 正文候选的负例边界

[`negative/replay.py`](negative/replay.py) 生成 Java 8 fixture、在临时副本上改写 class-file 的一个 `Signature` 常量，并对原件和变造件做验证与 Jarde 重放。它将 `CARGO_TARGET_DIR` 指向 `TemporaryDirectory` 内的私有目录；class、源文件和 Cargo 编译产物都在脚本退出时自动清理。运行命令：

```sh
python3 openspec/evidence/java-syntax-2026-09-24/body-method-local-generic-throws/negative/replay.py
```

脚本对各门使用硬断言：任一 JVM 验证/Jarde 失败、拒绝码不符、物理声明变化、核心生成类未通过 javac，或 typed caller/override 未失败，都会令进程非零退出。自调用回退类的 javac 失败是预期观测，因此脚本只断言其确实失败且错误涉及 `Exception`。脚本中的 `GenericThrowsCore.run` 是完整、合法的 Java 8 声明：`<X extends Exception> void run() throws X`，有非空正文且不包含本类调用。其原 class、擦除矛盾件和未绑定变量件均由 `java -Xverify:all` 成功加载运行；方法物理 descriptor 仍为 `()V`，物理 `Exceptions` 仍为 `Exception`。擦除矛盾变体仅将 Signature 从 `<X:Ljava/lang/Exception;>()V^TX;` 换成 `<X:Ljava/lang/RuntimeException;>()V^TX;`，于是 `X` 的擦除与物理 Exceptions 矛盾。另一个等长改写变体只将 throws 变量 `^TX;` 换为 `^TY;`，保留 `<X>` 声明但引用未绑定的 `Y`；它用于区分 scope proof 与擦除比较。JVM 验证器不校验此泛型元数据，因此该负例证明“能过 verifier”不足以证明 Signature 可投影；Jarde 对擦除矛盾件以 `invalid input (jvm_signature_erasure_mismatch)` 拒绝；对未绑定变量件以 `jvm_signature_scope_unproved` 拒绝。两者都回退为物理 `void run() throws java.lang.Exception`。

原件的 Jarde 拒绝发生得更早：非空正文不满足当前方法自有泛型 throws 候选的 `body_method_local_generic_throws_source_unproved` 门（同轮 AST/Code/SSA 未证明精确空正文）。它仍输出物理签名；生成的完整 `GenericThrowsCore.java` 可由 Java 8 `javac` 编译。把一个按泛型契约调用它的独立调用方编译到该输出上，会因物理回退暴露的 `throws Exception` 而失败。这是源码投影信息丢失的实测后果，但不是局部泛型正文候选已经通过证明的证据；本 fixture 有意保留非空正文，用来压测正文结构门，而非模拟提案中空正文首片。

另有 `GenericThrows` 同类调用用例。Jarde 明确以 `generic_call_binding_unproved` 拒绝：同类 `Methodref` 指向当前方法或相邻重载。回退源码包含 `this.run()`，其完整类 `javac` 因 `Exception` 未捕获而失败。此结果定位到调用绑定门；不能把它解释成方法变量作用域、throws 擦除证明或正文候选门的失败。`GenericThrowsCoreChild` 则给出原泛型声明下可编译的继承覆写；对 Jarde 回退后的父类重编时，子类同名泛型方法不再覆盖父类非泛型方法而失败。这说明父类声明变形会改变继承绑定，但不能单独指出投影审查应在哪个阶段拒绝。

已确认的先后门与证据范围：

- 原 fixture 的全部 Java 文件先以 `javac --release 8` 完整编译；原、变造运行时类均以 `-Xverify:all` 成功运行。
- 无调用的非空方法被 `body_method_local_generic_throws_source_unproved` 拒绝；因此它不能验证此候选的空正文路径。
- 同类调用先被 `generic_call_binding_unproved` 拒绝；调用方/子类针对物理回退源的编译失败是可观察后果。
- 擦除矛盾 Signature 与 `Exceptions` 属性不一致，被 reader 的 `jvm_signature_erasure_mismatch` 拒绝；未绑定 throws 变量被 `jvm_signature_scope_unproved` 拒绝。二者都是 verifier-valid、但元数据不能安全投影的 classfile 负例。
- 三种 Jarde 物理回退完整类均可单独重编。强类型调用和泛型覆写的失败分别记录为回退后的绑定变化。

局限：两种变造件都不是 `javac` 能生成的合法 Java 声明，而是直接改写 classfile 泛型元数据；它们证明 JVM verifier 不替工具检查 Signature 擦除一致性或变量作用域。`GenericThrowsCore` 非空正文刻意触发早期正文形状门，因而不能验证合法 `<X>` 声明走完空正文提案路径及发布时的完整声明检查；单独的 `^TY;` 变体只验证未绑定变量被 scope proof 拒绝。继承子类只验证一个简单的同签名覆写关系，不覆盖多层继承、接口默认方法或桥接方法组合。脚本在本地 OpenJDK 23.0.1 上以 Java 8 目标重放；它没有声称覆盖其他 JVM 或所有 javac 版本。
