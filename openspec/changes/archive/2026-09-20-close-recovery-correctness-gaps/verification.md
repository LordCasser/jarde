# 2026-09-20 完成性复核

本文件分两段：下面是**复核时点**（`cd6f2f0`）的判断与反例，第二段是**收尾实施与关闭**。第一段保留原样，因为第二条缺口正是从它的反例来的，改写它等于抹掉这段历史。

## 收尾实施与关闭（固定提交 `fd0aae8`）

[proposal](proposal.md) 的两条 P1 已在本 change 内关闭：`tasks.md` 4/4，反例、正向对照与执行基线已进入永久语料，固定提交的门禁已执行。

### 改了什么

`crates/jarde-java/src/build.rs` 两处判定，不新增实体、不新增依赖、不改公开 API：

- **实际求值位置贯穿渲染递归（P3-R8）**：`Builder::render_value` 的 `at` 明确为「这里生成的文本被求值的位置」，递归不再换成生产者 BCI——算术（1644–1645）、桥接 cast 的操作数（1688）、字段 receiver（1719）、数组读的表与选择子（1754–1755）、`new` 的实参（1977）、concat 片段（2307）都按 `at`；`call_expr`/`invoke_expr` 增加求值位置参数（1802–1807、1845），调用自身作为语句时仍传指令自己的 BCI（1205、1943）。每个节点的 origin 仍是它被生产出来的 BCI：origin 是来源、`at` 是求值，只有拒绝在两者之间移动。
- **fallback 收集未呈现的可观察生产者（P3-R9）**：`deferred_producers` 现在把已被 `field@1` 认领的字段读取按被拒读者的位置点名，并继续向它的操作数下降（2231–2241），于是静态读（可触发类初始化）、实例读（可抛 `NullPointerException`）与字段链都由产物引用交代；同时该 walk 的 load 判定固定在被拒消费者位置（2245），使 quote 点名的正是渲染器拒绝的那次读取。`invokedynamic` linkage 与 `enumswitch` 表读取仍不在该规则内，作为已知边界写在代码注释里。

### 怎么证明

- **反例**（受控 javac 23.0.1 `--release 8 -g:none`，公开入口 `Engine::recover_method`）：`nestedLocal` 修前写出 `arg0 = arg0 + 1; return arg0 + 1 + arg0;`（原方法 16、该文本 17），修后为 Mixed/Fallback，quote 点名 BCI 8 与被拒的 load BCI 0；同一类的调用实参形态 `nestedCall` 同样从 `tick(arg0) + arg0` 变为拒绝并点名 BCI 9/1。`fieldCast` 修前 quote 只有 `[3,6]` 且 `text_of_bci(0)` 为空（而字段记录称 `presented`），修后为 `{0,3,6}`。
- **正向对照**：`nestedPlain`（无跨写入的嵌套算术）仍 Java/Structured 并写出 `return arg0 + 1 + arg0 + 2;`；`leftRead`/`rightRead`（被认领的静态读配一次延迟调用，两种求值顺序）仍 Java/Structured 且调用只出现一次；`bump`/`doubleIt`/`loopAcross`、`p3_local_rewrite` 的 R1/R2 断言、`p3_execution_comparison` 既有成员分类、accessor/X1 双边与 A16/A17 隔离均未改变。
- **执行证据**：两个 fixture 各带一个提交的 baseline driver，由 ignored 对照用例编译并与样本一起运行、逐行断言——`nestedLocal(7)=16`、`nestedCall(3)=7`、`fieldCast()=ok`、`External` 初始化 1 次、`instanceCast(null)` 抛 `NullPointerException`、`chainCast()=chained`、`leftRead()/rightRead()=7`、被执行成员移动 `tick` 计数 3 次。本机 javac/java 均为 23.0.1。
- **变异**（各自单独执行、随后恢复并复跑绿）：算术语义递归改回 producer BCI → R8 用例红；调用求值位置改回 `bci` → `nestedCall` 用例红；删掉字段读取点名 → 三条 R9 用例与「presented 与实际产物一致」用例红；walk 的 use point 改回中间指令 → R8「quote 点名被拒读取」断言红（quote 退化为 `[8]`）。
- **库/CLI**：`crates/jarde-cli/tests/json_cli.rs` 对两个样本的每个成员走同一输入的两个入口并逐字段比较（只剔除 `elapsed_millis`），另断言 quote 集合 `8 0`、`9 1`、`{0,3,6}`、`{1,4,7}`、`{0,3,6,9}` 与 BCI 0 的字段记录/source map。

