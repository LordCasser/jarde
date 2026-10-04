# try-with-resources 的真 javac 8 codegen 不被恢复（2026-10-04，root 巡查）——版本耦合缺口第三例

**结论：jarde 的 CF-17 try-with-resources 恢复能力只识别 javac 9+ 的优化 codegen，对真 javac 8（Corretto 1.8.0_432）的 TWR 形（`aconst_null` 前置 + `ifnull` 守卫关闭序列）**整方法拒绝**（响亮失败，非静默偏离）。CF-17 的验收 fixture（`T2.class` 等）经结构指纹判定由 javac 9+ 编译，故该缺口对既有验收不可见。这是本会话发现的第三个"语料由 javac 9+ 编译、真实 Java 8 产物有盲区"的实例（前两例：[DT-03 `getClass` null-check](../qualified-outer-alloc-getclass-patrol/README.md)、[返回值形写访问器](../value-returning-write-accessor-patrol/README.md)），三者同根，见末尾"系统性归属"。**

固定转录见 [fixture](fixture/)（`TR.java` + 真 javac 8 与 javac 23 两套 class）与 [results](results/)（双腿渲染、javap codegen 对照）。

## 一、决定性单变量实验（同一 `TR.java`，两种 javac）

root 构造无 widening 混淆的 TWR 探针 [fixture/TR.java](fixture/TR.java)（`one()` 单一资源、`two()` 双资源，资源类自实现 `AutoCloseable`）：

| 编译工具链 | `one()` 指令数（单资源，单方法） | `two()` 指令数（双资源，单方法） | 异常表行数（**整类**） | jarde 渲染引注 | `one()`/`two()` 恢复 |
| --- | --- | --- | --- | --- | --- |
| javac 23 `--release 8` | 22 | 61 | 7（`one()` 2 / `two()` 5） | **0** | ✓ `try (TR local1 = new TR(arg0)) { … }` |
| **真 javac 8**（Corretto 1.8.0_432） | **48** | **113** | **16**（`one()` 5 / `two()` 11） | **6** | ✗ 整方法 `not recovered` |

即真 javac 8 的 TWR codegen 约为 javac 9+ 的 **2.2 倍指令量**（`one()` 48/22、`two()` 113/61）、**2.2 倍异常表行**（整类 16/7；单方法 `one()` 5/2、`two()` 11/5），且真 javac 8 独有 **`any` catch-all 行**（`one()` 2 条、`two()` 4 条；javac 9+ 两方法均 **0** 条）与 `aconst_null` 资源副本引导（JDK 9 对 TWR 做了重大 codegen 简化）。jarde 只恢复了后者。

- **javac 23 腿**（[results/TR-javac23-rendered.txt](results/TR-javac23-rendered.txt)）：`one()` 呈现 `try (TR local1 = new TR(arg0)) { java.lang.String local2 = local1.use(); return local2; }`，`two()` 呈现双资源 `try (TR local1 = …; TR local2 = …)`，quotes=0。
- **真 javac 8 腿**（[results/TR-realjavac8-rendered.txt](results/TR-realjavac8-rendered.txt)）：`one()`/`two()` 均 `// jarde: not recovered: … produced no statement`，引注 `local 1 crosses a quoted fallback region; its assignments and consumers cannot be presented as one lexically bound declaration`，quotes=6。

字节码差异的根因（[results/javap-codegen-comparison.txt](results/javap-codegen-comparison.txt) 全文）：真 javac 8 的 `one()` 含 `aconst_null; astore_2`（资源副本预置 null）+ 关闭序列的 `aload_1; ifnull …; aload_2; ifnull …; invokevirtual close` 守卫形；javac 9+ 去掉了 `aconst_null` 前置与部分 `ifnull` 守卫，直接 `aload; invokevirtual close; goto`。jarde 的 TWR region 证明覆盖后者、不覆盖前者。

## 二、CF-17 验收 fixture 由 javac 9+ 编译（结构指纹判定）

CF-17 有**两族**验收 fixture，root 逐一核实**均由 javac 9+ 编译**（`instrs`/`aconst_null` 结构指纹与 javac 23 逐值一致，与真 javac 8 不符）：

