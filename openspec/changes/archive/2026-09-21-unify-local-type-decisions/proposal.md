## Why

提升声明（`declarations()` 路径）不读「值来自本 body 已声明 boolean 的局部」这条证据，于是同一个值在两条声明路径上被定成两种类型（[完成复核](../../completion-review.md) 的 T3 行与其「就地声明与提升声明没有共享完整证据」一节；本 change 采用该行记录的数值，并已按当前代码位置复现）：

```java
public static int hoisted(boolean b, int n) {     // 复核的样本
    boolean a = b;
    boolean c;
    if (n == 0) c = a; else c = b;
    if (c) return 1;
    return 0;
}
```

`c` 由提升路径定成 `int local3`，随后被赋入 boolean 的 `a`／参数 `b`，再写成 `if (local3 != 0)`；报告为 Complete/Structured，把正文放进成员自己的签名后 javac 拒绝。第二个形状（literal-armed 的 hoisted 变量）

```java
public static int literalArmed(boolean b) {
    boolean x;
    if (b) { x = true; } else { x = false; }
    if (x) return 1;
    return 0;
}
```

复现结果是 `int local1; local1 = 1;`：类型与源码不同，但文本自洽。两个形状的性质不同，本 change 分别说清（见 design 决策 6）。

根因（复核的表述，本 change 已按当前源码核对）：`crates/jarde-java/src/build.rs::declarations()`（约 203 起，类型决定约 276–290）只用 `boolean_proof`（`build.rs:3104`：`Z` 参数槽、返回 `Z` 的调用、descriptor 为 `Z` 的字段读取），而**就地**声明路径 `declare()`（`build.rs:1618`）用 `boolean_evidence(stored) || boolean_local(stored, at)`——后者认识「本 body 已声明 boolean 的局部」（`build.rs:1512`）。同一个值因此得到两个答案，**类型取决于遍历顺序**；`write_statement`（`build.rs:1556`）遇到既有 int 声明仍直接赋入 boolean，没有校验也没有拒绝。`declare()` 的返回值把两种含义压进同一个 `Ok(None)`：`build.rs:1621`/`1628` 的「没有声明到期」与 `build.rs:1653`/`1670` 的「声明已经失败、fallback 已写」，调用方在后者之后仍会继续写那条赋值。

缺陷的底层形态（复核的框架）：**类型由多个位置各自猜、表达式由多个位置各自建**，修一处不会让别处停止产出矛盾结果。本 change 把这一个决定交回唯一所有者：**声明规划（declaration planning）为每个局部变量作一次类型决定**，按既有 `LocalVariable` 身份索引，由提升声明、就地声明、赋值、条件与返回消费；没有第二个位置再猜。

## What Changes

- 规划产出**一个类型结果**（`LocalVariable → 类型`），在既有声明规划里计算，MUST 覆盖该变量的全部写入（不只是第一次写入）并按有限证据清单决定：类文件 descriptor 事实（`Z` 参数槽的读取、返回 descriptor 为 `Z` 的调用结果、descriptor 为 `Z` 的字段读取）与「一次对本 run 已判为 boolean 的局部的读取」的传播；MUST NOT 跟随值链，MUST NOT 把 `0`/`1` 字面量重新接纳为声明的**发起**证据（它只在目标已经确定为 boolean 时按 `true`/`false` 适配）。
- 传播 MUST 用一个**有界工作队列**，接线到既有预算与取消检查（`stop.rs` 的 `charge`/`poll` 与既有已计费维度）；规划本身 MUST NOT 成为绕过预算的新无限工作量，预算或取消在规划期停止时发布既有停止。
- **消费点唯一**：提升声明、就地声明、赋值、条件与返回都读同一个结果，MUST NOT 再各自决定；两条声明路径对同一个值 MUST 给出同一个类型，遍历顺序（分支先后）MUST NOT 改变结论。
- **未知或冲突的类型终止该结构**：有限证据不能让它一致时，受影响结构 MUST 按既有 refusal 契约拒绝并保留 bytecode 与 origin，MUST NOT 发布猜测或类型矛盾的赋值（例如 `int local3; … local3 = <boolean 值>;`）。
- **`declare()` 的两种含义分开**：修复前 `Ok(None)` 同时表示「没有声明到期」和「声明失败、fallback 已写」；修复后两种结果 MUST 可区分，调用方在「已拒绝」之后 MUST NOT 继续写那条赋值。
- **验收**：两个形状与对照先复现（修正前完成），修正后以精确文本 + 成员自己签名下的 javac 编译 + 两侧执行对照验收；交换分支顺序 MUST 给出同一结论；跨分支复制经另一个 boolean 局部的链 MUST 成立；提升与就地 MUST 一致；纯 `int` 对照 MUST 保持；拒绝路径 MUST 终止该结构而不是继续发射。

## Capabilities

### New Capabilities

无。不新增 crate、公共模块、类型模型/`Type` 变体、报告平面或停止分支。

### Modified Capabilities

- `java8-recovery`：新增「一个局部的类型在语句构建之前只决定一次」的要求——规划产出按变量身份索引的类型结果、有限证据清单与逐写入一致性、有界工作队列接预算/取消、两条声明路径一致、未知/冲突终止结构、`declare()` 两种结果可区分。
- `recovery-validation`：新增「局部类型决定必须被编译器接受并与执行一致」的验收要求——两个形状的编译执行对照、分支顺序不变、拒绝边界，以及变异必须恢复「提升路径只读 descriptor」与「只读第一次写入」的行为。

## Impact

实现落在 `crates/jarde-java/src/build.rs`（`declarations()` 的规划与 `HoistedDeclaration`、`declare()` 的结果类型与 `write_statement` 的消费、`boolean_proof`/`boolean_evidence`/`boolean_local` 的证据入口；`value_type` 只读）。本 change 消费 [decide-comparison-contexts](../decide-comparison-contexts/proposal.md) 定下的「值自身的证据 / 目标位置的要求 / 字面量适配」三分（条件与返回是本规划的两个消费点），但不在本 change 里重开该决定。本 change 与 [decide-comparison-contexts](../decide-comparison-contexts/proposal.md)、[re-express-string-concatenation](../re-express-string-concatenation/proposal.md) 都改 `crates/jarde-java`（`build.rs`、`emit.rs`；本 change 落 `build.rs`），三者 MUST 按此顺序**串行**实施（本 change 为第 2 个），彼此不并行改动同一文件。

交付包含受控 fixture（源码、class 字节、来源 README、`tests/fixtures/README.md` 登记、fingerprint 再生成、reader fixture census 更新）、精确文本与拒绝边界回归、javac 编译与执行对照、变异与正向对照。反例 MUST 在修正前先记录（命令、正文、javac 的拒绝信息）。

非目标：不新增 crate、依赖、verifier 或新的类型模型/`Type` 变体；不做数据流类型推断或值链跟随，不把字面量重新接纳进声明的发起证据；不改 `value_type` 对其它类型的决定、不改 `emit.rs` 的打印规则；不 spawn 工作线程、不使用 `catch_unwind`；不新增拒绝词汇、报告平面或停止分支（拒绝走既有 fallback/refusal 契约）；不重开 R8/R9 与已归档的递归界、concat、boolean 上下文、数组类型修正；不声称类型保真或一般语义等价，也不声称覆盖整类类型缺陷；不做性能工作（`optimize-demand-workloads` 保持 0/22）。当前仅完成修正规划，实施任务全部待办；历史归档与既有验证记录保持原状。
