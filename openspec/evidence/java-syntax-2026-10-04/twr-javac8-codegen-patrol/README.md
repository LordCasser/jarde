# try-with-resources 的真 javac 8 codegen 不被恢复（2026-10-04，root 巡查）——版本耦合缺口第三例

**结论：jarde 的 CF-17 try-with-resources 恢复能力只识别 javac 9+ 的优化 codegen，对真 javac 8（Corretto 1.8.0_432）的 TWR 形（`aconst_null` 前置 + `ifnull` 守卫关闭序列）**整方法拒绝**（响亮失败，非静默偏离）。CF-17 的验收 fixture（`T2.class` 等）经结构指纹判定由 javac 9+ 编译，故该缺口对既有验收不可见。这是本会话发现的第三个"语料由 javac 9+ 编译、真实 Java 8 产物有盲区"的实例（前两例：[DT-03 `getClass` null-check](../qualified-outer-alloc-getclass-patrol/README.md)、[返回值形写访问器](../value-returning-write-accessor-patrol/README.md)），三者同根，见末尾"系统性归属"。**

固定转录见 [fixture](fixture/)（`TR.java` + 真 javac 8 与 javac 23 两套 class）与 [results](results/)（双腿渲染、javap codegen 对照）。

## 一、决定性单变量实验（同一 `TR.java`，两种 javac）

root 构造无 widening 混淆的 TWR 探针 [fixture/TR.java](fixture/TR.java)（`one()` 单一资源、`two()` 双资源，资源类自实现 `AutoCloseable`）：

| 编译工具链 | `one()` 指令数 | `two()` 指令数 | 异常表项 | jarde 渲染引注 | `one()`/`two()` 恢复 |
| --- | --- | --- | --- | --- | --- |
| javac 23 `--release 8` | 22 | 61 | 7 | **0** | ✓ `try (TR local1 = new TR(arg0)) { … }` |
| **真 javac 8**（Corretto 1.8.0_432） | **48** | **113** | **16** | **6** | ✗ 整方法 `not recovered` |

即真 javac 8 的 TWR codegen 约为 javac 9+ 的 **2.2 倍指令量、2.3 倍异常表项**（JDK 9 对 TWR 做了重大 codegen 简化），jarde 只恢复了后者。

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

**判据是 CFG/异常表驱动，不是指令序列匹配**。root 读码核实：TWR 的形证明在 `crates/jarde-java/src/guard.rs`（`Shape::NullableResourceFinally` 声明于 `guard.rs:363`、构造于 `3853`），其证明通篇用**块与异常表事实**——`facts.blocks_in((body_start, cleanup_start))`、`facts.covering(bci)` 的 row ordinal 集合恰等、`view.successor_ids(&handler_entry)` 须恰为 `{handler_call_block, rethrow_block}`、`successor_ids(&handler_call_block) == [rethrow_block]`、`successor_ids(&rethrow_block).is_empty()`、以及每条 `CanonicalEdgeKind::{Exception,Normal,Return}` 边的端点归属校验。区域侧 `region.rs:5252` `nullable_resource_finally_regions` 消费该 shape。

**而两腿的异常表拓扑不同**（root javap 实测 `P08_twr.one()`）：

| 腿 | 异常表行数 | `any`（catch-all）行 | 保护区间 |
| --- | --- | --- | --- |
| javac 23 `--release 8` | **2** | **0** | `8→11 target 17`、`18→22 target 25`（均 `Class java/lang/Throwable`） |
| 真 javac 8 | **5** | **2** | `21→25 target 28`、`10→13 target 43`、**`10→13 target 48 any`**、`58→62 target 65`、**`43→50 target 48 any`** |

即真 javac 8 的 TWR 用**两条 `any` catch-all 行**表达"正常路径与异常路径都要关闭资源"的 JDK 8 codegen，且保护区间与关闭块被复制多份（`one()` 48 指令 / 16 异常表项 vs javac 9+ 的 22 / 7）。