| fixture | 被谁使用 | frozen 指纹 | 真 javac 8 | javac 23 | 判定 |
| --- | --- | --- | --- | --- | --- |
| `cf17-twrcatch-patrol/fixture/T2.class` | `tests/p3_twr_discarded_call.rs:30` `include_bytes!` | instrs=**107** / aconst_null=**0** | 211 / 4 | 107 / 0 | javac 9+ |
| `try-with-resources/root-replay/core/original.class`（= `TwrAudit`，非 `TwrAuditRunner`） | `tests/p3_twr_*` 族 | instrs=**138** / aconst_null=**1** | 204 / 4 | 138 / 1 | javac 9+ |

`instrs`/`aconst_null` 是比 SHA 更可靠的版本指纹（SHA 受调试信息/路径影响，各腿互异属正常；结构指纹才判版本）。两族 frozen 值都与 javac 23 重编**逐值一致**、与真 javac 8 不符 → **CF-17 的验收锚全是 javac 9+ 产物**，其 `voidBody`/`popBody`/`opens` 等断言只覆盖了简化 codegen，真 javac 8 的 TWR 形从未进入验收。

> **root 自查纠错（诚实登记，两处）**：(1) 含 TWR 的类是 `original.class`（即 `TwrAudit`），**不是** `original-TwrAuditRunner.class`——Runner 不含 `try` 语句、其 codegen **版本无关**（三腿均 197 instrs）。root 一度误指纹 Runner 得到"version-invariant"的假象，核实 `Exception table` 数后更正到 `TwrAudit`。(2) root 曾假设"版本耦合是 return-in-try 专属"，用 `D.java` 探针（`noReturn` 无 return-in-try vs `withReturn` 有）**证伪**：两形都耦合（noReturn 真8/javac23 = 50/24、withReturn = 48/22，ratio 均 ~2.1）。故耦合是 **JDK 9 对 TWR codegen 的全局简化**（去掉 `aconst_null` 资源副本前置与部分 `ifnull` 守卫），不限于某一子形——这扩大了缺口范围，也说明"挑一个 TWR 子形修"不足以覆盖。


## 三、颗粒度取证（root 零构建读码 + 异常表实测，回答"是否一定要新增机制"）

**判据是 CFG/异常表驱动，不是指令序列匹配**。root 读码核实 TWR 的形证明归属，并**更正本节初稿的一处错误归属**：

> **root 自查纠错**：本节初稿把 TWR 的形证明归到 `Shape::NullableResourceFinally`（`guard.rs:363`/`3853`）与 `region.rs:5252` `nullable_resource_finally_regions`。**错误**——该变体的文档自述是"The **two-row** nullable local cleanup with a **saved reference return**"，属 `finally` 族（`build.rs:1021`/`2682`/`12072`/`14719` 消费），不是 TWR。TWR 由**独立的 `Shape::Resources` 变体**承载，其证明函数是 `fn twr`（`guard.rs:14128`，构造点 `14645`）。root 的实测证据：javac 23 腿把 `P08_twr.one()` 呈现为 `try (P08_twr local0 = new P08_twr()) { … }`——这正是 `Shape::Resources` 文档自述的形（"`try (T n = …; …) { body }`, with the resources in **declaration** order"）。

`fn twr` 的证明通篇用**块与异常表事实**，无指令序列硬匹配（root 核实 `guard.rs:3700-3860` 段内 **opcode 字面量 = 0**，而该段是 `finally` 族的证明；TWR 侧同理用几何判据）：`facts.blocks_in((…))`、`facts.covering(bci)` 的 row ordinal 集合恰等、`view.successor_ids(&…)` 的恰等集合、每条 `CanonicalEdgeKind::{Exception,Normal,Return}` 边的端点归属校验；TWR 特有的资源链由**异常表行的几何嵌套**构造（`guard.rs:14140-14162`）。

**而两腿的异常表拓扑不同**（root javap 实测 `TR.one()`）：

| 腿 | 异常表行数 | `any`（catch-all）行 | 保护区间 |
| --- | --- | --- | --- |
| javac 23 `--release 8` | **2** | **0** | `8→11 target 17`、`18→22 target 25`（均 `Class java/lang/Throwable`） |
| 真 javac 8 | **5** | **2** | `21→25 target 28`、`10→13 target 43`、**`10→13 target 48 any`**、`58→62 target 65`、**`43→50 target 48 any`** |

