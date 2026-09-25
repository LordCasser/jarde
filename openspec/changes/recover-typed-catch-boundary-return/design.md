## Context

见[三方冻结证据](../../evidence/java-syntax-2026-09-25/multicatch-finally-return/analysis.md)及[行为规格](specs/java8-recovery/spec.md)。独立 `PlainMultiCatch.choose` 与组合探针 `choosePlain` 的命名异常行均为 `[0,32)→33`。规范块 BCI 30 含 `ldc` 与 BCI 32 `areturn`；`region.rs::exception_edge_accounted` 当前要求异常表行尾后所有指令都是 `Operation::Transfer`，因此不能结算块上的命名异常边。现有 `Catches` 已正确合并同处理器的两条记录。组合探针的 `chooseFinally` 还有 catch-all 和三份清理副本，在已有 finally change 的证明范围内。

## Goals / Non-Goals

**Goals:** 在已有 `Catches`/Region 流程中，仅对命名捕获半开保护范围尾部的无效果返回解除错误引用；证明返回值来源、异常边归属和物理指令只认领一次，完成整类 JVM 行为对照。

**Non-Goals:** 不新增 catch AST、异常 IR 或 pass；不改变异常表的实际半开范围；不恢复 `finally`、TWR、monitor 或一般的范围外语句；不因 JADX `chooseFinally` 可编译而模仿其错误重复清理。

## Decisions

1. **保留异常边的现有结算入口。** 在 `exception_edge_accounted` 的行/规范块交集及处理器身份核对之后，识别唯一、终止的 `Operation::Return` 后缀。继续用 SSA 指令 BCI 与原异常表行尾核对，不拆规范块、不制造新的异常边，也不跳过 `region_at` 的所有权校验。相比把整个块当成行内或扩展 catch 行，后者会错误捕获范围外指令的异常。
2. **把范围外后缀限制为无效果完成。** 除现有 `Transfer` 外，只有同块最后一条 JVM return、前面没有其它行尾后指令且没有独立效果/异常来源时才可放行；返回操作数须由此块内、行尾前已经读到的值产生，SSA 绑定唯一。值生产者必须保留在原 `try` 体语义位置。`MethodCodeFacts` 不含方法 flags；从同次 `RecoveryRequest.facts.method().access_flags()` 只读透传 `ACC_SYNCHRONIZED` 判定给 Region，命中时拒绝该特例。显式 monitor 或其它异常完成也拒绝；不能简单地把所有 `Return` 都加入现有 `Transfer` 白名单。这是现有请求事实的窄接线，不建立方法元数据副本或新分析层。
3. **保持每条边的独立证明。** 同块有两条命名行时，分别检查行区间与处理器。任何 catch-all、不同端点的行、真实外部入口或其它未结算边都继续走旧的 guard/Region 拒绝。`chooseFinally` 不能因为其中也有 `areturn` 而成为该规则的正例。
4. **验证来源和整类语义。** 新定向回归以只含普通 catch 的 `PlainMultiCatch` 钉多重 catch 文本、全部 BCI 覆盖与原/Jarde 四行一致，组合探针用于 `chooseFinally` 的负边界；范围外含可能抛错操作和 catch-all 继续引用。用源码级 `javac --release 8` 与 `java -Xverify:all` 验证独立完整类的真实完成路径，不能为了使组合探针整类可编译而顺带修其 finally。JADX 只作为对照，不作 oracle；其 finally 输出已在 2/4 行发生重复清理。

现有 decoder、Canonical CFG、SSA 和构建器已提供所需事实，复用它们比引入新库或第二套分析通道更小。实现不改变输入解析、Java 8 方言验证、运行时选择或 JVM 校验：本变更只决定已解析并经 verifier 有效的字节码何时可投影为源码。

## Risks / Trade-offs

- `[风险]` 放宽后把行尾后的可能抛错表达式写进 `try`，改变被命名 handler 捕获的异常 → 只接纳精确终止且无效果的 return，值来源与物理 BCI 必须核对；负例维持引用。
- `[风险]` 同块中的隐藏后缀或同步方法完成改变可观察行为 → 遇到未知指令、monitor 或无法证明的完成边界拒绝，不猜测 Java 源级形态。
- `[风险]` 与已有 finally/resource guard 抢占范围 → 维持 guard 优先级与 catch-all 拒绝，不扩展本规则到 `chooseFinally`。
