# 插桩定夺（2026-10-06，coder）——两案对照与落点实测

本文件记录实现前的读码结论：预审计的三条断言逐条复核、两案（内层 `returns` 字段 vs 合成 temp 局部）
的对照事实，以及最终落点。全部结论来自本工作树代码与 `javap -c -p` 实测，无预测性陈述。

## 1. 预审计落点复核（机制为准，行号已漂移）

| 预审计断言 | 复核结果 |
|---|---|
| `Shape::Monitor { returns }` 的文档要求 `return` 写在该层 braces 内 | 成立：`guard.rs` 的 `returns` 文档（"the return is written *inside* the braces, and the statement continues nowhere"） |
| `InnerMonitor`（`guard.rs:160`）**没有** `returns` 字段 | 成立：`InnerMonitor` 只有 `span`/`enter_bci`/`normal_exit_bci`/`body`/`handler_entry`，嵌套形下 `return` 只能属外层 shape |
| build.rs synchronized return 分支（~14525）把 return 追加到**外层 body** 的嵌套块之后，内层渲染空 | 成立：`(Some(pair), Some(tree))` 分支走完嵌套 pair 的 region 树、`close_nested_pair` 关掉内层 braces 之后，`if let Some(return_bci) = *returns { … body.push(statement) }` 才把 `return` 追加进外层 `body` —— 内层块此刻已闭合，故渲染为空块 + 外层 `return` |

复核同时确认了两条决定实现的**结构事实**：

- **求值区间**：`NL.probe` javap 实测（两腿 BCI 相同）求值 BCI 11–27（`new StringBuilder` … `toString`），
  内层 `monitorexit` BCI 31、外层 BCI 33、`areturn` BCI 34；guard 的既证要求"被 return 消费值的定义
  指令在**外层** body 内"（`*produced >= row.start_bci && *produced < exit_load`）已成立（`produced` = 27），
  缺的只是"是否在**内层** body 内"这一读。
- **闭合时机**：`return` 形下 pair 的 `span.1`（内层 handler rethrow 之后）落在 `plan.body()` 之外
  （NL：40 > 32），而 walk 在 body span 外直接返回，因此 `at >= pair.span.1` 的**走中闭合**在 return 形
  不可能触发；闭合只发生在 arm 走完 region 树之后（build.rs `close_nested_pair` 调用点）。嵌套 `goto` 形
  （`synchronized(a){ synchronized(b){ y(); } y(); }`，实测 handler 落在外层 row 内）才会走中闭合——那条
  路径没有 `return`，不受本片影响。

## 2. 两案对照（按最小 diff / 零回退面定夺）

| | (a) `InnerMonitor` 增自身 `returns`（return 入内层 body） | (b) 外层渲染合成 temp 局部（jadx 形：锁内赋值 + 锁外 return） |
|---|---|---|
| 呈现文本 | 与源同形（`synchronized(LOCK){ synchronized(NL.class){ return "n" + o; } }`） | 引入源中不存在的局部：`String localN; synchronized{ synchronized{ localN = "n"+o; } } return localN;` |
| 求值次序 | return 写在内层 braces 内：求值在内层 `monitorexit` 前 | 赋值在内层 braces 内、return 在锁外：同样保真 |
| 需要的证明 | 与既证同形：被 return 消费值的定义落在**内层 body** BCI 区间内（复用 SSA `Definition::Instruction` 读法） | 同一证明，但**另需**：合成局部命名/类型/声明位置（`Declare` + 名字去重 + `undeclared` 账）、赋值语句注入同一"闭合前"落点 |
| 新增机制 | 0：`InnerMonitor` 一个字段 + build.rs 在既有关闭点带上语句 | 合成命名/类型/声明/赋值四件套，且注入点与 (a) 相同 |
| 诊断/账本面 | 无新诊断码；语句文本除位置外逐字相同；`source_map` 锚集不变（return BCI 本就是 plan facts） | 新增合成局部后 `statement(s)` 计数、`@method` 面文本、局部名来源都变，回退面更大 |
| jadx 先例 | jadx 用 (b) 是因为它的呈现层没有"把语句放回内层"的证明结构；(a) 在本仓库结构内直接可得 | — |

**定夺：(a)**。理由不是"少写代码"，而是 (b) 在同一位置既要证明求值区间、又要新造命名/类型/声明/账本四面；
(a) 的呈现同时是**源形**与**字节码次序**的一致解，回退面最小（仅有本片目标的嵌套 return 形变化）。

## 3. 最终落点（机制）

1. `guard.rs`：
   - `InnerMonitor` 增 `returns: Option<u32>`（+ `returns()` 访问器）；
   - `monitor()` 在既有的 `returns` 判定处同时记下被消费值的 `(value, produced)`；当 `produced` 落在
     **内层 body** `(body_start, exit_load)` 内、且 `expression_inside(facts, value, pair.body)` 证明该值
     的整条表达式链（定义 + 沿 operand-stack 的读者 + 已证构造 site 的 `owned`/`expression`）都落在
     内层 body 内时，写 `InnerMonitor.returns = Some(return_bci)`；
   - 新函数 `expression_inside`：**与渲染器同链**（局部读=名字，链在此停；`Entry` 即方法输入；
     `Phi`/`Caught` 是未证的控制流边界 → 答"否"），每个访问值都计费（`facts.charge`），故深链不会
     变成未计账工作。不满足时**不改变现状**（既不新拒也不换位），保持零回退。
2. `build.rs`：
   - `NestedPairBraces` 增 `returns: Option<u32>` 与 `closed_without_return: bool`；
   - arm 在 region 树走完、**关闭 braces 之前**渲染 `guarded_return`（复用既有失败路径：`region_quote`
     + `fallback`，诊断文本不变），把语句交给 `close_nested_pair(inner_return)`，由它在内层 body 末尾
     追加——即"求值留内层、闭合在内层之后"；
   - `close_nested_pair(None)`（走中闭合）在 pair 带 `return` 时记 `closed_without_return`，arm 随即以
     `jre_guard_body` 引注该语句（fail-closed；对 javac 的 return 形不可达，见 §1）；
   - arm 末尾的 `body.push(statement)` 在 pair 自带 return 时跳过（该 return 已写在内层）。