即真 javac 8 的 TWR 用**两条 `any` catch-all 行**表达"正常路径与异常路径都要关闭资源"的 JDK 8 codegen，且保护区间与关闭块被复制多份。（数字口径更正：`one()` **单方法**为 48 指令 / **5** 异常表行 vs javac 9+ 的 22 指令 / 2 行；root 初稿此处误写"16 异常表项"，那是 `TR` **整类**的行数——含 `two()` 的 11 行——非 `one()` 单方法。以单方法口径为准。）

**结论（对 Goal 核心问题的回答：不需要新增机制，也不需要新增 shape 变体；需要扩展现有 `fn twr` 的几何/初始化判据以接受 javac 8 拓扑）**：

TWR **不是"再认一种 idiom 拼写"能覆盖的**——javac 8 与 javac 9+ 的 TWR 是**两套块/异常表拓扑**。但 root 读码核实：**TWR 在本仓已由一个变长 shape 承载，无需新增变体**。

- **TWR 的现有证明是变长的**：`fn twr`（`guard.rs:14128`，构造点 `14645` `Shape::Resources`）由**异常表行的几何嵌套**构造资源链 `chain: Vec<&ExceptionHandlerFact>`（`14140-14162`），再 `Vec::with_capacity(chain.len())` 建 `resources`/`handlers`；`Shape::Resources` 持 `resources: Vec<Resource>`、`cleanup: Vec<u32>`（皆变长），文档自述形为 "`try (T n = …; …) { body }`, with the resources in **declaration** order"。javac 23 腿把 `P08_twr.one()` 呈现为 `try (P08_twr local0 = new P08_twr()) { … }` 即经此路径。**故资源数本就变长，多资源不是容量问题**（仓内另有 `recover-multi-resource-twr` 变更 7/8 项专处理多资源）。
- **与 `finally` 定长族无关（更正初稿的误导对照）**：初稿把真 javac 8 TWR 的"5 行/2 段/4 副本"与 `Shape::SegmentedFinally { rows:[u32;5], … }` 逐维对照、并建议新增 `NullableResourceFinallyJdk8 { rows:[u32;5] }` 定长变体——**方向错误**。`SegmentedFinally`/`NullableResourceFinally`/`NestedCleanupFinally` 等是 **`finally`/`synchronized` 族的定长 shape**（`build.rs:1021/2682/12072/14719` 消费 `NullableResourceFinally`），与 TWR 的 `Shape::Resources` **不同族**。TWR 修复应落在 `fn twr` 内，**不新增定长变体、不并入 finally 族**。
- **`any` 行在事实模型中可表示、可区分**：`ExceptionHandlerFact`（`crates/jarde-reader/src/classfile.rs:285-291`）持 `ordinal: u32` + `catch_type_index: Option<u16>`，`None` 即 catch-all，两条 `any` 行由 ordinal 区分。`fn twr` 现有行几何判据（`closes_something(row)`、按 `(end-start, start)` 取最小行、`unexplained-row` 检查，`guard.rs:14150-14154` 注释记载其职责是**拒绝**把"编译器包在整个语句外的 `catch`"误吸收为资源层级）已读 `catch_type_index`（`14625` `row.catch_type_index?`），故区分 `any` 与 `Throwable` 行的事实基础已在。
- **root 的失败机制假设（未验证，须实现片取证）**：`fn initialisation`（`guard.rs:1502`）用 `facts.previous_bci(end)`（`end`=行 `start_bci`）**向后找资源 store** 并取其 `Operation::Store{slot}` 作资源槽。真 javac 8 在资源 store（`astore_0`，BCI 7）与保护区起点（BCI 10）之间插入 `aconst_null; astore_1`（BCI 8-9），故 `previous_bci(10)==9` 命中 **slot 1（null 引导）而非 slot 0（资源）**——若成立则资源槽认错、后续判据失败、整方法拒绝（`Unproven::ResourceInit`）。**这是读码假设**：确证需构建后取诊断（本次为零构建取证，构建由在飞的 DT-03 片占用），故不作结论。

