# TWR 包围具名 catch 切片（`recover-enclosing-named-catch`，CF-17b）证据 — 2026-09-30

[巡查 README](../README.md) 根因 2 的实现侧证据：`guard.rs::enclosing_clauses` 在既有"分片具名行"读取之外新增**全跨度具名行**分支——`row.catch_type_index.is_some() && row.start_bci <= shape_start && row.end_bci >= handler_end`（原排除子句的取反即接受条件），外加 handler 块在 claim 外（`!own_handlers.contains`）与至多一行全跨度行（两行即双 catch/multi-catch，保持拒绝）；全跨度行的 handler 须证明为**一个直行块**（入口绑定 store 命名参数 [`handler_binding`]，其余指令在语句白名单 + 值 `return` 之内，块唯一 normal 后继即语句续块——纯 `goto` 桥按 `continuation_of` 折叠），不满足即维持 `jre_guard_unexplained_row` 既有拒绝。catch-all 包围行（`catch_type_index: None`）在两个分支都不可成为子句，继续 Unexplained（那是 finally/TWR 自身语义）。

呈现链路：`Plan` 增 `enclosing: Option<Box<EnclosingCatch>>`（row 序号、catch 类型池索引、参数槽、handler 块、体范围）；clause 的 handler 入口记入 `plan.facts`（catch 读者据此把该行视为规则已消费、整体让位）；handler 块记入 `plan.owned`（覆盖账本——statement 未到达的活块才由 uncovered-blocks 扫描点名的反面）；Builder 在 TWR 头之后发射 `} catch (E e) { … }`，头拼写复用 `catch_header` 的池+名字表路径，handler 体走普通语句通道（`body_range`），绑定 store 经 `clause_parameters` 跳过、参数槽经 `resource_slots` 永不切分——与 `Catches` 的 handler 体纪律一致。实现落点 `crates/jarde-java/src/{guard.rs,build.rs}`；回归 `tests/p3_twr_enclosing_catch.rs`。

## 负例与守卫的映射（tasks 1.2）

六形状 → 每个钉死一个守卫（`original/N17b.class`，JDK 23 Class-File API 手工下级化，[N17bGen.java](N17bGen.java) 可复现；TWR 行对/关闭/抑制/重抛与 javac 23 `--release 8` 布局同构，全跨度行为 `IllegalStateException` 具名行 `[0,34)→37`）。六类本体全部 `java -Xverify:all` 运行干净（`done`×6，测试钉死）——被拒的是恢复而非运行：

| # | 形状 | 落空的守卫 | 实现前 | 实现后 |
| --- | --- | --- | --- | --- |
| `catchAllSurround` | 全跨度行 type=any（catch-all） | 具名类型（catch-all 是 finally/TWR 自身语义） | `jre_guard_unexplained_row` + `jre_region_uncovered_blocks` | 逐字不变（`results/N17b-*.base.json` vs `results/N17b-*.after.json`） |
| `partialRow` | 具名行 `[0,11)` 覆盖 init+体、cleanup 前止 | 全跨度（原排除子句不动，分片分支照旧） | `jre_region_uncovered_blocks` | 逐字不变 |
| `overlapHandler` | 具名行 `[8,11)→18`，handler 是 claim 自身主 handler | handler 块在 claim 外 | `jre_guard_unexplained_row` + uncovered | 逐字不变 |
| `branchingHandler` | handler 体 `getstatic; ifne; areturn; areturn`（两出口） | 单直行块（语句白名单 + 唯一后继） | `jre_guard_unexplained_row` + uncovered | 逐字不变 |
| `doubleCatch` | `[0,34)→37 ISE` + `[0,34)→41 RE`（两个 handler） | 至多一行全跨度行 | `jre_guard_unexplained_row` + uncovered | 逐字不变 |
| `multiCatch` | `[0,34)→37 ISE` + `[0,34)→37 RE`（同 handler 两类） | 至多一行全跨度行（multi-catch 另片） | `jre_guard_unexplained_row` + uncovered | 逐字不变 |

## 基线重放与固定形状（tasks 1.1/2.1/2.2）

fixture SHA 见 [results/fixture-sha256.txt](results/fixture-sha256.txt)；巡查冻结 T1/T3 与 CF-15 冻结 C4 的 SHA 复核一致。主线基线：T3.voidNamed/voidNamedRecover、T1.twrVoidNamed/twrPopNamed、C4.twrNamed 整方法回退（巡查矩阵与 `jre_guard_unexplained_row`@0 一致；本目录 `results/W17b-*.base.json` 为正例族同一记录）。实现后五形全部恢复：

