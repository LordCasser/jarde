## 1. 冻结三方证据

- [x] 1.1 仿照 JADX `invoke/TestVarArg` 构造 Java 8 同类调用 fixture，含 `int...`、`Object...`、单元素 `String...`、普通 `int[]`、数组调用 lowering 及效果计数；冻结源码、class 哈希、`javap`、原/JADX/Jarde 基线和重编运行结果。原/Jarde 重编执行逐行相同；JADX 的简单正例和普通数组负例对齐，但重载、null 与协变数组控制暴露语义偏差，准入以原 class 为判据。源中写出的 `new int[]{...}` 与 varargs 实参在 javac lowering 后不可区分，仅作运行语义对照。
- [x] 1.2 增加同名重载、`null`/数组单元素、局部数组及低预算/取消对照，记录显式数组保留原因；定向集成测试和三方完整类运行验证原 class、旧 Jarde、新 Jarde 的语义。缺失/错误目标由精确 owner/name/descriptor 与缺失/竞争成员表单测覆盖；JADX 偏差单独记录为拒绝边界。

## 2. 精确目标与数组证明

- [x] 2.1 在同次选定物理类的成员事实中绑定调用的 owner/name/descriptor，核对 `ACC_VARARGS`、末参数组、无同名竞争、已证 `Object` 父类；定向单测覆盖错误 owner、非 varargs、重载、缺失成员事实及未闭合继承面。
- [x] 2.2 仅消费现有完整数组初始化表达式，并要求该调用为唯一消费者、数组类型精确匹配且单元素不可能是固定数组实参；本变更集成正反例覆盖副作用顺序、异常时序和持有数组保留，数组闭合与额外消费者拒绝复用 `tests/p3_array_initializers.rs::complete_initializer_chains_have_stable_bodies_and_claim_every_source` 及 `tests/p3_nested_array_initializers.rs::ordering_and_child_extra_use_do_not_become_nested_literals` 的既有证明/负例。

## 3. 原子源码投影

- [x] 3.1 在现有调用表达式构造中把已证明的末参初值按序转为调用实参，保留数组分配、元素写入及调用 BCI；未证明的调用继续输出显式数组，定向测试检查正文、source map/BCI 和预算停止时不发布半展开（analysis_steps=1 只验证原子发布边界，不声称预算穿越了投影中间态）。
- [x] 3.2 用冻结 fixture 完整类运行 `javac --release 8` 与 `java -Xverify:all`，对照原/JADX/旧 Jarde/新 Jarde 的值、效果次数及异常；确认声明仍是 `T...`，普通数组仍是 `T[]`。

## 4. Root 独立验收

- [x] 4.1 Root 用独立 target 重建 CLI，重放原/JADX/Jarde 的 Java 8 完整类编译与 `java -Xverify:all`：Jarde 与原 class 的 10 行输出逐行相同，JADX 的 `null`/协变数组两项确有偏差。`jarde-java` 库测试 214/214、两项变更的集成测试合计 6/6、corpus fingerprint 5/5 通过；`cargo fmt --all -- --check`、`git diff --check`、两个 OpenSpec strict validate 均通过，预算/取消集成测试无半展开。独立 Cargo target 验收后清理。
