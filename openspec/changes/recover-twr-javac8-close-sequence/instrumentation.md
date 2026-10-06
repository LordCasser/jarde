# 插桩转录与选向（task 1.1）

本文件是 task 1.1 的交付：用前任 agent 留下的 env 门控插桩（`crates/jarde-java/src/guard.rs` /
`region.rs`，`JARDE_TWR_TRACE=1` 逐判定点转录、`JARDE_TWR_DUMP=1` 打印 examination 时的完整块图与
异常表）在两腿上渲染巡查 fixture，回答 Q-i（第一道门）、Q-ii（A/B 选向）、Q-iii（呈现路径），
并记录选向理由。**插桩已在本片 2.3 全部移除（grep 零残留）**；本文件保留转录与结论。

## 0. 复现方法（转录口径）

```text
cargo build -p jarde-cli --locked
FIX=openspec/evidence/java-syntax-2026-10-04/twr-javac8-codegen-patrol/fixture
JARDE_TWR_TRACE=1 JARDE_TWR_DUMP=1 ./target/debug/jarde-cli class-source \
  --input $FIX/<real-javac8-TR|javac23-TR>/TR.class --policy single-class --class TR --format text
```

两腿完整转录（stderr+stdout 合并，含 dump）：

| 腿 | 转录 | 行数 |
| --- | --- | --- |
| 真 javac 8（Corretto 1.8.0_432） | [results/twr-trace-real-javac8.txt](../../evidence/java-syntax-2026-10-04/twr-javac8-codegen-patrol/results/twr-trace-real-javac8.txt) | 2302 |
| javac 23 `--release 8`（对照） | [results/twr-trace-javac23.txt](../../evidence/java-syntax-2026-10-04/twr-javac8-codegen-patrol/results/twr-trace-javac23.txt) | 3327 |

同一批插桩二进制对 javac 23 腿的渲染与 [冻结基线](../../evidence/java-syntax-2026-10-04/twr-javac8-codegen-patrol/results/TR-javac23-rendered.txt)
逐字节相同（差异只有 `elapsed_millis` 计时行），故插桩本身未改变两条判定路径。

## 1. Q-i：第一道门在哪判定、哪个 BCI

### 1.1 事实：region 层先问 guard，guard 答 `NotGuarded`，region 层再以异常边拒绝

真 javac 8 `one()` 的入口块（BCI 0）上，region 走查与 guard 的先后序由插桩直接给出：

```text
[twr-trace] resources: candidate row=1 range=(11,16) target=48 type=Some(15) closes=false start=0
[twr-trace] resources: row=1 before=10 op=Some(Store { slot: 2 })
[twr-trace] resources: candidate row=2 range=(11,16) target=53 type=None closes=true start=0
[twr-trace] resources: row=2 before=10 op=Some(Store { slot: 2 })
[dump] examine bci=0 blocks=13
[dump] row 0 range=(24,28) target=31 type=Some(15)
[dump] row 1 range=(11,16) target=48 type=Some(15)
[dump] row 2 range=(11,16) target=53 type=None
[dump] row 3 range=(63,67) target=70 type=Some(15)
[dump] row 4 range=(48,55) target=53 type=None
[dump] block 0 bcis=[0, 3, 4, 5, 8, 9, 10, 11, 12, 15, 16, 17] succ=["Normal->20", "Normal->46"] exc=[(1, 48), (2, 53)] takeable=[1, 2]
[twr-trace] resources: candidate row=1 range=(11,16) target=48 type=Some(15) closes=false start=0
[twr-trace] resources: row=1 before=10 op=Some(Store { slot: 2 })
[twr-trace] resources: candidate row=2 range=(11,16) target=53 type=None closes=true start=0
[twr-trace] resources: row=2 before=10 op=Some(Store { slot: 2 })
[region] bci=0 leaving=Some("jre_region_exception_edge") examine -> NotGuarded
```

序列读出三件事：

1. **region 层确实调用了 `guard::examine`**（`region.rs` 的 `leaving_edge` 非空即调用），
   `[region]` 行在其返回后打印。故 guard 的 `fn guarded` 链**被到达**，不是"guard 之前就拒"。
