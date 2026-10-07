# 1.2/1.3 切片：逐锚定位（instrumentation）

日期 2026-10-07；二进制 = clean HEAD（`target/debug/jarde-cli`，`git status` 仅含本切片新增文件）；
渲染与 region 树见 `baseline/`，两份 gating 补丁见 `patches/`，其结果见 `02-gating.md`。

## 1. 两个锚今天在哪里拒绝

**锚 12（explicit-lock，`lk.jar`）**

| 成员 | 渲染结局 | 拒绝句（逐字） | region 树 |
| --- | --- | --- | --- |
| `put()V` | `explanation_only` + `jarde_refused_body();` | whole-method quote（BCI 56 的引注块在 BCI 65 控制流退出） | `Straight[0]` + `Fallback(exception_edge @7)` + `Fallback(uncovered [15,27,56,66])` |
| `take()I` | 半正文（`this.lock.lock();` 后整段引注） | `block at BCI 7 leaves through exception handler 0: a handler's shape is not part of the recoverable subset` + `3 live block(s) … [14, 26, 59]` | `Straight[0]` + `Fallback(exception_edge @7)` + `Fallback(uncovered [14,26,59])` |
| `tryLockQuick()Z` | 半正文（`if` 两臂，臂内引注） | `BCI 32: the exceptional path repeats code the normal path also runs — the \`finally\` copy javac emits for a \`finally\` clause; this candidate lacks the complete straight-body, copy, range, and ownership proof needed to merge them into one \`finally\`` + `1 live block(s) … [32]` | `Fallback(jre_guard_finally_copy, rule=if, blocks [0,10,42])` + `Fallback(uncovered [32])` |

锚 12 的每一句都是 **region/guard 层**的话：`jre_region_exception_edge`（`region.rs:284`）与
`jre_guard_finally_copy`（`guard.rs:767`，`Unproven::FinallyCopy`）。**声明规划没有对这两个方法说
任何话**（`take()` 的局部 1/2 与 `tryLockQuick()` 的零局部见下）。

**锚 13（io-wrapping，`io.jar` 与 `ScopePlanCrossing`）**

| 成员 | 渲染结局 | 拒绝句（逐字） |
| --- | --- | --- |
| `IO.countLines(Ljava/lang/String;)I` | `explanation_only`（整方法引注） | `local 1 crosses a quoted fallback region; its assignments and consumers cannot be presented as one lexically bound definition-use slice` |
| `IO.readAll(Ljava/lang/String;)Ljava/lang/String;` | 同上 | 同上 |
| `ScopePlanCrossing.resourceAcrossFinally` / `flatFinally` | 同上 | 同上（本切片 fixture，`@bytecode 0 27 36 42 52` / `0 15 28`） |

这一句**是**声明规划的话（`build.rs:891`），但它只是**下游症状**：`IO.countLines` 的 region 树同样是
`Straight[0]` + `Fallback(exception_edge @27)` + `Fallback(uncovered [36,42,52])`（见
`baseline/region-trees.txt`），资源句柄 `local 1` 在 handler 里被读（BCI 54，落在引注区），所以规划
先按“证据不完整”拒绝整方法体。

## 2. 逐局部的三分类轨迹（为什么锚 12 到不了规划）

* `LK.take()`：局部 1 = 保存的返回值（写 BCI 49、读 BCI 57），局部 2 = handler 的异常临时（写 59、读 67）。
  两者的读写**都落在同一个引注 region**（uncovered quote `[14,26,59]` 覆盖 49/57 与 59/67），
  因此 `spans_regions = false` → `Local { owner }`（`build.rs:873`）：规划给出“词法 owner”，
  不产生任何拒绝句；引注区不写语句，声明也就无处可写。锚 12 的 gate 在规划**之前**。
* `LK.tryLockQuick()`：方法没有局部（只有 `this.lock`/`count` 字段访问），规划无话可说。
* `IO.countLines()`：局部 1（`r`：写 24 在 lead、读 27/45 在保护区、读 54 在 handler）跨 region 且含
  fallback → `Incomplete`，owner = 方法体（`build.rs:882-895`）。局部 4（保存返回值 43→49）与局部 5
  （异常临时 52→58）同样 `Incomplete`（同一 owner，先到者留下句子）。
* `IO.readAll()`：同一形状（`local 1` = `FileReader`）。

## 3. 顺序事实：region 树先于声明规划

`build.rs:7668` 的 `declarations(regions, canonical, ssa, …)` 把 `regions` 当**输入**；region 树由
`region.rs` 的 walk 在本函数之前建好。因此规划**不可能**消除一个 region fallback：锚 12 的
`Fallback(exception_edge @7)`、锚 13 的 `Fallback(exception_edge @27)` 都在规划运行前就定下来了。

## 4. 1.2/1.3 已有实现的三分类（本切片复核）

| 决策 | 落点 | 守卫 |
| --- | --- | --- |
| `Local { owner }` | `build.rs:873`（所有访问同一 region）与 `build.rs:983` | 声明由首个写入就地携带 |
| `Elevated { owner }` | `build.rs:1219` + `at_region` | 覆盖校验 `validate_declaration_placements`（`build.rs:1836`）；跨异常区还要 `cross_exception_store_type_is_proven` 与 `all_reads_reach_presented_writes`（`build.rs:1188-1197`） |
| `Incomplete { owner }` | `build.rs:857`（无 owner 的未归属使用）、`882`（fallback 使用）、`936`（catch 参数逃逸）、`956`（无完整 owner）、`1199`（跨保护区但 DA 未证）、`1238`（覆盖校验失败） | 拒绝 owner 所在 region，正文不提交（`build.rs:1894-1899`） |
| catch/resource 子作用域 | `catch_parameter_stays_in_clause`（`build.rs:3412`）、`resource_slots`（`build.rs:789`） | 只在子路径可见 |
| 未归属使用阻断 | `common_owner` 返回 `None`（`build.rs:3377`） | owner 为空路径 = 方法体 |

这些落点就是本切片新增测试所钉的行为（`tests/preserve_local_scope_plan.rs`）。
