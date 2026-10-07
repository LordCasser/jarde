# 任务 1 证据：锚复验、行集来源、前提漂移（change `recover-boxed-number-widening`）

## 1.1 巡查锚在 HEAD 复验（按锚名，不按行号）

| 项 | 巡查记录（2026-10-03） | HEAD 复验（实测） |
| --- | --- | --- |
| fixture SHA | `fixture-sha256.txt`：`C7.class` `8dc9933c…`、`C7$Consts.class` `e51cf9ac…`、`C8.class` `9d2b5071…` | `shasum -a 256` 三条**逐字相同**（`c8.jar` 内 `C8.class` 亦为 `9d2b5071…`） |
| 拒绝点 | C8 BCI 63：`the parameter 0 of the invocation at BCI 63 is declared \`java.lang.Number\` presents \`java.lang.Integer\` but the invocation requires \`java.lang.Number\` and this layer has no safe reference conversion evidence` | 基线二进制（本片行落地前的 HEAD，`/tmp/bn-render/jarde-cli-base`）渲染 `c8.jar`：**同句逐字**，引注计数 1（`gate/C8.base.txt` 第 64 行） |
| 健康面 | `useWitness`（显式见证）、`boxedTern`、`loopBuilder`（循环携带 builder）已恢复 | 逐字未动（`gate/C8.base.txt`） |

**呈现形态的漂移（如实记录）**：巡查当时 `C8.txt` 记的是"部分体 + 丢行"（`main` 里 `larger(3, 7)` 一行被引、
其余语句仍在）；今天 HEAD 的呈现把**整个成员**引注（`// jarde: not recovered: … produced no statement` +
`jarde_refused_body();`）。拒绝句与 BCI 完全一致，漂移只在呈现形态（这期间落地的呈现规则变更），不影响锚。

## 1.1 反射核对：java.lang 直接边全集

`probe/NumberUniverse.java` + `probe/run-number-universe.sh`（Corretto 1.8.0_432，`rt.jar` sha256
`b27515a6…` 与行来源文件一致）把 rt.jar 的 **20400** 个类名逐一 `Class.forName(name, false, …)`，
输出见 `probe/number-universe.out`：

- java.lang 直接子类（`getSuperclass() == Number.class`）**恰为六个**：`Byte`/`Double`/`Float`/`Integer`/`Long`/`Short`；
- java.lang 间接子类（经别的类到达 `Number`）**0 个**；
- java.lang 之外：直接 5 个（`java.math.BigDecimal`、`java.math.BigInteger`、
  `java.util.concurrent.atomic.AtomicInteger`/`AtomicLong`/`Striped64`）、间接 4 个（`DoubleAccumulator`/
  `DoubleAdder`/`LongAccumulator`/`LongAdder`）；
- `Number` 自身：`superclass = java.lang.Object`、`interfaces = [java.io.Serializable]`（**不实现 `Comparable`**）。

**结论（本片行集的取证义务）**：六行即 java.lang 的**全部**直接边；java.lang 内**没有**需要传递闭包 walk
的中间节点。因此本片按 `NUMBER_FAMILY` 六行落表，**不新增 walk**：现有函数里任何一张表的目标都不是另一张表
的源（`CharSequence`/`Comparable`/`Serializable`/`AbstractSet`/`Set`/`Collection`/`Iterable`/`Temporal`/
`TemporalAccessor`/`CompletionStage`/`Future`/`Number` 都不作为源出现），闭包与行集**相等**，walk 会是死代码。
java.util 表与 Throwable 通道的 walk 之所以必要，是因为它们的表里有 `List -> Collection -> Iterable` 这类链
（java.util 表）——那是它们的行集形状，不是本族的。设计文书写的"表+walk"中 walk 一项按此**如实收窄**。

## 1.2 冻结的变体/负例与前后行为

变体集 = `tests/fixtures/recover-boxed-number-widening/{C8,BN,BNX}.java`（两条 javac 腿：
`javac --release 8 -Xlint:-options`（23.0.1）与真 javac 8 Corretto 1.8.0_432；命令与 SHA 见 fixture README）。

| 形状 | 位点 | base（本片前） | patched（本片） |
| --- | --- | --- | --- |
| 巡查锚 `C8.main` `larger(3, 7)` | `Integer → Number`（BCI 63） | 1 引注 | 0 引注，整类呈现 |
| Double/Long 装箱变体（`BN.main`） | `Double → Number`、`Long → Number` | 各 1 引注 | 0 引注 |
| 六行全覆盖（`BN.main`） | `Integer`/`Long`/`Double`/`Float`/`Short`/`Byte` → `Number` | 8 引注（`main` 7 + `withParam` 1） | 0 引注 |
| `String → CharSequence` 变体（`BN.pickSeq`） | 擦除 `CharSequence` 形参 | **0 引注（已由姊妹片覆盖，前提漂移）** | 逐字不变 |
| 装箱**形参**位（`BN.withParam`） | `Integer` 参数 → `Number` | 1 引注 | 0 引注，两个实参位都呈现 |
| 同型控制（`BN.same`） | `Number → Number` | 0 引注、无 cast | 逐字不变 |
| 负例：`BigDecimal`/`AtomicInteger → Number`（`BNX.main`） | 表外 Number 子类 | 2 引注（逐字） | **2 引注（逐字不变）** |
| 负例：`Boolean → Number` | — | 源级**不可产生**（`Boolean` 不转换到 `Number`，javac 拒绝） | 钉在 `build.rs` 单元测试（表级拒绝） |

`java -Xverify:all` 行为（两条腿相同，源 class 与两腿一致）：`C8` = `x/1:2/7/eoeoeoe`、
`BN` = `7/7/7.5/7.5/7/7/yy/9/4`、`BNX` = `2/2`（记录见 `cli-roundtrip.out` 与本目录表格）。

## 前提漂移（本片诚实收窄的两处）

1. **`String → CharSequence` / 八装箱 → `Comparable` / → `Serializable` 已落地**：立项（2026-10-04）时
   `platform_reference_argument_widens` 闭集只覆盖 java.util 与 Throwable，故 proposal 把 java.lang 边
   （六数值装箱→Number、全部→Comparable、String→CharSequence/Comparable）整族算作缺口。此后
   `recover-charsequence-argument-widening`/`recover-comparable-argument-widening`（2026-10-06）已把
   `platform_interface_argument_widens` 的 `CHAR_SEQUENCE`/`COMPARABLE`/`SERIALIZABLE` 表落地——实测
   `BN.pickSeq` 在**基线**二进制上就 0 引注、`CO`/`CS` 的既有测试全绿。故本片**剩余 delta 恰为六条
   `→ java.lang.Number` 边**，其余不重复实现、不改既有呈现。
2. **"表+walk"的 walk 一项**：见上节反射核对——java.lang 直接边全集=六、无中间节点，walk 无内部可走；
   按既有行集纪律不写死代码。

## 行集来源（转录协议）

`openspec/evidence/java-syntax-2026-10-05/widening-row-sources/`：六行的依据是该文件**既有**的六装箱行
（各自逐字 `extends java.lang.Number`），`java.lang.Number` 自身的行也在案；本片不新增转录行，只增一段行集表。
自检 `results/selfcheck-javap-rows.sh`（同一 rt.jar、同一命令）对七型重跑 `javap`、按索引列重排后与该文件
**逐字节 diff 相同**（输出 `selfcheck-javap-rows.out`：`SELF-CHECK OK`）。