2. **guard 的答案是 `Verdict::NotGuarded`**，不是 `Refused`。`jre_region_exception_edge` 是
   region 层在 guard 说"这里没有我的形"之后，于 `region.rs`（`if let Some(reason) = leaving` 分支）
   写下的**自己的**理由：`FallbackReason::ExceptionEdge { block_bci: 0, handler_ordinal: 1 }`。
3. **拒绝的落点是 BCI 0（第一道门所在的判定）与 BCI 48**：`one()` 的块 0 与块 48 各有一条
   未被证明吸收的异常边（`exc=[(1,48),(2,53)]` / `exc=[(4,53)]`），两次 examination 都答
   `NotGuarded`（转录第 18、21 行）。其余 3 条 `jre_region_exception_edge` 来自 `two()` 的两个块，
   4 条 `jre_region_uncovered_blocks` 来自同一批未被 claim 的 handler 块（20/24/31/42/46…）。

### 1.2 事实：guard 内部的第一个失败判定（精确到 BCI 与谓词）

`fn resources` 的候选循环（`guard.rs:14104-14290`）对 `one()` 的两个候选行都走到 `continue`，
**根本没有进入 `fn twr`**（转录里没有任何 `twr:` 行）：

```text
[twr-trace] resources: candidate row=1 range=(11,16) target=48 type=Some(15) closes=false start=0
[twr-trace] resources: row=1 before=10 op=Some(Store { slot: 2 })
[twr-trace] resources: row=1 skip: completed local store (normal_close=None boundary=true closes=false)
[twr-trace] resources: candidate row=2 range=(11,16) target=53 type=None closes=true start=0
[twr-trace] resources: row=2 before=10 op=Some(Store { slot: 2 })
[twr-trace] outline: row=2 slot=2 end=16 normal=None handler=Some(Some((CloseTarget, 55)))
[twr-trace] resources: row=2 skip: null literal, outline fails (slot=2 normal_close=None)
```

| 候选行 | 谓词 | BCI | 结论 |
| --- | --- | --- | --- |
| row 1（`[11,16)→48`，`Throwable`，`closes=false`） | "已完成的局部 store"跳过（`guard.rs:14174-14186`：`single_statement` + `normal_close(row.end=16, slot 2) == None` + `!closes_something(row)` + `statement_boundary`） | 判定读取的 store 在 **BCI 10** | 该行被读成 `catch` 的条款体，跳过 |
| row 2（`[11,16)→53`，`any`，`closes=true`） | 空字面量资源准入（`guard.rs:14225-14233` → `null_resource_close_outline`） | `initialisation(11, 0)` 返回 `((9,11), 2)`；`normal_close(row.end=16, slot 2) == None` | 读出"`aconst_null; astore_2` 是资源初始化"，其 close 轮廓不成立 → 跳过 |

两个候选都用尽后 `header == false && deferred_finally == None` → `Ok(Verdict::NotGuarded)`
（`guard.rs:14289-14297`）。

**根因（插桩确证巡查第三节的读码假设，并给出确切 BCI）**：`fn initialisation`（`guard.rs:1522`）
读的是**保护区起点前一条指令**，在真 javac 8 形上是 BCI **10 的 `astore_2`——primary 空槽引导**，
真正的资源 store 在 BCI **8 的 `astore_1`**。故现有判据认错了"资源初始化"这一格，随后
`normal_close(at=16, slot=2)`（应为 slot 1）也不成立。这与巡查的假设一致，但**位置更前**：
不是 `initialisation` 返回后失败，而是它先把引导读成了资源、再由既有"跳过读成 catch"的
两条保守规则把两个候选都跳掉。

### 1.3 Q-i 结论

- 第一道门（用户可见拒绝码 `jre_region_exception_edge`，6 条）**由 region 层发出**，但它的**前提**
  是 guard 在 BCI 0 / BCI 48 上返回 `NotGuarded`；`jre_region_uncovered_blocks`（4 条）是同一批
  未被 claim 的块在 region 层被引用的结果。
- guard 内部的第一个失败判定是 `fn resources` 候选循环的两条保守跳过规则，**不是** `fn twr` 的
  任何内部断言（`fn twr` 在真 javac 8 单资源形上从未被调用）。
- 落点结论：本片必须在 **guard 层**（`fn guarded` 的 `resources` 链）让形被 claim；region 层
  无需先放行——它在拿到 `Claimed` 时走 `Region::Guard`（见 1.4），拿到 `NotGuarded` 时才拒绝。

