# EM-11：重载调用的 Java 8 源级绑定审计

固定库存为 [`expressions-misc.md` 的 EM-11](../../jadx-feature-inventory-2026-09-27/expressions-misc.md)。本次只读审阅本地 JADX `2fb1b16386941660fda07e9017285aec40fcb37f` 的四个指定测试与 `MethodInvokeVisitor`、`InsnGen`；[结果文件](observed/results.json)记录六个文件的 SHA-256、全部自写源和 class 的 SHA-256、Jarde CLI 二进制及生成源码的 SHA-256。测试基线 Java 编译器为 `javac 23.0.1 --release 8 -g:none`，JADX CLI 为 `1.5.6`；Jarde CLI 从本分支基线 `a25f9d57b957601ad866b65e5be308a5b4577b3a` 构建。

## 固定测试的证据强度

| 测试 | 活动断言与边界 |
| --- | --- |
| `TestCastInOverloadedInvoke` | 活动 `@Test` 检查首个 `ArrayList` 调用、转为 `List<String>` 的调用和 `instanceof String` 后调用的文本。另一个要求保留 `new ArrayList<String>()` 精确写法的断言标有 `@NotYetImplemented`，不能算已通过；内部 `check()` 的 110/1/111 是辅助行为检查，不能代替源码重编。 |
| `TestHierarchyOverloadedInvoke` | 活动断言覆盖接口/父类/子类同名方法中的 `ArrayList`、`List<String>` 和 `String` 绑定；`test4()` 的接收者 cast 在类内 `check()`，不在活动源码断言中。 |
| `TestOverloadedInvoke` | `JAVA8` 与两个 Dex profile 的活动断言要求数组目标上的 `(Object[][])`、`(Object)` cast，并允许冗余 cast；测试明确留下减少 cast 的 TODO。 |
| `TestCastInOverloadedAccessor` | `SmaliTest` 活动文本断言只检查匿名类访问外层两个 `String` 重载的调用形状；没有 Java 8 完整源码重编或行为断言。本审计不把它提升为合法 Java 8 正例。 |

JADX 的 `MethodInvokeVisitor.processOverloaded` 先从调用接收者收集重载、解析泛型变量，再按呈现的编译器类型、未知类型与泛型类型搜索目标；必要时插入 cast，未能找到最少 cast 时退到全部形参类型。`processUnknown` 只为未知目标处理 `null` 等未知实参。`InsnGen.makeInvoke` 最后打印已决定的调用接收者与实参，不重新证明绑定。Jarde 的对应边界在 `jarde-java::build::invocation_argument`：同型、目标 `Object`、`null` 和已证明数组上溯可过；普通 `ArrayList → List` 当前没有安全关系事实。另一个独立门 `class_source::generic_call_binding_unproved` 会保守拒绝相邻泛型重载的声明 `Signature` 投影。

## 可重放的完整源码对照

[完整 Java 8 输入](input/)分为两个小切片。`em11` 的 `OverloadCalls` 同时区分 `String`、`List<String>`、`ArrayList<String>` 与两个数组重载；`HBase → HMid → HLeaf` 分布同名方法，`HierarchyCalls` 观察继承绑定。`em11simple` 只测三种显式 `null` cast 与 `int[][][]` 的两个数组目标。每个切片都有完整 `Runner`，分别把原源码、JADX 输出、Jarde 输出的**所有类**重编，再运行 `java -Xverify:all`；不把原 class 混入生成源码的运行 classpath。

| 切片 | 原源码 | 固定 JADX | Jarde |
| --- | --- | --- | --- |
| 同类集合 cast + 继承重载 | 编译/验证运行通过，stdout SHA-256 `ea9cdf24c20e4907672af1fb32ba271ada12c15a285227d8bb48594716bd0df0` | 编译/验证运行通过，逐字输出一致 | 完整源码编译失败：`OverloadCalls.run` 的 `local2` 未赋值；单独编译继承四类又报 `HierarchyCalls.run` 缺返回。两处皆由 `ArrayList → List` 调用拒绝引起，无 Jarde 运行结果可比较。 |
| 独立 `null` + 数组 | 编译/验证运行通过 | 编译/验证运行通过 | 编译/验证运行通过；三方逐字输出 `String/List/ArrayList/Object[][]/int[][]`。 |

主切片首行区分八个调用：`ArrayList/List/String/List/ArrayList/String/Object[][]/int[][]`；第二行把 `instanceof` 的 `String` 分支改为 `none`；第三行区分继承的 `leaf-ArrayList/mid-List/base-String/mid-List/leaf-ArrayList/base-String/mid-List`。固定 JADX 的 [完整 `OverloadCalls` 输出](observed/source/jadx/em11/OverloadCalls.java)用 `(List<String>) new ArrayList()` 固定第二调用；其完整源码的运行结果与原源码一致。这不声称上述 `@NotYetImplemented` 对精确构造器写法的要求已通过。

Jarde 的 [同类输出](observed/source/jarde/em11/OverloadCalls.java)在 BCI 18 留下“`ArrayList` 呈现值需要 `List`，无安全引用转换证据”的拒绝，之后仍引用未赋值的 `local2`；[继承输出](observed/source/jarde/em11/HierarchyCalls.java)在 BCI 42 因相同转换拒绝整段 `run`。该拒绝是保守的，不能用物理 Methodref 参数直接猜一个可能失败的 cast；但它使合法 Java 8 的完整类无法重编。`null` 与数组的成功切片则证明现有显式目标 cast 路径独立可用。Jarde 仍将集合方法的 `Signature` 回退到原始类型；本次运行观察值没有检验泛型反射或强类型 API 消费者，不能宣称泛型声明已追平。

从仓库根运行：

```sh
cargo build -p jarde-cli --target-dir /tmp/jarde-em11-target
python3 openspec/evidence/java-syntax-2026-09-27/em11-overload-binding/replay.py \
  --jadx /opt/homebrew/bin/jadx \
  --jadx-checkout /Users/lordcasser/workspace/testzone/jadx \
  --jarde /tmp/jarde-em11-target/debug/jarde-cli \
  --out /tmp/jarde-em11-replay
```

`--out` 必须为空目录；脚本核对固定 JADX commit 与六个源文件哈希。持久的本次输出在 [`observed/`](observed/)，包含三方完整生成源码、编译日志、验证运行日志与哈希。编译中间物和临时 JAR 由脚本的临时目录自动删除。

## 缺口与拆分

为 `ArrayList → List` 的已证明上溯及同类/继承重载目标建了窄 [OpenSpec](../../../changes/preserve-proved-reference-overload-binding/proposal.md)。本页及 [`observed/`](observed/) 保留实施前审计；随后实现的完整源码复验记录在 [verification.md](verification.md) 和 [`observed-after/`](observed-after/) 中。`generic_call_binding_unproved` 的泛型声明投影、合成访问器的 Smali 路径，以及固定 JADX 的 NYI 精确写法均是独立边界。库存总表保持原样。