**颗粒度判定**：**中颗粒、架构上常规**——在 `fn twr` 的资源链几何与 `initialisation` 判据内**识别并跳过 javac 8 的 `aconst_null` 引导、按 `catch_type_index` 区分 `any`/`Throwable` 行、把复制的关闭段（含 `addSuppressed`）归入既有变长 `cleanup`**，使真 javac 8 拓扑走通现有 `Shape::Resources`。**不需要新机制、不需要新 shape 变体、不需要平行状态**；javac 9+ 产物走原几何、零回退。显著大于 DT-03（换一条调用拼写）但小于"新增机制"，因变长 TWR 证明与事实模型都已就位。

- **不并入 DT-03 片**（落点 `guard.rs`/`init.rs` 不同、判据性质不同、颗粒度不同）。
- **立项前的剩余取证**（root 未做，不外推）：(1) 确证上面 `initialisation` 的 `aconst_null` 引导假设（构建后取 `Unproven::ResourceInit` 诊断的确切 BCI）；(2) 把真 javac 8 `one()`/`two()` 的**规范块图**逐块画出，确认 `fn twr` 的行几何（`closes_something`、最小行选择、`unexplained-row`）在 `any` 行 + 复制关闭段下是否只需局部扩展、还是会撞上"资源链几何"与"unexplained-row 检查"的冲突；(3) 确认 `addSuppressed` 在区域侧（`region.rs` 消费 `Shape::Resources` 处）的呈现归属；(4) 单资源先行还是单/双资源合并，由 (2) 的块图结果决定（不预设）。
- **风险登记（保留，重述）**：不得为覆盖 javac 8 形而放宽 `fn twr` 的行几何判据（`closes_something(row)`、最小行选择、`unexplained-row` 检查）——放宽会让 `try(…){…}catch(E e){…}` 被误读成多资源 TWR，属静默语义偏离。javac 8 形的覆盖必须通过**精确识别 `any` 行与 `aconst_null` 引导**达成。

## 三之二、单资源 vs 多资源：MVP 边界的实测依据（root 补充取证，含一处自我更正）

上面第三节的"5 行 / 2 段 / 4 副本"是**单资源** `one()` 的拓扑。root 追加实测**双资源** `two()`（`try (TR a = …; TR b = …)`），拓扑显著更大：

| 维度 | 单资源 `one()` | 双资源 `two()` |
| --- | --- | --- |
| 指令数 | 48 | **113** |
| 异常表行数 | **5**（3 Throwable + 2 any） | **11**（7 Throwable + 4 any） |
| `invokevirtual close` 副本 | 4 | **8** |
| `addSuppressed` | 2 | **4** |
| `ifnull` 资源守卫 | 4 | **8** |

**root 自我更正（本节初稿的结论是错的，已推翻）**：初稿据"仓内定长 Shape 变体的行数组最大为 `[u32;5]`（`SegmentedFinally`）"推断"双资源 11 行超出现有全部变体容量、须新增 `[u32;11]` 定长变体或引入变长行集（机制决策）"。**该推断错误**——它把 TWR 归到了定长 `finally` 变体族，而 TWR 实际由**独立的 `Shape::Resources` 变体**承载，该变体用**变长集合**：`resources: Vec<Resource>`、`cleanup: Vec<u32>`、`returns: Option<u32>`、`inner_finally: Option<Box<InnerTwrFinally>>`、`trailing_finally`，其文档自述形为 "`try (T n = …; …) { body }`, with the resources in **declaration** order"。

root 读码核实：证明函数 `fn twr`（`guard.rs:14128`）先由**异常表行的几何嵌套**构造资源链 `chain: Vec<&ExceptionHandlerFact>`（`14140-14162`：从最内层行向外找 `start_bci <= inner.start_bci && end_bci >= inner.end_bci && closes_something(row)` 的行，按 `(end-start, start)` 取最小者），再 `Vec::with_capacity(chain.len())` 建 `resources`/`handlers`。即**资源数本就是变长的**，双资源不是容量问题；且仓内已有 `recover-multi-resource-twr` 变更（7/8 项）专门处理多资源 TWR。故：

