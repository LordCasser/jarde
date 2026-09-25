# root 独立验收：数组调用上溯与重载目标

## 架构裁决

实现仅在既有 `invocation_argument` 消费点增加私有、有界的数组关系判据：已呈现的源类型与真实 Methodref 形参共同证明同型组件、引用组件到 `Object`、数组整体到 `Object`/`Cloneable`/`Serializable` 及其递归数组形式。最多剥离 255 维；`int[]↛Object[]/Cloneable[]`、未知用户类/接口层级维持拒绝。非同型已证上溯统一复用 `cast_argument`，源值和调用 BCI 保留，且源级 cast 固定目标重载。没有引入 resolver、公开 Type 层级或新 pass。

本地 JADX `2fb1b163` 的 `TypeCompare.compareTypes` 递归组件、`TypeUpdate.invokeListener` 将方法形参送进 SSA 类型更新，`InsnGen.generateMethodArguments` 再直接打印参数。Jarde 借鉴组件比较与形参事实，但在调用消费处显式固定 Methodref 的静态目标，而不依赖全局推断恰好留下正确局部类型。JLS 8 §4.10.3、§5.1.5 保证这些上溯不改变数组对象且无需可能失败的窄化；`javac` 对显式源级 cast 可以发出冗余 `checkcast`，原调用的重载目标仍必须一致。

## 完整类、三方与拒绝对照

root 用当前源码在私有 target 重建 CLI，以永久 class 独立生成完整源码，均不改写生成方法。`javac 23.0.1 --release 8 -g:none` 编译原/Jarde，`java -Xverify:all` 运行；JADX 1.5.6 生成源码仅机械移除默认包被改成的 `package defpackage;` 行后编译同一 runner。

| 输入 | 原/JADX/Jarde 完整类结果 | 关键证据 |
| --- | --- | --- |
| `ArrayReferenceCore`（1,632 B、16 Code，SHA-256 `549e371a…ae0d`） | 三方 Java 8 编译、JVM 验证通过；28 行逐字相同，输出 SHA-256 `80169ee0…00f27`；Jarde 零 `@bytecode` | `int[][]→Object[]`、`String[]→Object[]`、`String[][]→Object[][]` 分别有目标 cast，源码 SHA-256 `03dc4c7a…bb20e8fb`；原源码重编 class 与冻结 class 逐字相同 |
| `ArrayReferenceOverloadTarget`（SHA-256 `116d468c…964f5dc`） | 三方 Java 8 编译、JVM 验证通过；8 行逐字相同，输出 SHA-256 `19c78802…b13a4bcaf6`；Jarde 零引用 | 原 `localObjectTarget` 为 `astore_1; aload_1; invokestatic overload(Object[])`，无 `checkcast`；恢复后仍是该 Methodref，空/null 对照选 `Object[]`，`effectful-evaluations=1` |
| `ArrayInterfaceProbe`（1,038 B、SHA-256 `83f4f36a…a3580`） | 三方 Java 8 编译、JVM 验证通过；3 行逐字相同，输出 SHA-256 `7da47d62…dfc9e9`；Jarde 零引用 | `int[][]→Cloneable[]`、`String[][]→Serializable[]`、`int[]→Cloneable` 的真实调用均无 `checkcast`；冻结 class 与原源码重编一致 |
| `UnknownArrayRelation`（SHA-256 `a9356b65…47bb98`） | 原合法 class 的 runner 通过 JVM 验证、执行 2 行；Jarde 仍对两处缺少层级证明的调用保留拒绝，source map 同时覆盖参数加载 BCI 0 与调用 BCI 1 | `Child[]→Base[]` 和 `MarkedChild[]→UserMarker[]` 不从目标描述符臆造 cast；未把拒绝类计入可编译正例 |

旧冻结 CLI 对 core 三处和 marker 三处均引用，完整源码因缺返回而不能重编。永久 `v8/` 仅有四个 subject class；runner 和未知关系的嵌套类为 source-only，避免无关 class 扩大语料。已有 37 行宽样本另含 `new Object()`/数组写入缺口，不在本案覆盖内。

## 门禁与独立债务

`jarde-java` lib 142/142，数组上溯专项 5/5；调用参数 9/9、即时函数式接收者 2/2、数组类型 3/3（1 ignored）、显式 cast 5/5（1 ignored）、数组初始化器 2/2（1 ignored）。专项验证了成功 cast 的生产者/调用 BCI 同段来源、未知关系拒绝的真实参数加载 BCI 0 和调用 BCI 1、default/all 正文一致，以及低输出预算和预取消不交付半份正文。root 在首次验收中发现原 quote 遍历漏掉稳定局部加载的 BCI 0，因而没有勾选来源任务；实现随后沿同一有界生产者遍历收集该 BCI，仅在引用确含调用时追加，并复测专项 5/5、相邻 `p3_eval_context` 10/10、`p3_popped_static_qualifier` 4/4、`p3_deferred_value_order` 2/2（1 ignored）及 `p3_invocation_arguments` 3/3（1 ignored）。root 用最终 CLI 再次重编三组完整正例，28、8、3 行输出均与原类一致。corpus fingerprint 重生成后为 572 个文件，5/5；`cargo fmt --all -- --check`、相应范围 `git diff --check`、OpenSpec strict 均通过。

严格 Clippy 在既存 13 处 `useless_conversion`、`too_many_arguments`、`type_complexity`、`needless_option_as_deref`、`collapsible_if` 被阻断；仅临时豁免这五类后，`jarde-java` lib 与新增专项测试以 `-D warnings` 通过，专项新发现的 `needless_borrow` 已在本案测试内修正。reader fixture census 当前真实 `(212,1354,126,611,8)`，旧断言仍为 `(164,1152,98,381,8)`；比上一轮真实 `(208,1318,126,611,8)` 增加四个 class、36 个 Code，属于待单独重钉的语料计数债务。相邻 `p3_lambda_adaptation` 有一个既存 RED：`strings()` 仍输出未适配的 `LambdaAdaptationSupport::pick`，对应尚未实施的 `preserve-lambda-descriptor-adaptation`，与本次只处理普通调用实参无关；本案不修改其断言或实现。

最终重编、专项回归及严格 OpenSpec 校验后清理了本轮 Cargo 编译产物：仓库 `cargo clean` 移除约 3.1 GiB，私有 target 移除约 766 MiB；仓库当前约 221 MiB，磁盘可用约 97 GiB。永久 fixture 和审计文档保留。
