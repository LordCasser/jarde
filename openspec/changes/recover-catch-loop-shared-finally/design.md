## Context

[固定 Test3 证据](../../evidence/java-syntax-2026-09-28/cf16-test3-catch-finally/README.md)钉住方法 `test(ClassNode,List)` 的 BCI 0–74：正文的 `load` 和 foreach 为 `[0,38)`；正常清理在 38–42，具名 catch 的日志在 45–53、其清理在 58–62，catch-all 在 65–73 进行第三次清理并重抛原 Throwable。四行异常表依次为 `[0,38)→45 Exception`、`[0,38)→65 any`、`[45,58)→65 any`、`[65,67)→65 any`。末行只保护 handler 的异常绑定，不保护 `unload()`。原类和固定 JADX 的正常/visitor 异常路径相同；主线 Jarde 对该方法安全拒绝。

已有 `SharedFinally` 的 joined completion 可表达一个具名 catch、三份清理与共同返回；现有 3 行证明只识别静态无参调用、布尔字段写入或 append，并且没有 handler 自保护行。Test11 的 handler 局部循环事实用于异常副本自身循环；这里的循环在正文中，按现有方法入口循环处理。Test4 的四行证书为清理内部具名 catch，不能复用其异常拓扑。

## Goals / Non-Goals

**Goals:** 在固定 Java 8 四行布局中保持循环/日志/清理顺序、异常覆盖和原异常身份，用既有 `SharedFinally` joined output 生成方法级 `try/catch/finally`；全部目标 BCI 有来源；有效近邻及预算/取消安全拒绝。

**Non-Goals:** 任意四行或任意 Java 编译器 lowering 的泛化，DEX/D8，静态 `LOG` 的 `<clinit>` 恢复，或把修改过的整类源码视作 Jarde 原始 class-source 验收。

## Decisions

1. **扩展现有共享 finally 证书，不新增 pass 或 AST 节点。** 在 4 行路径下先按本形态的行顺序、catch 类型、范围、handler 起点和自保护绑定做闭合预检；第 4 行只作为来源中性的 handler 绑定行，不能变成第二个 `finally`。为现有 `SharedFinally` 计划显式记录可选的绑定行，让旧 3 行证书仍按原约束工作。Region 继续用前 3 行建立正文和具名 catch 的边界，但必须把第 4 行异常边和 handler 块的来源一同记账。另一种做法是复制一套四行 Guard/Region/Build；它会重复 joined completion 和 catch 构建，未来容易在异常所有权上分歧。
2. **实例调用副本以接收者和值证明。** 三份 `aload_0; invokevirtual ClassNode.unload:()V` 只有在调用符号、调用种类、返回/参数 descriptor、三次接收者的 SSA 均指向同一方法入口 `ClassNode` 值时才合并。正文 loop 的 `List.iterator/Iterator.hasNext/next` 与 `DepthTraversal.visit`、catch 内 `LOG.error` 由既有 Loop/Catch/Builder 结构表达；证书核受保护范围、单入口/回边/出口和所有 canonical 边，不能只比较三个 `unload` 名称。清理调用在四行范围之外，任何新增对清理的异常覆盖都拒绝。
3. **异常完成一次提交。** 证明 handler 的 store/load/throw 消费同一个 Throwable，catch 正常流和正文正常流只在共同 return 汇合；具名 catch 的日志抛错必须经 catch-all 的一份清理后传播，`unload` 抛错覆盖先前异常且不重新进入清理。Region/Builder 在所有物理块、局部声明与目标方法 BCI 都可呈现后才发布 joined `try/catch/finally`；否则回滚并保留现有 fallback。借鉴固定 JADX `MarkFinallyVisitor.findCommonInsns` 沿三个出口找副本的次序，但不复制其按相似指令标记 `DONT_GENERATE` 的策略。
4. **把整类 `<clinit>` 缺口作为独立质量边界。** 原/JADX 完整类可重编；Jarde 目前未投影 `LOG = LoggerFactory.getLogger(...)`，所以本次用原声明/stand-in 构造方法级 harness，并核 Jarde `test` 的 AST、源码片段和运行行为。报告必须明确整类 Jarde class-source 仍可能不能编译，不能将 harness 的人工类头算作整类成功。后续类初始化任务独立处理。

## Risks / Trade-offs

- **旧 3 行共享证书被放宽** → 新四行条件单独预检，旧测试与 Test4/11、两/三/四/五行 finally 全量回归；任何额外行、边或副作用拒绝。
- **正文循环/日志可能局部回退** → Region 先完整取得两个子区域，Builder 只在无 fallback、无未声明局部且来源覆盖完整时发布，拒绝时不留下半个 `try`。
- **自保护行或异常覆盖误读** → 固定表与 verifier 有效近邻逐行核对；将 `load`、visitor、logger、`unload` 抛错分别作为可观察路径，明确 handler 自行抛错不会重入自身。
- **整类静态字段初始化不属于本 change** → 方法级 harness 与 class-source 状态分开报告，在 CF-16 账本继续保留整类缺口，不将 Test3 文件标成完整追平。