### 1.4 对照：javac 23 腿同一条路径（`Claimed` 才让 region 层放行）

```text
[twr-trace] twr: chain [(0, 9, 14, 20, Some(48))]
[twr-trace] twr: level row=0 init=(0, 9) slot=1
[twr-trace] twr: handler row=0 primary=20 close=22 guard=(21, 25) span=(20, 36) suppression=29 call=31 rethrow=35
[twr-trace] twr: body=(9, 14) inner=false
[twr-trace] twr: normal close slot=1 close_at=15 next=18
[twr-trace] twr: return_tail=Some((18, 19, 13)) at=18
[twr-trace] twr: explained ok pieces=[(14, 18), (0, 9), (9, 14), (20, 36), (21, 25), (18, 20)]
[twr-trace] twr: cleanup_blocks ok owned=[0, 20, 28, 34] cleanup_bcis={14, 15, 20, 21, 22, 25, 28, 29, 30, 31, 34, 35} rows=[0, 1]
[twr-trace] resources: row=0 twr claimed
[region] bci=0 leaving=Some("jre_region_exception_edge") examine -> Claimed owned=[0, 20, 28, 34] lead=(0, 0) body=(9, 14) join=None
```

即：**同一道 region 门，`Claimed` 放行、`NotGuarded` 拒绝**；`jre_region_exception_edge` 只是
"没人认领这条边"的记账名，不是独立的判定点。

## 2. Q-ii：A/B 选向（现有六项机制逐条对照）

`Shape::Resources` 的证明链（`resources` → `twr`）逐项在真 javac 8 单资源形（`one()`）上核对。
真 javac 8 `one()` 的规范块图（`javap -c` + `[dump]`，两腿一致口径）：

```text
 0: new TR; 3: dup; 4: aload_0; 5: <init>; 8: astore_1        资源初始化（块 0 前段）
 9: aconst_null; 10: astore_2                                 primary 空槽引导
11: aload_1; 12: use; 15: astore_3                            保护区 [11,16)（body）
16: aload_1; 17: ifnull 46        ┐ 主 close 第一道守卫（资源）
20: aload_2; 21: ifnull 42        │ 主 close 第二道守卫（primary 槽）
24: aload_1; 25: close; 28: goto 46 │ 有抑制臂（被 row 0 [24,28)→31 保护）
42: aload_1; 43: close             │ 无抑制臂（primary == null）
46: aload_3; 47: areturn          ┘ 返回尾（块 46）
48: astore_3; 49: aload_3; 50: astore_2; 51: aload_3; 52: athrow   重抛回路（row 1 的 handler）
53: astore 5; 55: aload_1; 56: ifnull 85     ┐ 抑制链 handler（row 2 / row 4 的目标）
59: aload_2; 60: ifnull 81                    │ 双守卫重演
63: aload_1; 64: close; 67: goto 85           │ 有抑制臂（被 row 3 [63,67)→70 保护）
70: astore 6; 72: aload_2; 73: aload 6; 75: addSuppressed; 78: goto 85
81: aload_1; 82: close                        │ 无抑制臂
85: aload 5; 87: athrow                      ┘ 重抛（块 85）
Exception table: [24,28)→31 T; [11,16)→48 T; [11,16)→53 any; [63,67)→70 T; [48,55)→53 any
```