| 形状 | 实现后呈现 |
| --- | --- |
| T3.voidNamed | `try (T3 local0 = new T3()) { touch(local0); } catch (java.lang.IllegalStateException local0) { return "caught"; }` + `return "done";` |
| T3.voidNamedRecover | 同上，handler 体为调用语句 `touch((T3) null);`（17a 体形状叠加） |
| T1.twrVoidNamed | `try (T1 local0 = new T1()) { local0.hashCode(); } catch (…) { return "caught"; }` |
| T1.twrPopNamed（17a+17b 叠加） | `try (T1 local0 = new T1()) { local0.toString(); } catch (…) { return "caught"; }` |
| C4.twrNamed（17a+17b 叠加） | `try (C4 local0 = new C4()) { local0.toString(); } catch (…) { return "caught"; }` |

无 catch 的 TWR 家族、Catches 既有路径（catch 包普通 try）、FinallyOnce/SharedFinally 证书、`Guarded` 其余成员逐字不变（全仓测试与 `p5_bulk_corpus` 钉值门禁为证；`withCatch` 为本片唯一翻转为恢复的既有拒绝，`p3_guard`/`p3_execution_comparison`/`p5_bulk_corpus` 三处预期随行为更新并注明缘由）。

## 正例族 W17b（行为对照载体）

[W17b.java](W17b.java) → 冻结 [original/W17b.class](original/W17b.class)，javac 23.0.1 `--release 8 -g:none`。五个成员覆盖验收矩阵的路径：normalReturn（正常续行）、bodyThrows（体内注入 ISE 走 catch）、handlerCall（handler 体为调用语句）、closeThrows（清理异常传播——close 的 ISE 直接落入具名子句、suppression 空）、suppressedBoth（体与 close 双抛，addSuppressed 链保留）。实现前五成员全部整方法回退，实现后全部恢复（`results/W17b-*.recovered.txt`）。

## 三方对照（tasks 3.2，`results/three-way/`）

原冻结 class / 固定 JADX dev（`jadx-cli/build/install/jadx/bin/jadx`）/ Jarde `class-source` 三腿各自 `javac --release 8` 重编后 `java -Xverify:all`。两腿源码随目录（`*.jarde.java`/`*.jadx.java`）；`run-sha256.txt`/`leg-source-sha256.txt`：

| 类 | 原 class | Jarde | JADX | 备注 |
| --- | --- | --- | --- | --- |
| T3（两形状经 main） | ✓ | 逐行一致 | 逐行一致 | `done/done` |
| T1（twrVoidNamed/twrPopNamed 在内） | ✓ | 逐行一致 | 逐行一致 | `done`×4 |
| C4（twrNamed/constructNamed） | ✓ | 逐行一致 | 逐行一致 | `done/a` |
| W17b（五路径） | ✓ | 逐行一致 | `closeThrows` 一处偏差 | JADX 腿把正常 close 移入保护区并虚构 suppression（`caught:close:[…close]`，close 被运行两次）——参照列自身 TWR 重构缺陷，与 17a 记录同类；原类为行为基准，jarde 列与原类逐路径一致 |

编译腿无需补丁：四类恢复文本 `javac --release 8` 一次通过（catch 参数与资源同槽的 `catch (E local0)` 与资源声明作用域不相交，源法律允许）。N17b 负例类本体 verifier 有效可运行，被拒的是恢复而非运行。

## 已知边界（如实记录）

- handler 体限于单直行块的语句子集（本片验收域）；分支/循环/throw 体按守卫保持 `jre_guard_unexplained_row` 拒绝，扩展随 `Catches` 路径另片。
- handler 体把子句参数传给形参声明收紧的静态方法（如 `helper(e)`）时，参数类型证据沿用槽复用前的变量判定（javac 让 catch 参数复用资源槽）而拒绝呈现——普通 catch（参数槽无复用）无此现象（`OC`/`OC2` 探针：`tag(local1)` 与 `Arrays.toString(local1.getSuppressed())` 均呈现）。内联链式消费（`e.getMessage()`、`e.getSuppressed()`）不受影响，W17b 三方对照即按此形态。类型决策按使用位置的归属属 `Catches` 扩展片域。

## 测试与门禁

- `tests/p3_twr_enclosing_catch.rs`：五固定形状命中文本、子句头拼写与 handler 入口锚（BCI 38）、N17b 六负例（quality=fallback + 既有诊断逐字）、N17b 本体运行、三腿编译运行（T3/T1/C4/W17b 全路径逐行）、预算/取消零发布。
- `tests/p3_guard.rs`：`withCatch` 由拒绝钉死改为恢复钉死（全构造 catch 即语句自身子句，handler 入口 BCI 43 为锚）。
- `tests/p3_execution_comparison.rs`：`withCatch` 预期 `Quoted(jre_guard_unexplained_row)` → `Executed`（编译+运行对照）。
- `tests/p5_bulk_corpus.rs`：`explanation_only` 5→4；`MANY_METHOD_CLASS`/`DIRECT_ARM`/`SHARED_ARM` 三钉值按 `record_the_billing_table` 重录（+50 IrItems/+34 AnalysisSteps/−51 output_bytes，唯一翻转成员 `Guarded.withCatch`）。