### 固定提交门禁（2026-09-20，`fd0aae8`）

| 门禁 | 结果 |
| --- | --- |
| `cargo fmt --all -- --check` | 通过 |
| `cargo test --workspace --all-targets --all-features --locked --no-fail-fast` | **1122 passed / 0 failed / 5 ignored**，61 个测试二进制（基线 1113 + 7 条 R8/R9 回归 + 2 条 CLI 逐字段用例） |
| `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` | 通过（改动 crate 与其依赖方重检，其余命中既有 clippy 缓存） |
| `cargo test --test p3_execution_comparison --locked -- --ignored` | 2 passed / 0 failed（JDK 23.0.1，18.8 s） |
| `cargo test --test p5_benchmark --locked -- --ignored` | 1 passed（smoke；本次不新增任何性能结论） |
| `git diff --check` | 干净 |
| `openspec validate --all --strict --no-interactive` | 归档前 19 passed / 0 failed（18 份主规格 + 本 change） |
| CI run on `fd0aae8` | [run 35490735745](https://github.com/LordCasser/jarde/actions/runs/35490735745)（attempt 1）**四 job success**：`stable / test and specification`（fmt、clippy `-D warnings`、两轮固定 seed 全量测试、**JDK 25 instruction-boundary oracle**、P3 编译执行对照、公开 API 示例、feature 树/依赖边界/分层闭包检查、OpenSpec strict、`git diff --exit-code`）、`MSRV 1.88.0`、`supply chain`、`fuzz smoke`（三个有界 fuzz 目标与提交语料检查） |

CI 只作为该固定提交的门禁证据；`cd6f2f0` 的 run 35484396101 四 job success 不构成本次验收。

**归档后校验**（`openspec/changes/archive/2026-09-20-close-recovery-correctness-gaps/`）：`openspec validate --archived --strict --no-interactive` = **9 passed / 0 failed**；`openspec validate --all --strict --no-interactive` = **18 passed / 4 failed**，其中 18 份主规格——含并入本次两份 delta 的 `java8-recovery` 与 `recovery-validation`——全部通过，4 项失败属于工作区里新出现的 4 份 proposal-only change（本次收尾之外、当时未提交，也不在本 change 范围内）。归档把两份 delta 各 `~ 1 modified` 合入主规格，原有 scenario 一条不少、各新增两条（见上文场景对照）。

### 本次留下的边界与记录

- 规则版本未变：15 条 `crate::pass` 规则都不拥有这两处判定（它们在 `build.rs` 的局部值等价检查与 quote walk 里），`pass.rs` 未被触碰。
- 未新增 crate、依赖、公开 API 或持久化格式；生产改动只在 `jarde-java`，另有 `jarde-reader` 的 `#[cfg(test)]` fixture 计数（37→41 classes、138→153 bodies）随新语料更新。
- `invokedynamic` linkage 与 `enumswitch` 表读取在其消费者被拒时仍不被点名，算术例外（`idiv`/`irem`）也不新增点名：本层自己的 effect 词汇就是「call 与字段读取是 effect；push、load 与算术是值表达式」（`pass.rs` 的 `StatementFree`/`Replayable`），收尾按这条词汇把「无语句但有 effect 的生产者」補全为字段读取，与既有的延迟调用、失效 load 同列。如果将来要在算术例外或 indy linkage 上点名，那是一条新规则，不是本次规则的回归。
- **记录修正**：`tests/fixtures/p3-corpus/README.md` 的 p3-handlers 行原写「70 lines」，本轮实跑为 **78**——该行的执行成员集在 `0b5eeb0` 加入 `open`/`openFailing` 之后未同步计数，行文本自 `04cf83a` 起未改；本次只改这一个实测数字，其余历史记录不动。
- **fixture 源修正**：`instanceCast(External e)` 原计划读 `External.value`，但那是静态字段（javac 会编译成 `getstatic`，没有 `getfield`、也不会抛 NPE），改为 `External` 的真实实例字段 `instance`，让「实例读可抛 NPE」这条验收真的有对象；字节码与 digest 见该 fixture 的 README。

## 判定与基线（复核时点，`cd6f2f0`）

**当前不能确认整体实现完成。** `cd6f2f07acdba690082dec97273d2c40c0102d96` 的 P0–P5 及分层已归档，P3/P4/P5 分别 12/12、10/10、10/10，18 份主规格已同步。但本轮独立反例证明当前恢复产物仍有两条 P1 正确性缺口，转入本 change，实施为 **0/4**。规划齐全不等于修复完成。

本轮只修改规划/说明文件；原有架构与 fuzz README 未提交修改已保留，生产代码和仓库测试未改。独立探针建在临时目录，通过 path dependency 使用当前源码，未添加进原有 1113 条回归计数。

## 已关闭的旧问题

- P3-R1–R4 的原反例均有修复与回归：直接 `return x++` 现在可靠降级；被拒 cast 前的普通调用保留；分支声明作用域修复；确定性比较递归排除 elapsed。
- P3-R5–R7 的 boolean 定型、debug slot 复用和未归属已解码指令检查已有交付。声明、receiver/debug、按需同类 callee 和物理方法 source map 已从公开入口接通，A12/A16/A17 不重开旧缺口。
- 简单 `return tick()` 重复调用仍为已关闭问题。新 R8/R9 是旧规则未覆盖的组合，不把旧修复记录改写为未发生。

## P3-R8 · P1：嵌套表达式用生产位置证明延迟求值

受控源方法：

```java
public static int nestedLocal(int x) {
    return (x + 1) + ++x;
}
```

OpenJDK 23.0.1，`javac --release 8 -g:none`；实际指令：`0 iload_0; 1 iconst_1; 2 iadd; 3 iinc 0,1; 6 iload_0; 7 iadd; 8 ireturn`（字节 `1a 04 60 84 00 01 1a 60 ac`）。按 Java8 RuntimeProfile、standalone snapshot、全部分析阶段调用 `Engine::recover_method`，结果 **Java/Structured**，生成体为：

```java
{
    arg0 = arg0 + 1;
    return arg0 + 1 + arg0;
}
```

把该 body 原样包入 `static int nestedLocal(int arg0)`，经 `javac --release 8` 和 `java -Xverify:all`：**原方法 nestedLocal(7)=16，生成方法=17**。这个结果不能用 Unchecked/Unproven 或“方法体不是完整文件”解释；包装没有改变方法体，错误发生在基本值语义。

根因：`crates/jarde-java/src/build.rs` 的 Arithmetic 分支递归调用 `render_value(..., bci, ...)`（基线 1627–1628 行），检查内层 load 时使用算术生产者 BCI 2；产物实际在 iinc 之后求值，旧值检查因此错误放行。修正要贯穿实际求值上下文，不能只对最外层 Load 增加守卫。

## P3-R9 · P1：字段生产者在 fallback 产物中丢失

受控源码的核心：

```java
public class Probe {
    static int calls;
    public static String fieldCast() { return (String) External.value; }
}
class External {
    static Object value;
    static { Probe.calls++; value = "ok"; }
}
```

同一编译方式，`fieldCast` 的实际指令为 `0 getstatic External.value:Object; 3 checkcast String; 6 areturn`。恢复只读取 Probe.class，不需要 External 的 Body；公开结果为 **Mixed/Fallback**，文本只引用 BCI **3、6**，没有 BCI 0 或 `External.value`。`report.text_of_bci(0)` 实测为 **[]**；同时 `report.fields` 却包含 `bci=0, presented=true, refusal=None`。

原 CLASS 的受控执行结果为 `fieldCast="ok", class_init_calls=1`，证明 getstatic 可观察。Mixed 不要求可编译，但必须在可靠语句或低级引用/source map 中保留该生产者；独立 FieldRecord 仍存在不能替代产物映射，也不能声称字段已经呈现。

根因：`build.rs` 的 Field 读取分支不发语句（基线 1334–1348 行），`deferred_producers` 主要收集 deferred invoke 与被覆盖 local，遍历到静态字段时没有操作数可继续走，字段 BCI 未加入 fallback。已有 CFG ledger 证明原指令属于某个块，不能证明它已被产物交代。修正须保留 effect 依赖闭包，同时避免重新引入重复调用。

## 独立探针与复现范围

探针使用一个自建 Probe.java/External CLASS，公开 Engine 建立 standalone snapshot，Header 提供 ClassBytesId，以 Java 8 profile 和 `AnalysisStage::ALL` 恢复指定成员。Rust 调用形状可复用现有 `tests/p3_local_rewrite.rs` 的 `recover_all`，只替换输入、方法名与 descriptor；未使用生产文本解析作为返回值 oracle。

临时证据位于本机 `/tmp/jarde-completion-probe-path` 指向的目录，含 Probe.java、Generated.java、Compare.java、逐方法 body 和独立 Rust 调用端；日志 `/tmp/jarde-completion-probes.log`。该目录不作为永久门禁，任务 2.1 负责把最小反例纳入正式语料。另测 leftRead/rightRead、callTwice、callBeforeStore 的呈现作为结构对照；fieldPost/callAcross 仍明确 fallback，不把这类拒绝记为已证明等价。

## 当前基线验证

| 验证 | 本轮结果 | 范围 |
| --- | --- | --- |
| `cargo test --workspace --all-targets --all-features --locked --no-fail-fast` | **1113 passed / 0 failed / 5 ignored**，60 个测试二进制 | 原仓库用例；不含临时反例 |
| `cargo test --test p3_execution_comparison --locked -- --ignored` | **2 passed** | 本机 OpenJDK 23.0.1，原有受控编译/执行语料 |
| `cargo test --test p5_benchmark --locked -- --ignored` | **1 passed** | benchmark smoke；未据此新增性能结论 |
| `cargo fmt --all -- --check` | 通过 | 当前源码 |
| `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` | 通过 | 当前源码 |
| 当前 HEAD CI | **四 job success** | 见下；不包含本轮未提交的规划修改 |

[CI run 35484396101](https://github.com/LordCasser/jarde/actions/runs/35484396101) 对应完整 SHA `cd6f2f07acdba690082dec97273d2c40c0102d96`，本轮读取 GitHub 结果确认 stable、MSRV 1.88、supply-chain、fuzz-smoke 全部 success。stable 的 JDK 25 instruction-boundary oracle、两轮固定 seed 测试、P3 编译执行、示例及依赖门禁均 success。本机只有 JDK 23/18，没有声称在本机运行 JDK25；CI 的显式成功补足这一环境证据。

五条默认 ignored 中，P3 两条与 P5 benchmark 本轮已显式运行；JDK25 oracle 由上述 CI 验证；fingerprint 再生成器会改语料清单，不是尚未通过的测试。

这些绿色证明已有门禁覆盖的范围；R8/R9 恰好证明其未覆盖的边界，不能据此确认整体完成。

## 验收口径与后续顺序

A15/A18 对已实现路径的适用范围通过：完整执行的语义/identity/origin/coverage/diagnostics 必须一致，实际减少的 usage charge 单独记录；不存在的 index/parallel/merged 路径为不适用，不要求为了通过而先实现。P5 归档中的“部分通过”保留为历史判断，当前按主规格重新解释。差分等价也抓不到两侧共同存在的 R8/R9，必须保留独立反例。

当前顺序：**R8 实际求值上下文 → R9 fallback producer/origin → 永久独立语料与库/CLI → 修正后固定提交门禁与完成确认**。阶段旧任务不重做。MethodParameters、类级事实、现代恢复/CLI、handler 根策略、容量权重和规模语料单列后续范围，不能混入两条 P1 修复。

## 规划收尾校验

本轮 OpenSpec strict：**19 passed / 0 failed**（18 份主规格 + 1 个 active change）。规划四项工件齐全，实施仍为 **0/4**；两个 MODIFIED requirement 分别保留原有 6/4 个 scenario，各补 2 个反例/验收场景。14 份当前相关 Markdown 的 111 个本地路径/锚点均可解析，`git diff --check` 干净。当前入口、路线、验收、支持矩阵和 OpenSpec context 已同步；历史 archive 未改。

## 独立 oracle 探针（jadx，2026-09-20，任务 2.1 的最小范围）

**目的**：为 R8/R9 增加一条**假设不同**的独立证据。架构说明已明确「JADX、其他反编译器和 JDK 工具可作为算法参考及测试 oracle；不存在唯一 oracle」，本探针即该定位的第一个具体用法。

**环境**：jadx **1.5.6**（`/opt/homebrew/bin/jadx`）、OpenJDK **23.0.1**、`javac --release 8 -g:none`。探针全部在 `/tmp/jadx-oracle`，**未进仓库、未进回归计数**。

**方法**：按本文件两条反例重建受控源 → 编译 → jadx 反编译 → **编译 jadx 产物并执行** → 与原 CLASS 基准比对。**只做行为对照**：jadx 会重写，文本比较无判别力。

| 反例 | 基准（原 CLASS） | jadx 产物（编译+执行） | jarde 产物 |
| --- | --- | --- | --- |
| R8 `nestedLocal(7)` | **16** | **16 ✅** | 17 ❌ |
| R9 `fieldCast()` / `External` 初始化次数 | `"ok"` / **1** | `"ok"` / **1** ✅ | `getstatic` 在文本与 source map 中均缺失 ❌ |

**jadx 的 R8 产物**（说明为什么不能比文本）：

```java
public static int nestedLocal(int i) {
    return i + 1 + i + 1;
}
```

它**不忠实**：没有还原 `++x`，而是把自增折成 `+1`——因为它能证明 `i` 在 `return` 后不可观测。值语义正确（16），结构不等价。这是一处真实的能力差异：jadx 求「最简的正确 Java」，jarde 求「保语义 + 保来源 + 不确定则拒绝」。

**外加语料事实**：jadx 对现有 **37/37** 个 fixture CLASS 全部产出源码，无失败。

**结论与限制**：jadx 对这两个反例构成**可用的行为 oracle**。但它**不能**判定 jarde 的拒绝（jadx 总是尝试，拒绝是 jarde 的设计而非缺陷），**不能**证明结构忠实（已实测会重写），也**不能**消除共同假设盲区（两者都实现 JVMS，共同误读不会暴露），与本文件前面「差分等价抓不到两侧共存的 R8/R9」同源。**它通过不构成 jarde 正确性的证明**；只有不一致才是线索，且仍需受控确认。

**benchmark 暂缓（已测但不足以发布）**：同一台机器上 jadx 的固定开销压倒工作量——`java -version` 40–85 ms、jadx 空输入（仅自身初始化）464–518 ms、37 个 CLASS（21 KB jar）991–1037 ms，且**1 个与 37 个几乎相同**。即 ~0.5 s/次是 JVM 与 jadx 初始化，37 个 CLASS 的实际工作量仅约 450 ms（≈12 ms/class）。与 P5 已发布的 jarde 基线（131–147 µs/查询，进程内、无启动）直接相除会得到「快 8000×」这类**测错对象的数字**，属 P5 明令禁止的无证据性能数字。**待大型语料到位后按重复测量与固定开销分离重做**；发布仍只声明测得范围，不设阈值。