**结论（对 Goal 核心问题的回答）**：TWR **不是"再认一种 idiom 拼写"能覆盖的**——它要求 `guard` 的 `NullableResourceFinally` 形证明**接受一套不同的块/异常表拓扑**（含 `any` 行的归属、复制的关闭路径、以及 `successor_ids` 恰等集合的相应放宽或分支）。这属**机制层扩展**（中大颗粒），与 DT-03（同构序列里换一条调用拼写，窄片）、EM-15（判据扩返回值形 + 健全性负例，中片）都不同。故：

- **不并入 DT-03 片**（落点、判据性质、颗粒度都不同）。
- **立项前须先做一次独立取证**：把真 javac 8 的 `one()`/`two()` 块图与异常表逐块画出，判定 `NullableResourceFinally` 是"加一个 `any`-行变体"即可覆盖，还是需要第二个 shape（如 `ResourceFinallyJdk8`）。root 未做该块图取证，**不外推**其结论。
- 取证前的诚实判断：**若**只需为 `any` 行加归属规则，则是中片；**若**关闭路径复制导致 `successor_ids` 恰等集合无法用单一 shape 表达，则需新 shape，属大颗粒。这个分岔只能由块图取证决定。

## 四、健全性与严重性

- **失败是响亮的**：真 javac 8 的 TWR 方法 `not recovered` + 引注 + 整方法不投影，**不产生"可编译但行为不同"的文本**，符合核心不变量。故这不是静默偏离事故。
- **但它是目标层级上的真实覆盖缺口**：jarde 的唯一目标输出层级是 Java 8（`classfile.rs:145` `OutputLevel::Java8`），而真 javac 8 的 TWR 是 Java 8 极常见形（`try (Resource r = …) { … }`）。真 javac 8 产物与 javac 9+ `--release 8` 产物**都是 major version 52 的 Java 8 class**（root 实测四份产物 major 均 52），故真实世界的 JDK-8 编译产物会命中该缺口，而 `--release 8` 交叉编译产物不会。
- **TWR 缺口比 DT-03 大**：DT-03 只是 null-check 一条调用的拼写差异（`requireNonNull` vs `getClass`，结构同构），而 TWR 是整个 region 的指令序列与异常表结构差异（48 vs 22 指令、16 vs 7 异常表项），涉及 `aconst_null` 资源副本、`ifnull` 守卫关闭序列的 region/latch 证明。故 TWR **不是一条 idiom 拼写能覆盖的**，须独立取证其 region 证明能否扩展到 JDK 8 关闭序列——可能是中等到大颗粒的机制工作，非窄切片。

## 五、系统性归属（三例同根，这是本巡查的主要产出）

本会话连续发现三例**同一根因**的缺口：

| 构造 | javac 9+ 形（jarde 恢复） | 真 javac 8 形（jarde 拒绝） | 落点 | 颗粒度 |
| --- | --- | --- | --- | --- |
| DT-03 限定外部实例构造 | `Objects.requireNonNull(Object)Object` | `Object.getClass()Class` | `init.rs` / `member_inner.rs` | 窄（一条 idiom 拼写，结构同构）|
| 写访问器 | （读形 `getfield;ireturn` 已覆盖） | 返回值形 `dup_x1;putfield;ireturn` `(LC;D)D` | `accessor.rs:480` | 中（判据扩返回值形 + 健全性负例）|
| try-with-resources | 优化 codegen（22 指令 / 7 异常表）| `aconst_null`+`ifnull` 守卫（48 指令 / 16 异常表）| TWR region 证明 | 大（region/异常表结构，非 idiom）|

**共同根因**：整个 `tests/fixtures` 与 `openspec/evidence` 语料由 **javac 9+（含 `--release 8` 交叉编译）**产出，无真 javac 8 产物。故凡"编译器版本耦合的 codegen 惯用法"，其真 javac 8 形**对既有验收结构性不可见**——CI 全绿不代表真 Java 8 产物被覆盖。root 全仓普查佐证：`requireNonNull` 形 8 类 / `getClass` 形 **0** 类；含 `access$` 的真实 class **0** 个；CF-17 的 `T2.class` 为 javac 9+ 指纹。

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