| # | 现有机制（真 javac 8 形上是否成立） | 判定 |
| --- | --- | --- |
| 1 | **`initialisation`**（`guard.rs:1522`）：资源 store = 保护区起点前的指令，且是"一条语句"。javac 8 形上起点前是 BCI 10 的**空槽引导**，真 store 在 BCI 8。 | **不成立**。须新增"引导+真 store"读法（引导是独立语句，资源语句在它之前结束）。 |
| 2 | **`close_handler` / `close_of_level`**（`2079`/`2156`）：handler 头 `Store primary; Load r; [ifnull L]`，抑制接收者 == primary 槽，重抛槽 == primary 槽。javac 8 形的抑制链 handler（块 53）在资源守卫之后还有一道 `Load P; ifnull L2`（块 59），抑制接收者是引导槽 2、重抛槽是 handler 自身绑定槽 5。 | **不成立**。须新增双守卫 close 组读法，并新增"抑制接收者是引导槽（= primary 的载槽）"事实。 |
| 3 | **`normal_close`**（`15188`）：`aload r; ifnull L; aload r; close`（守卫形）或无守卫形。javac 8 形在 `ifnull L` 之后是 `aload P; ifnull L2`，close 在更深一层。 | **不成立**。须新增双守卫读法（两臂都 close、都回到 L）。 |
| 4 | **资源链几何**（`14355-14390`）：按行嵌套 + `closes_something` 由内向外取链。javac 8 形上：`any` 行（row 2）`closes=true` 可作层级行，同范围的 `Throwable` 行（row 1，`closes=false`）**不在链中**。 | **部分成立**。链本身可用；但 row 1/row 4（relay 与其再捕获行）必须作为**新增行事实**被证明并纳入 `rows`。 |
| 5 | **`unexplained-row` 检查**（`14806-14833`）：覆盖语句跨度的行必须全在 `rows` 内或被 enclosing clause 证明。javac 8 `one()` 有 5 行，现有构造只产出 2（层级行 + close guard 行）。 | **不成立（作为现状）**，但它正是"新增行事实必须齐备"的强制门：rows 需扩到 5（层级行、relay 行、再捕获行、异常 close 守卫行、正常 close 守卫行）。 |
| 6 | **`cleanup_blocks`**（`1019`）：语句跨度外的块必须整块在 cleanup 跨度内、只能由 `rows` 中的行或候选块进入/离开。javac 8 形多出块 48/53/59/63/70/81/85。 | **成立（作为机制）**，无需改动：只要 4 补齐 `rows`、并把 relay 跨度加入 cleanup 跨度，既有检查逐条通过（块图逐边核对见 2.2）。 |

另外两项**新几何事实**（不在六项里，但缺了就不健全）：

7. **引导槽的写者唯一性**：引导槽（slot 2）在整个 statements 的 owned 块内只能被"引导对"与
   "relay 的存入"写。否则主路径第二道守卫会走**有抑制臂**（块 24-31），而该臂会把 close 自身
   的异常 `addSuppressed` 后 `goto 46` **吞掉**、方法照常返回——这是 javac 8 lowering 里
   依赖"主路径上 primary 必为 null"才成立的分支。插桩实测该臂存在（`[dump] block 24 … exc=[(0,31)]`）。
8. **relay 行与层级行的表内相邻性**：relay 行（有 catch 类型）必须在层级行（`any`）**前一条**且
   覆盖同一保护区。JVM 的"首个匹配行胜出"是重抛回路语义的一部分。

### 2.2 块图逐边核对（方向 A 是否只需"加严"）

用 `[dump]` 的块图逐边核对 `cleanup_blocks` 的两条检查（进入边 ∈ rows / 候选块，离开边 ∈ 候选块 /
rows）：

| 块 | 进入边 | 离开边 | 判定 |
| --- | --- | --- | --- |
| 48（relay） | `exc[(1,48)]` | `exc[(4,53)]` | 需 row 1、row 4 ∈ rows |
| 53（抑制链 handler） | `exc[(2,53)]`、`exc[(4,53)]` | `Normal->59`、`Normal->85` | 需 row 2 ∈ rows；59/85 为候选块 |
| 59（第二道守卫） | `Normal<-53` | `Normal->63`、`Normal->81` | 候选↔候选 |
| 63（有抑制 close） | `Normal<-59` | `Normal->85`、`exc[(3,70)]` | 需 row 3 ∈ rows |
| 70（addSuppressed） | `exc[(3,70)]` | `Normal->85` | 需 row 3 ∈ rows |
| 81（无抑制 close） | `Normal<-59` | `Normal->85` | 候选↔候选 |
| 85（重抛） | `Normal<-63/70/81` | 无 | 候选↔候选 |
| 31（主路径 addSuppressed） | `exc[(0,31)]` | `Normal->46` | 31/46 都在主跨度 base 内（**不是**候选块），故 `cleanup_blocks` 不管它 |