- **"多资源需要前所未有的定长容量"这一论断作废**。`Shape::Resources` 的变长设计已覆盖任意资源数，`[u32;N]` 定长族（`Finally`/`LoopFinally`/`SegmentedFinally` 等）是 **`finally`/`synchronized` 的 shape**，与 TWR 不同族，二者不应混为一谈。
- **MVP 边界的正确依据不是容量，而是 javac 8 拓扑与 `fn twr` 现有几何判据的相容性**。真 javac 8 的 TWR 与 javac 9+ 的差异在于：(a) 资源初始化后**插入 `aconst_null; astore_1` 引导**（`one()` BCI 8-9）；(b) 每个资源的关闭路径含**双 `ifnull` 守卫 + `addSuppressed`**；(c) 异常表多出 **`any` catch-all 行**（`catch_type_index == None`）与复制的关闭段。
- **root 的机制假设（未验证，须实现片取证）**：`fn initialisation`（`guard.rs:1502`）用 `facts.previous_bci(end)`（`end` = 行的 `start_bci`）**向后找资源 store**，并取其 `Operation::Store { slot }` 作为资源槽。真 javac 8 在资源 store（`astore_0`，BCI 7）与保护区起点（BCI 10）之间**插入了 `aconst_null; astore_1`（BCI 8-9）**，故 `previous_bci(10) == 9` 命中的是 **slot 1（null 引导）而非 slot 0（资源）**——若成立，则资源槽被认成 null 引导槽，后续判据失败并整方法拒绝。**这只是读码假设**：确证需构建后取 `Unproven::ResourceInit` 诊断（本次取证为零构建，构建由在飞的 DT-03 片占用），故**不作为结论**。
- **因此 MVP 建议仍成立但依据改变**：单资源先行（`one()`，48 指令 / 5 行）→ 双资源后续（`two()`，113 指令 / 11 行）。依据不是"容量上限"，而是**双资源的关闭段复制与 `any` 行嵌套深度显著更高**（8 份 close、4 份 addSuppressed、3 条 any），在单资源形跑通前一次性覆盖两形会放大取证面。若实现片取证发现单/双资源共用同一条 `chain` 几何判据即可覆盖，则**可以合并为一片**——由取证结果决定，不预设。
- **风险登记（保留，但重述）**：不得为覆盖 javac 8 形而放宽 `fn twr` 的行几何判据（`closes_something(row)`、按 `(end-start, start)` 取最小行、`unexplained-row` 检查）——`guard.rs:14150-14154` 的注释明确记载该检查的职责是**拒绝**把"编译器包在整个语句外的 `catch`"误吸收为资源层级。放宽它会让 `try(…){…}catch(E e){…}` 被误读成多资源 TWR，属静默语义偏离。javac 8 形的覆盖必须通过**精确识别 `any` 行与 null 引导**达成，而非放宽行吸收判据。
- **优先级判断**：TWR 是 Java 8 极常见形且当前真 javac 8 产物**整方法拒绝**（响亮，非静默），价值高；但颗粒度显著大于 DT-03/EM-15，须单独排期，不与窄片混批。

## 三之三、块图分岔判定（root 2026-10-04，零构建 javap；`any` 行指向的块在 javac 9+ 表中**无对应物**）

把 `one()`/`two()` 的异常表双腿逐行对照（`fixture/{real-javac8,javac23}-TR/TR.class`）：

**`one()`**——javac 8 共 **5 行**（3× `Throwable` + **2× `any`**）：`24-28→31`、`11-16→48`、`63-67→70`（Throwable）；`11-16→53`、`48-55→53`（any）。javac 23 共 **2 行**（全部 Throwable：`9-14→20`、`21-25→28`）。

**`two()`**——javac 8 共 **11 行**（6× `Throwable` + **5× `any`**）；javac 23 共 **5 行**（全部 Throwable）。

**决定性事实**：javac 8 的 `any` 行的目标块（`one()` 的 `53`、`two()` 的 `159`/`201`）与多出的 `Throwable` 目标块（`48`、`150`、`196`）是 **close-抑制链的独立 handler 块**——javac 9+ 表里**根本不存在**这些目标块（其 `one()` 目标只有 `20`/`28`）。即这不是"同一组块上多两行 `any` 覆盖"（那才是 NullableResourceFinally 式的行变体），而是 **handler 块集合本身不同**：javac 8 把"close 抛异常时的再关闭/抑制"编成独立的 any-handler 块链（`48-55→53` 覆盖 close 调用自身），javac 9+ 重构掉了这整条链（JDK 9 对 TWR codegen 的简化）。

