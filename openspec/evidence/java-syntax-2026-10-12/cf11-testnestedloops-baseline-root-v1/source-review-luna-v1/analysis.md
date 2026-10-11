# TestNestedLoops 静态源码审阅

本记录只描述被冻结源码中的实现结构和测试意图。没有运行 Jarde/JADX、Java、Git、Atlas 或任何工具链；因此不对本 fixture 的实际反编译文本、编译是否成功或运行时输出作判断。

## 固定输入的控制流与副作用

`TestNestedLoops.java:14-29` 的 `TestCls.test(List<String> l1, List<String> l2)` 有两个嵌套增强 for：外层遍历 `l1`，内层遍历 `l2`（15-16）。内层先调用 `s1.equals(s2)`，相等时再检查 `s1.length() == 5`（17-18）；满足时执行 `l2.add(s1)`，否则执行 `l1.remove(s2)`（19-22）。两层循环之后另有 `if (l2.size() > 0) l1.clear()`（26-28）。测试断言关注两个增强 for、相等比较和两种 list mutation（32-40）。这些行只能说明 fixture 的源码和测试意图，不能证明待采集 classfile 或生成源码的真实形状。

从控制流语义看，`add` 与 `remove` 都是循环体内的有副作用调用；任何循环结构化都必须保持它们原先的执行次数、条件和相对顺序。循环后的 `size`/`clear` 是循环结束后的独立语句，不能因识别循环而提升、下沉或合并进循环体。Jarde 当前候选构造把已有循环 body 原样复制进新的 `ForEach`，并只移除识别到的 iterator 声明/初始化（`build.rs:21644-21662,21664-21715`）；这是源码层面的实现约束，不是对该 fixture 实际转换结果的观察。

## Jarde：增强 for 识别边界

`crates/jarde-java/src/build.rs:21222-21248` 的 `iterable_for_each_candidate` 只考虑 Java release 8 及以上、条件为精确 `Iterator.hasNext()Z` interface invoke、且循环体首条语句为带初始化值的声明。函数随后要求首条声明值是同一 iterator 上无参 `next()`，可选地包一层 cast（21274-21323）。它从同一方法语句中回找该 iterator 的 `iterator()` 初始化，并要求 iterable 是 local（21324-21359）。初始化之后到循环测试之前，不允许其他有副作用的已初始化语句、一般赋值或 iterator/iterable 的重赋值（21360-21376）。

候选再校验物理 invoke owner/name/descriptor 与呈现的 iterable 类型相符：只接受 `Iterable`、`List`、`Collection` 的相应平台 owner；`hasNext()Z` 和 `next()Object` 必须是 `Iterator` interface invoke（21391-21433）。SSA 检查包含 iterator 初始化结果的唯一 store 消费、header iterator phi 仅由初始化值和自身构成、测试/next 都读取该 iterator、iterator 值没有未授权消费者，以及 next 结果只有指定 store 或识别出的 cast 一个消费点（21435-21575；后续 21576-21642 还检查 cast 与异常处理覆盖）。因此这里证明的是一个窄的 iterator 控制流/数据流模式，不是基于 `List<String>` 泛型参数直接推导 foreach 元素类型。

转换时生成变量类型固定为 `java.lang.Object`，保留已有 body，并把 `next()` 表达式替换为该局部变量（21664-21715）。对 String 元素的 cast 是否由输入 AST 作为 inline cast 表达、并在剩余 body 中保持正确，是候选路径所处理的情况；实际 fixture 是否满足全部结构条件仍须看 root 的真实 baseline。候选对 presented iterable type 的检查是精确字符串匹配（21350-21359、21410-21417），并不在此处读取 method Signature 的泛型实参。

已有 `tests/p3_iterable_foreach.rs:122-169` 覆盖 raw/generic、带/不带 debug names、保留 cast，以及额外 `next()`、接触顺序、cursor/protected 等拒绝边界；`241-293` 另以原类与恢复类执行结果对比作为既有测试。不过这些 fixture 不是 `TestNestedLoops`，也不能替代本次目标的实际采集。

## 泛型 Signature：独立证明层与耦合边界