结论：六项机制里 1/2/3/5 需要**新增**判据（每一条都以"新增事实"表达，javac 9+ 形上恒假：
javac 9+ 既无 `aconst_null` 引导、无双守卫、无 relay 行），4/6 是既有机制在**扩充后的行集**
下继续工作。**没有任何一项需要放宽**：既有的 `initialisation`/`close_of_level`/`normal_close`
判据可以逐字不动，新形由**并列的新读法**（新函数）承接，只在 `fn resources` 的候选循环里
增加一条"新读法成功则 claim、失败则按原路径继续"的分派。故 **方向 A**（扩展既有
`Shape::Resources`，承载方式 = 新证明函数产出同一个 `Shape::Resources` plan）。

**方向 B（新 shape 变体）被否的证据**：`Shape::Resources` 是变长且由 `Region::Guard` 统一呈现
（`build.rs:14386` 只读 `resources/returns/inner_finally/trailing_finally/trail`），新变体家族会
把同一份呈现路径复制一份，并迫使 `build.rs`/`region.rs` 增加分支（design 明确要求不触碰
`emit.rs`、呈现形不变）。而 2.2 的逐边核对证明既有几何**不需要**新变体——只是需要新的**事实**。

### 2.3 Q-ii 结论

**方向 A**。理由链：① 六项机制中需要新增的 1/2/3/5 全部可以表达为"javac 9+ 形上恒假"的结构
事实（引导对、双守卫 close 组、relay 行对、正常路径 close 守卫行），不涉及任何既有判据的放宽；
② 4/6 无需改动；③ 7/8 是新增的健全性事实，把主路径"有抑制臂吞异常"与 relay 语义钉死；
④ 承载方式选"新证明函数 + 同一个 `Shape::Resources` plan"，`emit.rs`/呈现路径零改动。

## 3. Q-iii：呈现路径无需改动

真 javac 8 与 javac 23 的可呈现性所需事实相同：`Resource{slot, init, close_bci,
exceptional_close_bci}` + `returns` + `cleanup`。`build.rs` 的 `resource_declaration`
只读 `resource.init()` 的**最后一条指令**（= 资源 store）与其栈操作数渲染头表达式，
`plan.lead()` 与 `plan.body()` 决定花括号外的引导区/括号内的正文，`returns` 决定
`return` 尾。故只要证明给出 `init = (0, 9)`（真 store 在 BCI 8）、`body = (11, 16)`、
`returns = Some(47)`，呈现即写出 `try (TR local1 = new TR(arg0)) { … }`，
BCI 9/10 的引导对不进任何文本（它们只在 `cleanup`/`facts` 里作证据）。javac 8 的
`addSuppressed`/守卫/relay 全部落在 `cleanup` 与 `facts`，源级不出现——与 javac 9+ 腿同构。

> 呈现文本的**唯一**可预期差异是正文局部变量的合成名：javac 8 把 `use()` 的返回值存在
> slot 3（slot 2 被引导占用），javac 23 存在 slot 2，而合成名是 `local<slot>`（`names.rs:744`）。
> 头部的资源名两腿相同（slot 1 → `local1`）。这是两份 class 的真实局部变量分配差异，
> 不是呈现路径的差异。

## 3.5 实现期追加的插桩发现（post-fix 转录，插桩移除前捕获）

实现方向 A 后，同一批插桩又暴露了一条**决定实现形态的事实**：region 走查在把块交给
guard 之前，会先用 **`Sites::empty()`** 的 guard 副本问一次 `guard::catches`（region.rs 的
`try_region` → `guard::catches` → `guarded`），并且只有在 guard 未 claim 时才读出"用户 catch"。
真 javac 8 的入口块与 javac 9+ 不同：其保护区间起点（BCI 11）落在 canonical 块**内部**（不是块边界），
故 `starts_catch(block 0)` 为真，这次预读**一定会发生**：

```text
[twr-trace] resources: candidate row=2 ... start=0
[twr-trace] resources: row=2 twr_lead failed Span at 0      ← Sites::empty() 的预读（catches）
[twr-trace] resources: candidate row=2 ... start=0
[twr-trace] resources: row=2 twr_lead claimed               ← 真正的 examination（带真 Sites）
[region] bci=0 leaving=Some("jre_region_exception_edge") examine -> Claimed owned=[0, 20, 24, 31, 42, 46, 48, 53, 59, 63, 70, 81, 85] lead=(0, 0) body=(11, 16) join=None
[region] guard prefix=[] owned=[0, 20, 24, 31, 42, 46, 48, 53, 59, 63, 70, 81, 85] join=None
[region] tree guard blocks=[0, 20, 24, 31, 42, 46, 48, 53, 59, 63, 70, 81, 85]
```

