# 验证（change `recover-boxed-number-widening`）

实现者自查记录。任务证据在 `results/`：`01-anchors-and-rows.md`（锚复验/行集来源/前提漂移）、
`02-rows-anchors-tests.md`（落表/锚/测试）、`03-gates.md`（门禁/语料/指纹）。

## 实现（`crates/jarde-java/src/build.rs`）

`platform_interface_argument_widens` 内新增 `NUMBER_FAMILY`（六行，六数值装箱 → `java.lang.Number`）并在
`.any()` 链入列；命中走既有 `cast_argument` 呈现（零第三种呈现）；release-8 门、阵列位谓词与既有六张表的行
逐字未动。`build.rs` 文件级 diff = 102 插入 / 3 删除（行表 + 文档 34/3；本片单元测试 68 插入）；3 行删除是函数
文档首段改写为 "implementer sets and superclasses"。

## 前提漂移与诚实收窄（两处，证据在 `results/01-anchors-and-rows.md`）

1. proposal 立项时把 java.lang 整族（六装箱→Number、八装箱/String→Comparable、String→CharSequence）算作缺口；
   `recover-charsequence-argument-widening`/`recover-comparable-argument-widening`（2026-10-06）已落地其中
   `CharSequence`/`Comparable`/`Serializable` 三表——**实测 `BN.pickSeq` 在基线二进制上即 0 引注**。故本片
   只实现剩余六条 `→ java.lang.Number` 边，不重复、不改既有呈现。
2. tasks 2.1 的"表+walk"中 **walk 一项按反射核对收窄**：java.lang 直接子类恰六、间接 0（无内部节点），且该函数
   任一表的目标都不是另一张表的源 → 闭包 = 行集，walk 会是死代码。java.util/Throwable 的 walk 由各自的链
   （`List→Collection→Iterable` 等）承载，不是本族的形状。

## 锚与行为

- `C8`（巡查锚）：BCI 63 的 `Integer → java.lang.Number` 拒绝消失，整类 0 引注；剥离后由装机 `javac --release 8`
  与真 javac 8（Corretto 1.8.0_432）各编译一次、`-Xverify:all` 运行，答案 `x/1:2/7/eoeoeoe` 与 fixture 自身
  class 逐字一致（`results/cli-roundtrip.out`）。
- `BN`（六行 + 变体）：六行各一位点 + `String→CharSequence` 变体 + 同型控制 + 装箱形参位，0 引注；
  两腿运行 `7/7/7.5/7.5/7/7/yy/9/4` 与自身 class 一致。
- `BNX`（负例）：`BigDecimal`/`AtomicInteger` 两条拒绝**逐字不变**（计数恰 2）。
- 单元测试：`the_boxed_number_rows_reach_exactly_their_pairs`（6 正 + 20 负）；既有
  `platform_interface_argument_widening_reaches_exactly_the_table_rows` **未改且绿**。
- 根测试：`tests/recover_boxed_number_widening.rs`（3 passed + 1 ignored 双腿 replay；facade Engine +
  `ClassSourceRequest` + jar 自头断言）。

## 零回退

- **全量语料双腿渲染 diff**：`openspec/evidence/**` 的 **1987** 个 class，基线二进制 vs patched 二进制，
  移动**仅 1 类**——巡查 `C8.class`（引注 1→0），新增引注 0、非渲染 0（`results/full-corpus-diff.out`，
  自测：C8 必须动、`C7` 必须不动）。查询类名由 `javap` 声明行解析，故文件名≠内部名的 fixture 也在比较面内。
- **候选面**：目标描述符 `Ljava/lang/Number;` 的候选（散装 16 + jar 条目 6 = 22，非渲染 0）里移动 2 处渲染，
  均为同一 `C8`（散装 + `c8.jar` 条目），引注 2→0（`results/corpus-sweep.out`，自测：正例 `C8`/`BN` 必须归零、
  负例 `BNX` 必须保持 2）。候选面对该变化是**完备**的：要求类型 `java.lang.Number` 只能来自调用自己的
  Methodref 描述符，故任何受影响的类其常量池必含该描述符。
- 既有表/通道逐字未改（`git diff` 复核）；工作区全量测试、fmt、CI-exact clippy、`openspec validate --all --strict`、
  `git diff --check` 见 `results/03-gates.md`；`p3_execution_comparison --ignored` 语料移动纪律 3 passed / 0 failed
  （无需更新任何陈旧 oracle 期望）。

## 边界与未验证面

- 表外 `Number` 子类（`java.math.*`、`java.util.concurrent.atomic.*`）保持拒绝——保守，本片按 spec 的封闭六行；
- `Number` 自身 → `Serializable` 保持拒绝（表只陈述六个子类的该关系；`Number` 的 `Object` 位由既有 Object 分支
  回答，不在本表）；
- 平台→平台、用户类、快照内层级（snapshot walk）通道未触，仍按各自先例回答；
- `root` 独立复核（tasks 3.3：闭集逐对、三方行为、EM 账本与巡查记录）尚未进行——本片实现者自查不等于独立 review。