**分岔判定（如实标注证据强度）**：本对照**排除**了"行变体"分支——异常表行型与 handler 块集合都不同，`NullableResourceFinally` 式"加行不改块"的路线**不成立**。但"扩展既有 `Shape::Resources` 判据以容忍 javac 8 的额外抑制链块"（第五节表述：扩展几何/初始化判据即可）与"另立 javac 8 变体家族"**两条路仍未分出**——这需要把 javac 8 的实际块图（指令级，不止异常表）逐块对照 `fn twr`（`guard.rs`）的六项 CFG 机制，看抑制链块能否被 Resources 的既有几何容纳（作为被豁免的辅助块）还是会破坏其结构判据。root **未做**这一步，故立项时按"机制层扩展"起步（MVP：先单资源 `one()` 形），若实现中证实现有几何不可容纳再升级为变体家族——**不得**在未做块级对照前把任一分支写成已定论（本节初稿曾把"第二个 shape 家族"写成结论，root 自查后降级为待分岔，正是此因）。**MVP 边界沿用三之二**：单资源先行，双资源（5× any）独立扩验。

> **指令级补充（root 2026-10-04，`one()` 全指令双腿实录）**：差异**不止 handler 块——主路径几何本身不同**。javac 8 主路径为：`9: aconst_null; 10: astore_2`（primary-exception 槽置空）+ **守卫式 close**（`17: ifnull 46` 资源空则跳过、`21: ifnull 42` 有 primary 异常则走无抑制 close、否则 `24: close; addSuppressed 路径`）；而 javac 23 主路径的 close 是**无条件**的（`14: aload_1; 15: close`，无任何 ifnull、无 aconst_null 槽）。另外 javac 8 的 primary 异常进入抑制链的方式是"存槽→**athrow 重抛**→被 `48-55→53` 的 any 行再捕获"（48-52: astore_3/aload_3/astore_2/athrow），javac 9+ 无此再抛回路。这把分岔进一步压向"几何扩展"以上：主路径判据（现有 Resources 证明所认的"无条件 close"序）在 javac 8 形上**不成立**，故无论走哪条路都**必须**新增对"守卫式主 close + 空槽引导 + 重抛回路"的几何事实——正是"机制层扩展而非 idiom 补丁"（第三节结论）的指令级佐证。

## 四、健全性与严重性

- **失败是响亮的**：真 javac 8 的 TWR 方法 `not recovered` + 引注 + 整方法不投影，**不产生"可编译但行为不同"的文本**，符合核心不变量。故这不是静默偏离事故。
- **但它是目标层级上的真实覆盖缺口**：jarde 的唯一目标输出层级是 Java 8（`classfile.rs:145` `OutputLevel::Java8`），而真 javac 8 的 TWR 是 Java 8 极常见形（`try (Resource r = …) { … }`）。真 javac 8 产物与 javac 9+ `--release 8` 产物**都是 major version 52 的 Java 8 class**（root 实测四份产物 major 均 52），故真实世界的 JDK-8 编译产物会命中该缺口，而 `--release 8` 交叉编译产物不会。
- **TWR 缺口比 DT-03 大**：DT-03 只是 null-check 一条调用的拼写差异（`requireNonNull` vs `getClass`，结构同构），而 TWR 是整个 region 的指令序列与异常表结构差异（单方法 `one()`：48 vs 22 指令、5 vs 2 异常表行；整类 `TR`：16 vs 7 行，含双资源 `two()` 的 11 行），涉及 `aconst_null` 资源副本、`ifnull` 守卫关闭序列的 region/latch 证明。故 TWR **不是一条 idiom 拼写能覆盖的**，须独立取证其 region 证明能否扩展到 JDK 8 关闭序列——可能是中等到大颗粒的机制工作，非窄切片。

## 五、系统性归属（三例同根，这是本巡查的主要产出）

本会话连续发现三例**同一根因**的缺口：

