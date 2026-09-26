# 3.1 捕获证明验证

本阶段只在已经唯一绑定的 root/Member 物理定义及一致的双向 `InnerClasses` 关系上建立捕获证书。`member_family.prepared.capture` 分别报告 `proved` 或 `refused`；证书记录 child 字段表索引、字段名、物理构造器、写入 BCI，以及每次 child 字段读取的物理方法、BCI 和 SSA 消费者 BCI。两类结果均保留 root 与 child 原有物理报告及文本。`proved` 不表示成员构造调用、隐藏构件、嵌套 writer 或完整家族源码已获证明。

当前接受的构造器是明确的窄模板：唯一 `<init>(Outer, …)`，且整段恰为 `aload_0; aload_1; putfield capture; aload_0; invokespecial Super.<init>()V; return`，BCI 为 0/1/2/5/6/9，无异常处理器。构造写入的 SSA 两个来源分别是入口 `this` 和首个物理参数。此模板覆盖冻结的 Java 8 正例；它不外推到其他合法 javac 构造器。多构造器、`this()` 委托、不同 super 调用、额外指令、异常范围、native/abstract 方法、字段 MethodHandle 使用和不完整 SSA 均拒绝捕获证书。

| 样本或检查 | 证据等级 | 结果 |
| --- | --- | --- |
| 冻结 `NamedMemberFamilyStage1` JAR（哈希见阶段一证据） | 真实 Java 8 class，原执行有 `-Xverify:all` 记录 | 字段 `this$0` 索引 0、构造写 BCI 2、child `read` 的捕获读 BCI 8/消费者 BCI 11 获证；同类型显式 `other.state` 读取 BCI 1 不在证书内。|
| 重复 Outer 字段、去掉 `final`、重复 `<init>` | proof-unit：改动已解码的成员表事实，不声称变体可由 JVM 验证 | 证书拒绝；冻结物理定义作为原报告保留。|
| 将构造写位置改为 `invokespecial`、加入覆盖 prologue 的异常处理器 | proof-unit：改动已解码的 Code 事实，不声称变体 verifier-valid | 窄构造模板拒绝。|
| 将 `other.state` 的字段引用改为 capture、将 child `read` 的 capture 读改为额外写入 | proof-unit：复用真实 SSA，向生产使用的字段访问闭包扫描器送入受控 Code 变体；不声称变体 verifier-valid | 前者因 receiver 来源为显式 `other` 拒绝，后者因非唯一写入拒绝。|
| 将 `method_bodies` 限制在两份物理报告完成后的用量 | 请求集成测试 | 捕获状态为 `refused`、根 execution 为 `Partial`，child 物理 execution 仍 `Complete`，其方法表和原始文本保留。|

字段扫描对每个匹配的物理 Fieldref 检查操作码、唯一写入点、读取 receiver 的 SSA 来源和读取结果的全部直接 SSA 消费者。证明无法跨 phi 或未知消费者时拒绝。未被证明的读取不会按字段名或 Outer 类型重写。完整值表达式、异常等价和源位置投影仍由后续任务逐点证明。

Root 独立验收：在隔离 Cargo target 中运行 `cargo test --locked -p jarde --all-features --test member_family_identity`（4/4）、`--lib member_inner::tests::family`（3/3）和 `--lib class_source`（22/22），均通过。复核 `ClassSourceMemberFamily::Prepared.capture` 只增加物理捕获证书，不改变 root/child Java 文本；低预算时 `capture=refused`、root `Partial`、child `Complete`，两份方法和来源仍可检查。代码只接受唯一 synthetic/final Outer 字段及冻结的六指令构造器，未证明的结构拒绝并留给后续任务，当前不声称源码家族可重编。隔离 target 在验收后清理。