**结论（实现含义）**：`twr_lead` **不能**因为"Sites 为空导致资源初始化读不出"而降级为
`NotGuarded`（那样 `catches` 会把该块读成用户 `try/catch`，clause 引用 relay 块 48，与 Guard 的
ownership 重叠 → 整方法 `jre_region_ownership_overlap` 拒绝）。实现因此把"该块是本 lowering 的
层级"这一判断拆成**两个不需要 `new@1` plan 的事实**（空槽引导 + 重抛 relay），一旦成立，后续任何
链接失败都返回 **`Verdict::Refused`**（归到 TWR rule 名下）——这正是 `catches` 早已写下的原则：
"a `try`-with-resources this build cannot prove is never spelled as a user `catch`"。
空槽引导与 relay **都不成立**时才是 `NotGuarded`，其他读法（既有的 `catch`/`finally` 路径）
逐字不变。

两腿 post-fix 完整转录：[results/twr-trace-post-fix-real-javac8.txt](../../evidence/java-syntax-2026-10-04/twr-javac8-codegen-patrol/results/twr-trace-post-fix-real-javac8.txt)、
[results/twr-trace-post-fix-javac23.txt](../../evidence/java-syntax-2026-10-04/twr-javac8-codegen-patrol/results/twr-trace-post-fix-javac23.txt)。

## 4. 与负例的关系（task 1.3 的冻结依据）

三条削弱探针（各削一条几何事实）在**现状**（插桩后、实现前）都必须仍被拒绝：

| 探针 | 削弱的事实 | 现状下的拒绝路径（转录依据） |
| --- | --- | --- |
| 削一条 `any` 行（去掉 row 2 或 row 4） | 重抛回路/再捕获 | row 2 缺失 → 块 53 的进入边 `exc[(2,53)]` 成为"未在 rows 的行" → 无法 claim |
| `ifnull` 目标互换 | 双守卫顺序 | 第二道守卫的 `ifnull` 目标不再指向无抑制 close → 新读法的 close 组两臂校验失败 |
| 抑制链断头（去掉 48→53 的 `any` 行，或改 53 的重抛槽） | 抑制/重抛链 | 块 48 的离开边 `exc[(4,53)]` 无行可归 → relay 证明失败 |

三条在实现后仍须保持拒绝（task 3.3）。

## 5. 残余边界（交 root 复核时一并核对）

1. **catch 类型不可读**：`ExceptionTable` 的 `catch_type_index` 在本层是不透明的池下标（guard 不解析类型名，
   与模块自述的 "`AutoCloseable` 是 resolution 事实、本层不读" 同一边界）。故 relay 行与层级行按**位置**
   （表内相邻、同保护区、先匹配先胜）读取，而不是按类型名重推：对"首个覆盖保护区的行"会拦住 body 的
   抛出这一点，是几何事实，不是类型事实。实际 javac 8 产物的该行恒为 `java/lang/Throwable`，
   该读法对真实产物完备；对**手工构造**的、把首行类型收窄到比被抛异常更窄的 class，本读法仍会接受
   （其语义与源码 TWR 在"body 抛出 <close 也抛出"时可能不同）。这是本片已知的最窄残余，登记在此供
   root 判定是否需要后续加严（需要类型解析，属跨层事实）。
2. **双资源仍拒（design Non-Goal）**：`TR.two()` 保持 `not recovered`。因此巡查 fixture `TR.class` **整类**
   在"只剥注释"后不能编译（`two()` 无正文）；验收改以两种等价口径给出：① 剥注释 + 去掉被拒的 `two()`
   及其调用 → `javac --release 8` exit 0 且 `one()` 路径输出与原 class 逐行一致；
   ② 单资源锚 `TROne`（同一 `one()` 形）双腿整类往返（渲染 → `javac --release 8` → `java -Xverify:all`
   逐行一致）。证据：`results/roundtrip-single-resource/`。
3. **全语料差分的覆盖口径**：2678 条比对里 700 条因 `--class <文件干名>` 无法解析而与两侧同为错误输出，
   有效渲染覆盖 1978 条（含全部 `p3-*`/`finally`/TWR 族）。见 `results/corpus-render-differential.md`。