classfile reader 仅把 `Signature` 常量池 UTF-8 原始字节保存在 `AttributeFacts`，明确把 grammar 留给 metadata consumer（`classfile.rs:2490-2502,3015-3025`）。`signature.rs:356-367` 明确限定 `prove_method_signature_erasure` 的职责：证明已解析 method Signature 的变量作用域、参数/返回/throws 擦除与物理 descriptor/Exceptions 一致；它不证明完整泛型形状能否拼写成源码，也不证明调用点重载绑定保留。验证具体参数的擦除匹配在 `signature.rs:438-489`。

源级投影是另一个调用层：`src/class_source.rs:5432-5474` 从 member 的 Signature 属性进入 `project_method_signature_with_state`；`5523-5577` 解析并调用擦除证明；之后根据方法形状走不同声明证明分支，普通无方法类型参数的签名会进入 `ordinary_parameterized_declaration`（5834-5859），其余泛型方法按不同 body/SSA 证据分支处理（5731-5898），再通过 `project_generic` 发布（5902-5912）。因此“reader 有 erasure proof”不能等同“这个目标方法签名已被成功 source-project”；但它与 foreach 的 iterator-pattern/SSA 证明在代码职责上是分开的。

对本 fixture 的实际耦合点仍需由观测确认：method Signature 可能决定形参声明的 source type，而 foreach recognizer 要求 iterable 表达式呈现为精确的 `java.util.List`/`List` 一类 reference 名称；候选本身输出 `Object` 元素声明。静态代码能确认这些是不同检查路径，但不能确认本次被冻结 classfile 中呈现类型、投影状态或最终编译行为。若 baseline 暴露泛型声明问题，应把它记录为 Signature/source projection 观察；若暴露 loop refusal，应按候选门槛定位。当前证据不足以把二者归并成一个新机制或判断谁导致何种实际失败。

## JADX 对照：先建 LoopRegion，再识别和赋型 foreach

JADX `LoopRegionMaker.process` 根据 loop start/exits 建立 `LoopRegion`，从 region 构造条件并处理方向（`LoopRegionMaker.java:61-98`）；`makeLoopRegion` 过滤 exit/condition block、检查 precondition 与 exits 后选定 region（187-245）。这是循环控制结构恢复层。

增强 for 的识别实际位于 `LoopRegionVisitor.checkIterableForEach`，而非 `LoopRegionMaker`：访问器取条件唯一 iterator SSA local，拒绝 iterator local 被 phi 使用的情况，并要求 iterator 初始化、`hasNext()`、`next()` 的使用形状符合模式（`LoopRegionVisitor.java:260-280`）。它针对 next 结果是 inline cast、wrapped expression 或普通 result 分别找元素变量并做局部消费检查（281-322），移除 iterator 相关生成指令后构造 `ForEachLoop(iterVar, iterableArg)`（324-337）。`fixIterableType` 会查看 iterable 的泛型类型实参，并尝试将其与元素变量类型协调；缺少泛型时可从 iterable 的 object 类型和变量类型建立泛型 iterable type，但若 immutable type 冲突，仅允许 Object 元素变量（342-378）。`RegionGen` 最终由 `ForEachLoop.getVarArg()` 和 iterable arg 生成 `for (var : iterable)`（`RegionGen.java:187-209`）。

JADX 的局部变量 debug Signature 另有路径：`DebugInfoAttachVisitor.getVarType` 解析局部变量 Signature、展开 type variables，经过 `checkSignature` 后才用扩展类型（`DebugInfoAttachVisitor.java:153-182`）。这是 local-variable debug metadata；不能仅凭它推断本 fixture method parameter 的真实类型来源，也不能推断输出。整体对照表明，JADX 同样把 loop-region/iterator pattern 与泛型类型协调分层处理，但其接受条件与 Jarde 不同，不能据此预测 Jarde 将接受或拒绝目标 fixture。

## 证据状态

以上“源码事实”均由本文件下方 pins 所列文件的冻结 SHA 与行号支持。实际 javac classfile、javap、Jarde source、JADX source、重新编译及运行结果都尚未在本任务中采集；这些结果必须以 root 随后的真实 baseline observations 为准。没有将静态推导包装为实测结论。