| 构造 | javac 9+ 形（jarde 恢复） | 真 javac 8 形（jarde 拒绝） | 落点 | 颗粒度 |
| --- | --- | --- | --- | --- |
| DT-03 限定外部实例构造 | `Objects.requireNonNull(Object)Object` | `Object.getClass()Class` | `init.rs` / `member_inner.rs` | 窄（一条 idiom 拼写，结构同构）|
| 写访问器 | （读形 `getfield;ireturn` 已覆盖） | 返回值形 `dup_x1;putfield;ireturn` `(LC;D)D` | `accessor.rs:480` | 中（判据扩返回值形 + 健全性负例）|
| try-with-resources | 优化 codegen（`one()` 22 指令 / 2 异常表行）| `aconst_null` 引导 + `ifnull` 双守卫 + `addSuppressed` + `any` 行（`one()` 48 指令 / 5 行，其中 2 条 `any`）| `guard.rs` `fn twr`（14128）→ `Shape::Resources` | 中（变长 TWR 证明与事实模型已就位，扩展几何/初始化判据即可，**无需新机制或新 shape 变体**；见第三节更正后的判定）|

**共同根因**：整个 `tests/fixtures` 与 `openspec/evidence` 语料由 **javac 9+（含 `--release 8` 交叉编译）**产出，无真 javac 8 产物。故凡"编译器版本耦合的 codegen 惯用法"，其真 javac 8 形**对既有验收结构性不可见**——CI 全绿不代表真 Java 8 产物被覆盖。root 全仓普查佐证：`requireNonNull` null-check 形 8 类 / `getClass` 形 **0** 类；含 `access$` 的真实 class **0** 个；CF-17 的**两族** fixture 均为 javac 9+ 指纹（`cf17-twrcatch-patrol/T2.class` = 107 instrs / 0 `aconst_null`；`try-with-resources/original.class` 即 `TwrAudit` = 138 / 1，皆与 javac 23 重编逐值一致）。

**这不要求为每个构造新增机制**（Goal 的核心问题）：DT-03 与写访问器是"判据认事实而非认某一版本的拼写"，在既有架构内扩展即可（DT-03 已派发 [recover-javac8-getclass-null-check-idiom](../../../changes/recover-javac8-getclass-null-check-idiom/)，其 design 决策 1 的"单一所有者谓词 + 两处投影"就是该原则的落地）；TWR 则须先取证其 region 证明的可扩展性，可能是真实的机制工作。

**但三者共同要求一个横切的验收方法学修正**（比任何单点修复更重要）：**涉及 javac 合成 codegen 的能力，验收必须含真 javac 8 腿**，否则版本耦合盲区不可见。这已固化进 [handoff.md](../../../../handoff.md) 的"字节码惯用法普查必须按指令序列匹配"纪律（含双 javac 腿要求）。是否对整个 fixture 语料做一次"真 javac 8 双腿补强"是独立的大颗粒项，须单独评估（见下处置）。

## 六、处置

- **DT-03**：已派发 [recover-javac8-getclass-null-check-idiom](../../../changes/recover-javac8-getclass-null-check-idiom/)（窄片，单一谓词 + 两处投影，禁用版本门）。
- **写访问器**：已登记为 EM-15 下的独立债务（[summary.md](../../jadx-feature-inventory-2026-09-27/summary.md) EM 剔除条目段 + [取证 README](../value-returning-write-accessor-patrol/README.md)），待独立取证（须先读已合入的 `d09f5dea` private-setter-helper 通路）后立项。
- **TWR（本巡查）**：**不立即立 spec**——须先独立取证真 javac 8 的 TWR region 能否在既有 region/guard 架构内恢复（`aconst_null` 资源副本 + `ifnull` 守卫关闭序列），判定是窄扩展还是机制工作。登记为 **CF-17 的已证差距**（真 javac 8 形），排入取证队列。
- **横切（语料双腿补强）**：登记为独立大颗粒项，须评估成本（真 javac 8 重编全部 fixture 会改变 SHA、fingerprint、以及所有以字节 SHA 断言的测试），不在本巡查范围。

原 class 为行为基准（真 javac 8 与 javac 23 产物**行为相同**，root 以 `java -Xverify:all` 实测，逐行如下——注意 `closed:` 先于 `used:` 出现，因为资源在 `return` 求值后、`println` 收到返回值前就已关闭）：

```text
closed:p
used:p
closed:qb
closed:qa
used:qaused:qb
```
