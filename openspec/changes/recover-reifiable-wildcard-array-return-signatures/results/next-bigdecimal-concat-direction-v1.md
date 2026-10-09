# BigDecimal 案例中的 `arraylength` concat 边界

## 结论

当前 `jre_concat_interleaved_effect` 是 concat 候选检查对 `Operation::ArrayLength` 的保守拒绝：它处在 StringBuilder 的 `init` 与后续 `append` 之间，但 concat 的 operand producer 白名单没有它。Builder 已能把该值渲染为 `array.length`，concat renderer 也按 append 顺序逐项渲染表达式。因此可以先用**仅放行 `ArrayLength` 作为 append operand 的值表达式**验证复用现有链路；静态代码没有显示必须新增 concat 表示、Builder renderer 或额外 pass 的理由。这个方向是候选，不是已实现或已复测的结论。

该变更只能拆掉第二个拒绝。BigDecimal 元素写入 `Number[]` 的 reference-assignability 缺口仍独立存在；在该赋值事实没有证明前，完整 `main` 仍不能恢复。另有 `System.out` field 未进入正文和数组拒绝后 `values` 局部未绑定的派生现象，不能混称为本 concat 根因。

## 当前物理形状与拒绝点

本次 `candidate-root-v1` 的真实双 JDK 输入、原始双流、完整报告以及对应 class 身份记载于同目录的 [next-bigdecimal-baseline-audit-v1.md](/Users/lordcasser/workspace/projects/jarde/openspec/changes/recover-reifiable-wildcard-array-return-signatures/results/next-bigdecimal-baseline-audit-v1.md)。其 `main` 关键顺序为：`StringBuilder` 的 `new/dup/init` 在 BCI 20/23/24，数组的 `arraylength` 在 BCI 28，随后 `append(int)`、`append(String)`、`append(Object)`、`toString` 在 29/34/40/43。报告在 concat 层记录 BCI 28 的 `jre_concat_interleaved_effect`，具体原因是当前 walk 将 append calls 写成一个表达式会跨过该指令。

[concat.rs](/Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/concat.rs:1575) 将 append 链范围内的指令分成可作为 operand 的表达式和会让链重排的 statement。当前白名单是 `Push`、`Load`、`Arithmetic`、`Negate`，以及有后续链内 reader 的值型调用；其他已解码操作落到 `StatementFree` refusal，并指出顺序或执行次数可能变化。[`ArrayLength` 不在该白名单](/Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/concat.rs:1631)，所以它被统一归入拒绝分支。这里不是“JVM 的 arraylength 必然有外部副作用”：该指令是一个会在空引用时抛 NPE 的值产生操作，当前规则尚未把这种值形纳入可内联 operand 集合。

Builder 对它已有直接渲染路径：[build.rs](/Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/build.rs:25242) 要求该指令有唯一 array stack read，再递归渲染数组值，产生 `ExprKind::ArrayLength`，来源锚定在该 BCI。concat renderer 会按 `chain.appends` 的顺序逐个读取 append 参数，调用既有 `render_value`，并把 append BCI 作为 derived origin；concat 节点保留链拥有的 BCI 集合（[build.rs](/Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/build.rs:27839)）。这意味着一旦 verifier 将 `ArrayLength` 作为合法的值 producer 纳入 `owned`，已有 renderer 已有表达它和映射其来源的形状。

顺序条件也有现成基础。每个 append operand 都必须由本方法内某条指令产生，且 producer BCI 严格位于前一个链指令与当前 append 之间；随后 parts 依 append 顺序渲染。故对于该形状，BCI 28 的 `values.length` 会留在 BCI 29 对应的第一个 append operand 中，之后的字符串常量和 `values[0]` 仍按 34、40 的顺序进入同一个 concat 表达式。NPE 仍发生在求该 part 的位置，而不是把链外的效果任意跨越过去。这个判断来自当前静态调用顺序，尚无本片候选代码或新测试运行结果。

## 最小实现边界与验证

