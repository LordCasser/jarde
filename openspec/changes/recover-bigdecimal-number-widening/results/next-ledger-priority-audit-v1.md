# 71 单元后续队列审计

本审计只整理下一阶段候选，不启动实现。依据 2026-09-27 inventory summary 及其分组清单，并与现存 OpenSpec `tasks.md` 对照。候选最多三项；“优先”是推进顺序建议，不代表源码或验收已完成。

## 队列

| 顺序 | 71 单元 | 可执行的完整家族 | 现有 JADX 正向依据与实现入口 | 当前完成状态、依赖与缺口 |
| --- | --- | --- | --- | --- |
| 先收尾，不开新机制 | **EM-18 数组初始化/填充** | 先结束本轮 BigDecimal 两腿及 child-array / wildcard sibling 的既有验收，再选一个剩余的、真实 `fill-array-data` Java/DEX 形态做完整源集闭环；不要把多个残项合成一个无边界目标。 | `expressions-misc.md` EM-18 列出 `arrays/TestArrayFill.java`、`TestArrayFill2.java`（仅排除标为 `@NotYetImplemented` 的 `test2`）、`TestArrayFillWithMove.java`、`TestFillArrayData.java`；算法入口是 JADX `ReplaceNewArray.processNewArray`、`InsnGen.fillArray` / `filledNewArray`。 | summary 记录异构赋值、构造元素、primitive constructor conversion、child covariance 与 wildcard return 已分别有窄片；这些已验收窄片不能重复立项，也不等于 EM-18 整单元完成。当前 EM-18 仍部分已测；BigDecimal 窄片和 child/wildcard 变更各有现存 tasks，须先以 root 当前确切代码/CI状态收尾。剩余 DEX `fill-array-data`、未测效果/位置以及完整单元分母仍未闭合。不得把 `TestArrayFill2.test2` 当正向目标。 |
| 1 | **CF-16 finally 清理与完成路径** | `ImplicitCleanup.run()` 的受保护正文含分支/throw 的完整 finally 家族：原类、JADX 与 Jarde 完整类重编、验证运行；同时保留覆盖型 return/throw 和外部入口拒绝。 | `control-flow.md` CF-16 的真实 JADX 测试包括 `trycatch/TestFinally.java`、`TestFinallyExtract.java`、`TestTryCatchFinally.java` 与多个 `TestTryCatchFinallyN.java`；入口是 `MarkFinallyVisitor.processTryBlock`、`ProcessTryCatchRegions`、`RegionGen.makeTryCatch`。 | 多个直线或共享清理窄片已完成，不能重做 Test7/12/13/14/16 等已有验收。当前明确缺口是 `recover-proved-finally-cleanup/tasks.md` 2.3/2.4/3.1/3.2 与 `recover-structured-finally-bodies/tasks.md` 1.2/2.1/2.2/3.2：非直线子 Region 的所有权、清理异常覆盖完成语义、来源闭包与失败原子回退。优先用已冻结 `ImplicitCleanup.run()` 继续这一家族，避免新增泛化 finally 规则；CF-17 的 `TestTryWithResources` 是唯一未实现目标，不得冒充正向 fixture。 |
| 2 | **DT-26 lambda 捕获** | 以真实 `int[]` 捕获并在 lambda 内执行 `t[0] += i` 的完整类为起点，闭合数组元素捕获值的创建时类型/效果证明，再整类 Java 8 重编及行为比较；局部捕获、泛型 `List.removeIf`、复杂短路体作为不同后续子形，不顺手纳入。 | `declarations-types.md` DT-26 列出 `java8/TestLambdaExtVar.java`、`TestLambdaExtVar2.java`、`TestLambdaInstance.java`；实现入口为 JADX `CustomLambdaCall` 与 `InsnGen`。 | 已完成的 `recover-lambda-primitive-array-capture` 只覆盖 P02 原生多维数组分配类型的窄片，不能视为本项数组元素捕获已恢复。summary/declarations-types 记录双 javac 的数组元素捕获实际拒绝，位置 `lambda.rs:774`，理由是 SSA 捕获操作数为 `Object`，站点描述符/实现需要 `int[]`，创建时转换与效果未证明；该具体缺口尚未见独立 OpenSpec 任务。先单独冻结该完整类与实际拒绝/来源，再确定证明复用点；不要改写已通过的原生数组创建片。 |

## 排除与推进约束

- EM-18 的异构赋值、constructed-element、primitive conversion、child-array covariance 和 reifiable wildcard return 是不同已立/在收尾的窄片。新阶段应先读各自最新 `verification-root.md`、未勾 tasks 与 root 确切 CI，不可重开已验收范围，也不可用其窄片结果宣称 71 单元 EM-18 完成。
- CF-16 不是再测一遍简单副本；当前可执行差距是分支/throw 子 Region 与 Java completion priority。现有 tasks 中 root-only 验收未完成，应由该完整家族收束。
- DT-26 的 primitive-array-capture 旧片和 `TestTryWithResources` 的 JADX 未实现状态均不是新正例；本队列只选前者之外、inventory 已记录的真实正向 lambda 捕获测试及明确未覆盖的 array-element consumer。
- `recover-committed-local-multireads` 在 tasks 中明确阻断，且没有可按完整正向家族推进的已冻结入口，本次不列入候选。不要按未勾选数量排序；先判断 task 是实现缺口、root 文档验收还是已被后续证据覆盖。

## 判定边界

inventory 的 71 是语法验收单元数量，不是完成率。只有被冻结的原始 class、完整 JADX/Jarde 源集合、隔离 Java 8 重编、`-Xverify:all` 运行与逐项行为/来源证据齐备，才能把家族标为已验收。本文没有运行 Cargo、Git、Java、CLI 或目标程序，也没有修改产品、tasks 或账本。
