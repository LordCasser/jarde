# root 最终验收：`instanceof` 的静态类型与消费边界

实现仍在既有 decode、AST、表达式构建和 boolean 判据内：类型测试读取真实 CP 目标，在最终消费者位置渲染左操作数；具体引用在需要时用现有 Cast 安全上溯 `java.lang.Object`，函数式值先固定已有工厂目标，显式 `checkcast` 保留在内层。没有引入继承解析器、通用 pass 或新的跨方法推断。JADX 1.5.6 的 `InsnGen` 直接拼 `operand instanceof Target`，对这份输入的静态不兼容类型和无目标方法引用会生成不可编译 Java；这里不能照搬其呈现算法。

## 独立完整类对照

root 从永久 `InstanceOfProbe.class`（SHA-256 `d8f4a437d0cd0f3ec12672791f27b1c11cfbcdc6885d10eddcd3114aa0838e56`）用当前私有 CLI 生成完整源码，SHA-256 `8471047151de2cc8722418d2632e0c304d75c51012de6f38fcaba7f40e8c7a8a`，零 `@bytecode`/`not recovered`。原 class 与完整 Jarde 类分别用 Java 8 编译/运行 `-Xverify:all`，22 行逐字相同，输出 SHA-256 `37a6cb0205157884d2e39a7b98f19271dec72a8e025f21b27626d46a843494f8`；包括 null、数组、静态不兼容目标、函数式引用、boolean 局部/参数/分支、调用一次及生产者抛错。

root 对 `InstanceOfTypes` 静态类型边界类另作 Java 8 原/Jarde 完整类重编与严格校验，11 行同为 SHA-256 `94a63eb6a5605a3b5fadf736c0e1ca5ea69c70b35cb69ef8cc1be7a608e9e15f`；当前生成源码 SHA-256 `622f15526a6801e1f90afc1891d86075bc4d1d5344f1ce023e7a79724dabec8e`。一个独立的 `((String) value) instanceof CharSequence` 最小 Java 8 探针，原/Jarde 完整类在 String、null、Integer 上同为 `true`、`false`、`java.lang.ClassCastException`；Jarde 源保留内层 `(java.lang.String)`，不能以外层 Object 加宽吞掉真实运行时检查。

root 对原冻结类重放 JADX 1.5.6，JADX 源 SHA-256 `ca3669849b181be5271344945602736d53e37ef2c145adc0b0bcc7da7e0b6457`。仅机械移除默认包的 `package defpackage;` 以便与原默认包 helper 一起编译，没有修正生成方法；`javac --release 8` 仍退出 1，报两处 `String` 不能转换为 `Integer` 及一处无目标类型的方法引用。JADX 阶段因此没有可执行正文可与原 JVM 比行为。

## 拒绝、来源与相邻回归

强化后的专项测试要求 pop、重复消费、保留旧局部值及 boolean→int 消费者明确引用 producer/type-test/consumer 的真实 BCI；局部值被覆盖的合法形状则必须写出类型测试、`false` 覆盖和当前局部返回。default/all/replay 同正文，类型测试 BCI 1 的直接来源归属其物理成员；正文输出预算或预取消不发布半份 Java；来源预算不足时正文保持不变，仅交付已计费的来源前缀。`p3_instanceof` 普通 6/6，显式 JDK 整类 1/1。

相邻 `p3_reference_cast` 曾仍要求 `nestedCall` 的下游 `instanceof` 引用，现已与本案直接冲突。root 独立确认真实 `source(); checkcast Runnable; checkcast Number; instanceof Comparable` 完整类可编译：原/Jarde 的 20 行 runner 输出逐字相同，SHA-256 `b1bceb660f5514cd4e8f4dce3c9405b1fc36ccef9e2fc81efc12492de2a1eed4`；`nestedCall` 都抛 `ClassCastException` 且 `source()` 只调用一次。测试现断言四个真实 BCI 及完整类，普通 5/5、JDK 1/1。`p3_boolean_contexts` 13/13、`p3_immediate_functional_receivers` 2/2、`p3_class_literals` 4/4、`p3_primitive_conversions` 3/3；`p5_corpus_fingerprint` 5/5。`cargo fmt --all -- --check`、`git diff --check`、本 OpenSpec strict 均通过。两项直接测试目标在对既存五类 Clippy lint 作临时豁免后通过，未引入新告警。

更广的相邻巡查发现独立旧断言：`p3_hoisted_boolean` 两项分别仍要求已恢复的整数→boolean 返回被引用、并固定旧 IR 计数 400（现 448）；`p3_content` 两项仍把已恢复显式 reference cast 的返回当 ExplanationOnly。reader fixture census 另因共享语料增长报真实 `(208,1318,126,611,8)` 对旧钉值 `(164,1152,98,381,8)`；本案补测没有新增 class。它们与本案代码改动无关，已按各自语义/预算/语料范围记入[roadmap 后续范围](../../roadmap.md)，没有混入 `instanceof` 的恢复实现或测试修正；不能把这些红灯隐去称为全仓绿色。

结束后 `cargo clean` 清理了审计/测试私有 target，移除约 2.6 GiB；代理的约 140 KiB 临时类型边界复放目录也已核对并删除。项目约 220 MiB，本机约 98 GiB 可用。
