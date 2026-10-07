# 任务 2/3 证据：行集落地 + 双 javac 腿锚 + 测试（change `recover-boxed-number-widening`）

## 实现（file: `crates/jarde-java/src/build.rs`）

`platform_interface_argument_widens` 内新增一张 `const` 行表并在 `.any()` 链里入列：

- `NUMBER_FAMILY`：**6 行**——`java.lang.{Byte,Short,Integer,Long,Float,Double}` → `java.lang.Number`。

命中仍走既有 `cast_argument(argument, required, bci)` 呈现（零第三种呈现）；release-8 门与阵列位谓词
（`platform_array_argument_widens`，同一函数即其组件回答者）逐字未动；既有六张表的行逐字节未改。

**diff 形态（如实披露）**：`build.rs` 文件级 = **102 插入 / 3 删除**，其中本片行表与文档 = 34 插入 / 3 删除、
单元测试 `the_boxed_number_rows_reach_exactly_their_pairs` = 68 插入。3 行删除是函数文档注释**第一段**由
"implementer sets" 改写为 "implementer sets and superclasses"（点名 `java.lang.Number` 族）；`.any()` 链
数组加一行 `NUMBER_FAMILY,`。既有段落、既有表行、门与诊断文本逐字未动（`git diff` 复核）。

## 单元测试（`build.rs` `mod tests`）

`the_boxed_number_rows_reach_exactly_their_pairs`：**6 条正例**（每行一条，release 8）+ **20 条负例**
（release 7/9 门、非 Number 子类 `Boolean`/`Character`/`String`/`Object`、表外子类
`BigDecimal`/`BigInteger`/`AtomicInteger`/`AtomicLong`/`LongAdder`（经 `Striped64` 间接到达）、
`Number → Serializable`（表只陈述六个子类）、下行 `Number → Integer`、无关向 `Number → Comparable`、
`Number → Object`、数组位、原始型、泛型拼写、用户类）。既有测试
`platform_interface_argument_widening_reaches_exactly_the_table_rows` **逐字未改**且仍绿（其负例里没有一条
以 `Number` 为目标，故本片不放行任何既有拒绝）。

实测：`cargo test -p jarde-java --lib the_boxed_number_rows_reach_exactly_their_pairs` = 1 passed / 0 failed；
`…platform_interface_argument_widening_reaches_exactly_the_table_rows` = 1 passed / 0 failed。

## 冻结锚（`tests/fixtures/recover-boxed-number-widening/`，两条 javac 腿）

源：巡查 fixture `C8.java` **逐字节复制**（`cmp` 实测相同）+ 本片 `BN.java`（六行 + 变体）/`BNX.java`（负例）；
腿：`javac --release 8 -Xlint:-options -d v8 *.java`（javac 23.0.1）与真 javac 8（Corretto 1.8.0_432）
`-d v8-javac8`。SHA 与行为见 fixture README。

**来源核验**：巡查 `fixture/C8.class`（`9d2b5071…`）与本片两条腿的 `C8.class` 字节不同（同一份源、不同
编译产物；巡查 class 的 minor/major 同为 52，差在常量池/属性细节），但**行为三条腿相同**（`x/1:2/7/eoeoeoe`）。

## 锚渲染（两条腿逐字一致；`tests/recover_boxed_number_widening.rs` 钉住整段文本）

| 锚 | base（本片前） | patched（本片） |
| --- | --- | --- |
| `C8.main` | 整成员引注（BCI 63 `Integer → Number` 拒绝） | `java.lang.System.out.println((java.lang.Object) larger((java.lang.Number) java.lang.Integer.valueOf(3), (java.lang.Number) java.lang.Integer.valueOf(7)));`，0 引注 |
| `C8.useWitness`/`loopBuilder`/`larger` | 健康面 | 逐字不变 |
| `BN.main` | 8 引注（六行 + 形参位） | 九个语句全呈现（六行各一位点 + `(java.lang.CharSequence)` 变体 + 同型控制 + 形参位），0 引注 |
| `BN.withParam` | 整成员引注（BCI 5） | `return larger((java.lang.Number) arg0, (java.lang.Number) java.lang.Integer.valueOf(0));` |
| `BN.pickSeq` | 0 引注（姊妹片） | 逐字不变 |
| `BNX.main` | 2 引注（`BigDecimal` BCI 21、`AtomicInteger` BCI 46） | **2 引注逐字不变** |

## 根测试（`tests/recover_boxed_number_widening.rs`，facade Engine + `ClassSourceRequest` + jar + 自头断言）

- `the_patrol_anchor_presents_its_boxed_argument`（C8 四段文本 + 全类零引注）
- `the_boxed_number_positions_are_presented`（BN 五段文本 + 全类零引注）
- `the_number_subclasses_outside_the_closed_rows_still_refuse`（BNX 一段文本 + 引注计数恰 2）
- `every_stripped_anchor_answers_what_its_class_answers`（`#[ignore]`：剥离注释→装机 `javac --release 8` 与
  真 javac 8 双编译→`-Xverify:all` 运行→与 fixture 自身 class 的答案逐字节一致）

实测：`cargo test --locked --test recover_boxed_number_widening` = 3 passed / 0 failed / 1 ignored；
`JARDE_JAVAC8=/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home/bin/javac cargo test … -- --ignored`
= 1 passed（真 javac 8 腿在场）。同样的双腿往返也由 `results/cli-roundtrip.sh` 走 CLI 入口复现
（`results/cli-roundtrip.out`：`ROUNDTRIP OK`）。

## 预算/取消（任务 2.2）

本片只加行表，不触预算维度、步数、取消点或遍历形状：diff 里没有任何 budget/cancellation 代码路径。
实测 `cargo test --locked --test generic_method_budget`（`<T extends Number>` 形参的预算/取消族）全绿，
工作区全量门禁见 `03-gates.md`。
