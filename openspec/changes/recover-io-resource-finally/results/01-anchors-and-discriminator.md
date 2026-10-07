# 1.1 锚与判别变量（插桩，HEAD + 本片二进制）

## 判别变量：两行 vs 一行

`IO.countLines` 与 `LK.put/take/tryLockQuick` 的异常表（javap，双腿一致）：

| 方法 | 行集 | 保护范围 | 副本接收者 | 完成形 |
| --- | --- | --- | --- | --- |
| `IO.countLines` | **2 行**：`[25,45) -> 52` + `[52,54) -> 52` | 25–44（含循环 `27→36→27`） | **资源局部** `aload_1`（定义在范围前的 BCI 24） | `SavedReturn`：`istore 4`(43) 在体内，`iload 4`(49)/`ireturn`(51) 在副本后 |
| `IO.readAll` | 2 行：`[17,37) -> 44` + `[44,46) -> 44` | 17–36 | 资源局部 `aload_1`（定义在 BCI 8） | `Transfer`(41) → 尾 `53..57`（`aload_2; toString; areturn`，与语句同块融合） |
| `LK.put` | **1 行**：`[7,46) -> 56` | 7–45（含循环 `7→11→14/26`） | **实例字段读** `aload_0; getfield lock`（采集调用 `lock()` 读的同一 SSA 值） | `Void`：副本后 `goto 66`，方法自身的 `return` |
| `LK.take` | 1 行：`[7,50) -> 59` | 7–49 | 同上（字段读） | `SavedReturn` |
| `LK.tryLockQuick` | 1 行：`[10,23) -> 32` | 10–22 | 同上（字段读） | `SavedReturn`（`if` 规则内） |

判别变量三条，缺一不可：

1. **行数/行覆盖**：IO 两法的第二行只覆盖 handler 自身的绑定 store（`[52,54)`/`[44,46)` 恰为
   `astore` 的 2 字节），即 resource lowering 的自保护行；LK 单行且无自保护行。本片证书的
   cheap half 读的正是这个形状（两行、皆 catch-all、同 handler、第二行 `== (handler.bci,
   span_end(handler.bci))`、ordinal 相邻）。
2. **跨 finally 的东西**：IO 跨的是**资源句柄局部**（loop 读 + 两副本 close 同一 SSA 值，slot 唯
   一 store 在范围前）；LK 跨的是**字段读**（采集调用读的字段，方法内无写）。
3. **完成形**：IO `countLines` = `SavedReturn`（保存值在体内）；`readAll` = `Transfer` 到尾
   （本片**不**纳入，见下）。

## 插桩证据（HEAD 基线 + 本片二进制，逐条实测）

基线（父提交 `c99e26a5`）与门控实验（[02-gating.md](02-gating.md)）之外，本片用一次性探针
（`JRE_GUARD_PROBE`，实施后已全部移除）钉住下列事实：

- **基线为何拒**：region walk 在 BCI 0 不触发 guard 询问（该块的两条被覆盖指令 `iconst_0;
  istore_2` 都不可抛，canonical 图不给异常边 → `leaving_edge` = None），走到循环头 BCI 27 时
  **loop 规则先进入**（`loop_region`），随后整段退化为 `jre_region_exception_edge` +
  `jre_region_uncovered_blocks` 引注；声明层再以 `local 1 crosses a quoted fallback region`
  整方法拒。
- **询问点**：本片把 `starts_resource_guard(&current)`（cheap half，零 charge 纯读）并入
  `shared_finally_candidate` 的进入条件，于是 walk 在**行集起点所在块**（countLines: 块 0 持有
  BCI 25；readAll: 块 17 = 行起点）就询问证书，早于 loop reader。
- **证书判定（countLines）**：`rows=[0,1] slot=1 normal_cleanup=(45,49) handler_cleanup=(54,58)
  completion=SavedReturn{save:43,returns:51}`；handler 块 = `astore 5; aload_1; close; aload 5;
  athrow`（5 条）；两副本 `aload_1; invokevirtual close()V` 同目标同 SSA 值；slot 1 全方法**唯一
  store 在 BCI 24**（范围前），loop 读(27)、两 close(45/54) 的 load 都读该定义。
- **只读证书，不做别的**：`readAll` 的完成形是 `Transfer` → 本片证书直接 `Ok(None)`（不 charge
  尾指令），readAll 保持基线拒绝（登记边界，见下）。

## 本片三部件（各自独立提交/门控）

| 部件 | 内容 | 单独翻转什么（[03-component-gating.md](03-component-gating.md) 实测） |
| --- | --- | --- |
| 行集 resource-guard 证书 | `Shape::ResourceGuardFinally` + `prove_resource_guard_finally`（sibling，不动 LK 单行判据）+ region 询问点/体读取 + builder 呈现 + 声明提升 | `IOMidRead.countRemaining`（同形，无构造链/无深度需求）单独翻转 |
| 两行 `java.io` 宽化表行 | `platform_reference_argument_widens` 增 `FileInputStream→InputStream`、`InputStreamReader→Reader`（javap 转录，见 [04-widening-rows.md](04-widening-rows.md)） | `WideningProbe` 的两个实参位各自单独翻转 |
| `new@1` 深度 2→3 | `MAX_NESTED_CONSTRUCTION_LAYERS`（root 裁定纳入，单独提交） | `NestedDepth.threeLayer` / `X4.threeLayer` 翻转；4 层仍拒（边界实测） |

`IO.countLines` 需要三者同时在场（实测：缺任一即拒）——它是巡查的代表性方法（readLine 循环正是
IO 样板核心）。

## 登记的边界（本片**不**纳入，实测保持拒绝/不变）

1. **`IO.readAll` 的可观察 copy-and-store 形**（root 2026-10-07 裁定）：其 loop test 的
   `dup; istore` 目标可观察，而 copy 族的循环测试纯度判据（`unobservable_store_dance_part` /
   `LocalAssignmentTest::LoopTest => continue`）拒绝它，且该族 design 明文把"带异常 handler 的
   等价变换"列为 non-goal。本片**未**触碰该判据；`readAll` 的拒绝逐字钉在
   `tests/recover_io_resource_finally.rs`。附带：guard 体内循环局部（`c`）的声明位同属该后续片。
2. **固定 CF-16 void-loop 形**：`TestTryCatchFinally2$TestCls.test` 的完成形同为 `Transfer`
   （尾 = 方法尾声 `return`），本片证书拒绝它（`Transfer` 分支直接 None），其呈现与基线**逐字节
   相同**（门控表 `void-loop-fixed|identical`）。这正是证书"不夺取固定证书既有形状"的判据：
   证书的完成形只收 `SavedReturn`。
3. **多资源嵌套 try、close 带返回值形**：负例，双腿拒绝逐字（`IONegatives`，见
   [02-gating.md](02-gating.md)）。
4. **单行 resource-guard 形**（若某编译器只写一行、无自保护行）：walk 的 cheap half 只询问
   resource lowering 的**两行**集（与 `shared_finally_candidate` 既有 cheap-half 前置同规：询问窄于
   证明），因此这类形不被询问、保持拒绝。证明本身按同一论证会承认单行（`rows.len() > 2` 才拒），
   但今天没有入口到达它——登记在此，避免读者以为单行已被放行。
5. `LOCK` 三法（`LK.put/take/tryLockQuick`）逐字节不变（门控表三腿 `identical`）。