若 root 后续决定单独处理该边界，最窄改动点是 `concat.rs` 中 operand producer 的操作分类：只将 `Operation::ArrayLength` 纳入既有可呈现值集合，并沿用当前 `produced_at` 顺序约束、append reader、链 `owned`/alias 检查和 `StatementFree` 默认拒绝；无需新增 AST 节点、Builder 规则、pass 或全局 effect 模型。应复用现有表达式 renderer，使 `ExprKind::ArrayLength` 继续携带直接 BCI origin，concat 再保留 append 与链节点的 derived origins。

验收至少应隔离并覆盖：

- 正例：数组赋值类型事实已知且闭合，concat 在 append operand 中读取数组长度；完整方法呈现 `values.length + ":" + values[0]` 对应的加法/拼接，BCI 28 仍出现在 source map，左右 append 顺序不变。
- 空引用边界：长度操作不能被删除、提前到先前 append 之前，也不能重复求值。检查 renderer 对 array operand 的实际 SSA 身份与已有 reader/ownership 证明；不能仅凭 `ArrayLength` enum 白名单宣称任意 array expression 均安全。
- 负例：在 builder 链内部插入 void call、局部 store/`iinc`、数组写入或其他 statement-producing operation，仍由原 `StatementFree` 分支拒绝；不把新白名单泛化为所有“看起来能返回值”的操作。
- 类型隔离：用当前已证明可赋给 `Number[]` 的包装类元素（例如 `Integer`）单独检查 concat，避免 BigDecimal→Number 类型拒绝掩盖 concat 路径；之后再对 BigDecimal 完整程序复测两类拒绝是否分别消失。
- 双 JDK 正源完整重编译和原始 stdout/stderr 对比；仅 exit 0 或仅无 bytecode marker 不足以证明语义正确。

这些是后续验收建议，不代表当前已执行。特别是原 BigDecimal 例子在 assignment gate 修复前，即便 concat 生成链也仍应因 `aastore` 类型事实失败而拒绝整正文。

## JADX 对照的范围

本机源码 [SimplifyVisitor.java](/Users/lordcasser/workspace/testzone/jadx/jadx-core/src/main/java/jadx/core/dex/visitors/SimplifyVisitor.java:323) 的 `convertInvoke` 识别 `StringBuilder.toString()`，从嵌套 invoke 参数用 `flattenInsnChainUntil` 收集调用链，或从 SSA register 用 `collectUseChain` 收集使用链。后一条路径要求 builder 的 use 链没有 phi，且链指令同 block、连续（343–384 行）。`convertStringBuilderChain` 将 constructor 的初始对象参数和每条 append 的 operand 收集为 concat 参数（387–413 行）；`getArgFromAppend` 仅取 append 的第二个实参（542–550 行）。它不按 Jarde 的 JVM BCI walker 判断每个中间 opcode；嵌套的 arraylength 若已成为 append operand 的 `InsnArg`，转换会保留该参数节点，再构造 `STR_CONCAT`。这说明该表示方向在 JADX 的 IR 形态中可表达，不证明 Jarde 的 verifier 证明已足够，也不替代当前双 JDK 复测。

JADX 已有 StringBuilder 正例测试覆盖常量、变量及 primitive/ref append：[TestStringBuilderElimination2.java](/Users/lordcasser/workspace/testzone/jadx/jadx-core/src/test/java/jadx/tests/integration/others/TestStringBuilderElimination2.java:14)。负例 `TestStringBuilderElimination3` 在两个 append 之间调用 `updateF()`，期待保留 `StringBuilder`，展示其效果顺序保护（[第 25–57 行](/Users/lordcasser/workspace/testzone/jadx/jadx-core/src/test/java/jadx/tests/integration/others/TestStringBuilderElimination3.java:25)）。本次只读检索未找到专门覆盖 `append(array.length)` interleaving 的 JADX 测试；不把算法结构或历史 BigDecimal 双腿成功运行等同于这项新边界的专门测试。

## 明确不处理

本备忘只讨论 `arraylength` 作为 concat 链 operand 的顺序拒绝。BigDecimal 是否可赋给 `Number`、其是否能由所选 runtime snapshot 证明、`System.out` field 呈现、以及 refusal 后未绑定局部的派生质量问题均保持独立。本次没有改产品、测试、fixture 或 OpenSpec 任务，也没有运行 Cargo、Git、rustfmt、Java、JADX CLI 或 Jarde CLI。
