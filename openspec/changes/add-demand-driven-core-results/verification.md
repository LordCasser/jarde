# D0 验证记录：冻结 revision、字段角色与计数见证（1.1 / 1.2 / 1.3）

本文件是 `add-demand-driven-core-results` **D0 阶段**的实施记录。它只登记已在本机真实执行的观测、命令与结果：
冻结的实施 revision 与 dirty 范围（1.1）、逐字段角色清点和 CFG/SSA 分离复核（1.2）、有界计数见证与人为变异证据（1.3）。

三件仍然开放的正确性缺口（数组参数槽宽、`append(int)` 消费 `char` 的 T5、活动容器跨 worker 交接等）**不**在本 change 内修复，也**不**因本文件被标成已修复；见 §5 与 proposal 的登记方式。

---

## 1. 实施 revision 与 dirty 范围（1.1，D03/D12）

冻结命令与真实输出（在本 worktree 内执行）：

```text
$ pwd
/Users/lordcasser/.grow/worktrees/projects-jarde/subagent-01a0c2e9-643b-7202-995e-6df7e2a170be

$ git rev-parse HEAD
809ca81c79ddaa99e9cacb42ad5c8a2e6a147310

$ git log -1 --format='%H %ad %s' --date=iso
809ca81c79ddaa99e9cacb42ad5c8a2e6a147310 2026-09-21 15:25:56 +0800 perf: count the operation's totals in atoms, not behind one lock

$ git status --porcelain=v1
 M JVM_Rust_Engine_Final_Architecture.md
 M openspec/benchmark-protocol.md
 M openspec/changes/add-parallel-bulk-recovery/design.md
 M openspec/changes/add-parallel-bulk-recovery/evidence/cost-attribution.md
 M openspec/changes/add-parallel-bulk-recovery/specs/measured-execution/spec.md
 M openspec/changes/add-parallel-bulk-recovery/tasks.md
 M openspec/changes/add-parallel-bulk-recovery/verification.md
 M openspec/changes/optimize-demand-workloads/design.md
 M openspec/changes/optimize-demand-workloads/proposal.md
 M openspec/changes/optimize-demand-workloads/specs/measured-execution/spec.md
 M openspec/changes/optimize-demand-workloads/tasks.md
 M openspec/roadmap.md
?? openspec/changes/add-demand-driven-core-results/
?? openspec/changes/add-parallel-bulk-recovery/evidence/arm-comparison.md
?? openspec/changes/fix-descriptor-slot-facts/
```

**dirty patch 的构成**：冻结时刻的 dirty 内容**全部是文档**——12 个被修改的 Markdown（架构文档、benchmark 协议、两个其它 change 的规划/证据、`openspec/roadmap.md`）和 3 个未跟踪目录（本 change 的文档、bulk 流的 `arm-comparison.md`、descriptor 流的 `fix-descriptor-slot-facts/`）。
`git diff --name-only -- src crates tests Cargo.toml Cargo.lock` 在 D0 动工前输出为空，即**冻结基线在源码层面等于 `809ca81`**：三个并行流的源码改动都在各自的工作区，未出现在本 worktree 的 dirty 集合里。

**本轮（D0）对源码的唯一改动**，与冻结基线一起冻结：

| 文件 | 性质 | 摘要（sha256） |
| --- | --- | --- |
| `src/d0_counts.rs`（新增） | test-support 计数端口，无行为 | `9ed5f345325710646e4ec768ff16f91842885123619e32fbbb664bf59a3590d1` |
| `src/lib.rs` | 只增加模块声明（`pub mod` / `mod` 两分支 cfg） | 见 `git diff src/lib.rs` |
| `src/facade.rs` | 只增加计数调用与注释，无行为改动 | 合并 diff `c4e47b8efd57610d67efe99a1764463c354d441c7226a3463eecc7b25905ade8` |
| `tests/d0_demand_counts.rs`（新增） | D0 计数门禁 | `15c6de6af46e6273b46ac549f435f593560397bab538d66d11ca2fa424940843` |

被明确排除的三个并行流的文件（`crates/jarde-query/**`、`crates/jarde-reader/src/{classfile,prepared,artifact,scope_cursor,ledger,budget,facts_cache}.rs`、`crates/jarde-jvm/src/{frame,lambda,engine,method_ir,providers,callee}.rs`、`crates/jarde-java/**`、`src/class_source.rs`、`src/bulk.rs`、`crates/jarde-cli/**`、`Cargo.toml`）本轮**一个字节都没有改动**（见 §8 的 `git status --porcelain` 复核）。

## 2. 现有“完整证据”的源码入口（1.1）

冻结 revision 上，一次恢复的“完整证据”没有任何开关，全部由同一条路径固定构造：

| 证据类别（design §4） | 构造入口 | 进入报告的位置 |
| --- | --- | --- |
| 正文与语义平面 | `jarde_java::recover`（`crates/jarde-java/src/report.rs`），先 `region::recover` 建 `Recovered`，再 `build::build` 建语句、`emit::emit` 写正文 | `RecoveryReport::{text, representation, quality, syntax_status, content}` |
| SourceMap（完整段表） | `crates/jarde-java/src/emit.rs` 的 `Emitter::segments` → `Emitted::source_map` | `RecoveryReport::source_map` |
| RegionDetails | `report::region_records`（`Region` → `RegionRecord`） | `RecoveryReport::regions` |
| RuleDetails（各规则记录） | `concat::Plan::records`、`bridge::Plan::record`、`init::Sites::records`、`init::Prologues::record`、`field::Plan::records`、`enumswitch::Plan::records`、`declaration::Plan::record`、`build::Program::{lambdas,accessors}` | `RecoveryReport::{lambdas,concats,accessors,bridges,news,fields,enum_switches,init,declaration}` |
| NameDetails | `names::NameTable` 的别名集合 | `RecoveryReport::aliased_names` |
| ReadDetails | `jarde_jvm::callee::read_callees` / `read_prepared_callees`（facade 的 `read_named_callees` / `read_prepared_named_callees`） | `RecoveredMethod::callees`（`CalleeReadReport`） |
| 完整 analysis report | `jarde_jvm::engine::analyze_method` / `analyze_prepared_method` | `RecoveredMethod::analysis`（`MethodAnalysisReport`），与 `MethodIr` 载荷分开 |

组合入口（都是“一次请求 = 完整证据”）：`Engine::recover_method`、`Engine::recover_target`、`Engine::class_view`（按需 body）、`Engine::class_source`（每个成员一次完整恢复）；`Engine::analyze_method`/`analyze_target` 只跑分析。CLI 侧 `recover`/`class-view`/`class-source` 不传递任何证据选择。

## 3. query 顺序基线的源码入口（1.1）

| 基线 | 源码入口 |
| --- | --- |
| 容器/entry 物理顺序、consumer 固定子顺序、consumer 内位置顺序 | `crates/jarde-query/src/xref/mod.rs::scan_units`（`'units` 循环：`provider.units` 顺序 → `scan_unit` 内的固定 consumer 顺序） |
| 先收集范围再分页 | `crates/jarde-query/src/xref/mod.rs::ProviderScan::collect`（`units` 一次性收集）+ `scan_units` 的 `unit_items`（整个 unit 的结果先构造，再按 `max_items` 发布） |
| consumer 实现 | `xref/{code,metadata,bootstrap,resource}.rs::scan`，统一入口 `xref::scan` |
| 页/游标 | `QueryPage{has_more,returned_items,cursor}`、`QueryBoundary{container,ordinal,item_index}`（`crates/jarde-query/src/query.rs`）；续页按 `item_index` 重放 unit 前缀 |
| 逐页覆盖 | `QueryCoverage::scanned_items`（本次调用实际产出/重放的项数）、`usage.{archive_entries,read_bytes,class_bytes,code_bytes,result_items}` |
| 入口 | `src/facade.rs::Engine::query`（一行委托 `jarde_query::query::execute`） |

## 4. 开放缺口清单（1.1）

以下条目在本 change 之外独立登记，**本轮不修、不标已修**（design §15 与 proposal 的写法保持）：

| 缺口 | 现状证据 | 修复所有权 |
| --- | --- | --- |
| 数组参数槽宽 | §5.1 本轮复跑：`long[]` 参数槽位按元素宽度 2 计算 | 独立修复；`src/class_source.rs::descriptor_type` + reader 结构 descriptor 事实 |
| `append(int)` 消费 `char`（T5） | §5.2 本轮复跑：`65!` → `A!`、`x65` → `xA` | 独立修复；`crates/jarde-java/src/build.rs::concat_expr` + `emit.rs` |
| 活动容器跨 worker 交接 | `openspec/completion-review.md` 登记；本轮未复核 bulk 队列 | `add-parallel-bulk-recovery`（bulk 流） |
| 精确局部/跨方法数据流查询 | design §15：内部 SSA 不等于查询产品 | 明确延期，另立查询契约 |
| >255 维、任意深表达式、通用可赋值性、CLI 文档停止额度 | 既有已记录边界 | 各自独立 change / 原边界 |

D0 自身的开放项（本 change 内，属于 D1–D4 的工作，不在此标为已修）：`ProviderScan::collect` 的整范围收集与 `unit_items` 的整体构造、`scan_units` 续页按 `item_index` 重放、`RecoveryReport` 固定构造全部可选表、普通组合入口的重复准备。
**本文件只立门禁**：§7 给出这些缺口的可失败观测值。

## 5. 反例复跑（1.1）

### 5.1 数组参数槽宽（本轮真实复跑）

自造 fixture：`javac 23.0.1`（`/usr/bin/javac`），命令与产物：

```text
$ cat ArraySlot.java
public class ArraySlot {
    public static int f(long[] xs, int n) {
        return n;
    }
}

$ javac --release 8 -g:none -d out ArraySlot.java
（javac 打印 3 个「源值 8 已过时 / 目标值 8 已过时」警告，仍然写出 class）

$ shasum -a 256 out/ArraySlot.class
30ca19988ee500b29762223b9c0c4be9bffa5ac2df6239c95167b8a9bbb902f7   （165 字节）

$ javap -p -c out/ArraySlot.class
public static int f(long[], int);
  Code:
     0: iload_1            // 参数 n 在槽 1（数组是引用，只占一个槽）
     1: ireturn

$ target/debug/jarde-cli class-source --input out/ArraySlot.class --policy single-class --class ArraySlot --format text
    public static int f(long[] arg0, int arg2) {
        // @method f([JI)I
        // @declaration a static method of `ArraySlot`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return arg1;
    }
```

产物签名把 `int n` 放在 **arg2**（`[J` 按元素宽度 2 记槽），正文按槽 1 写 **arg1**，两者不一致。把产物交给 `javac`：

```text
$ javac --release 8 -g:none -d jardeout ArraySlot.java      # 去掉两条 // jarde: 头部注释后的产物
ArraySlot.java:14: 错误: 找不到符号
        return arg1;
               ^
  符号:   变量 arg1
  位置: 类 ArraySlot
1 个错误
```

对照（`int[]` 参数，同一路径）：`public static int g(int[] arg0, int arg1)` + `return arg1;`，槽位正确。结论与 design §15 的一致：根因在**数组沿用元素宽度**，`[J/[D/[[J` 都被记成 2 槽。
本轮**不修**此缺口：`src/class_source.rs` 属于并行流冻结面。

### 5.2 `append(int)` 消费 `char`（T5，本轮真实复跑）

fixture 与既有 T5 记录的同形源码（`javac --release 8 -g:none`，原类与恢复类都可编译）：

```text
public class ConcatChar {
    public static String value(char c) {
        return new StringBuilder().append((int) c).append("!").toString();
    }
    public static String prefixed(char c) {
        return new StringBuilder().append("x").append((int) c).toString();
    }
}
```

```text
$ java -cp out Runner            # 原类执行
65!
x65

$ target/debug/jarde-cli class-source --input out/ConcatChar.class --policy single-class --class ConcatChar --format text
    public static java.lang.String value(char arg0) {
        return "" + arg0 + "!";
    }
    public static java.lang.String prefixed(char arg0) {
        return "x" + arg0;
    }

$ javac --release 8 -g:none -cp recovered -d recovered/out recovered/ConcatChar.java Runner.java   # 恢复产物
$ java -cp recovered/out Runner  # 恢复类执行
A!
xA
```

原值 `65!` / `x65`，恢复值 `A!` / `xA`——**两边都可编译而值不同**。与既有记录的差异：既有记录给的是 6 行差异表与脚本；本轮是直接复跑，观察到同样的两行产物与同样的执行差异。
代码锚点（只读核对，未改动）：`crates/jarde-java/src/build.rs::concat_expr` 只对 Boolean 做转换，其余片段按原样消费；`crates/jarde-java/src/emit.rs` 只按首段是否 String 决定空串前缀。本轮**不修**此缺口（`crates/jarde-java/**` 属于并行流冻结面），T5 的独立修复所有权不变。

## 6. 字段角色清点（1.2，D01/D04/D11）

### 6.1 四种角色

| 角色 | 含义 | 判定问题 |
| --- | --- | --- |
| **算法输入** | 算法读取的输入，不是本次运行的产品 | 换一次请求它会变吗？换调用方它必须变吗？ |
| **内部计划** | 为产生产物所必需、随本次运行释放的内部结构；不是报告字段 | 没有它能否决定同样的产物？ |
| **必要结果** | 任何证据选择下都必须交付：定位身份、实际答案、独立平面、覆盖/执行、核心缺口 | 缺了它，答案还能被读懂/核对吗？ |
| **可选明细** | 只在显式选择对应证据时物化；未选择时不构造、不复制、不保留 | 只有审计/追问才需要它吗？ |

### 6.2 `RecoveryReport`（27 字段，`crates/jarde-java/src/report.rs`）

拥有者：`jarde-java`（`report.rs`，一次运行一个报告）；生命周期：随 `RecoveredMethod` / `ClassSourceOutcome::Recovered` 交给调用方，运行内构建的全部内部计划在 `recover()` 返回前释放；计费：正文/结构计 `OutputBytes`/`IrItems`，区域遍历计 `AnalysisSteps`，报告边界上的每个已发布项由 facade 计 `ResultItems`，序列化计 `OutputBytes`。

| 字段 | 角色 | 备注（拥有者内的一处说明） |
| --- | --- | --- |
| `method` | 必要结果 | 报告的目标身份 |
| `profile` | **算法输入** | 请求的策略输入，被回显；它决定哪些 pass 被承认 |
| `rules` | 可选明细 | 由 `Recovered::rules()` 汇总，RuleDetails 的索引 |
| `representation` | 必要结果 | 独立平面 |
| `quality` | 必要结果 | 独立平面 |
| `syntax_status` | 必要结果 | 独立平面（别名事实留在这一位，不靠 `aliased_names`） |
| `compile_status` | 必要结果 | 独立平面（本 slice 恒 `NotAttempted`） |
| `semantic_validation` | 必要结果 | 独立平面 |
| `verification` | 必要结果 | 独立平面 |
| `execution` | 必要结果 | 真实停止/用量 |
| `outcome` | 必要结果 | `Produced` / `Stopped(reason)` |
| `content` | 必要结果 | 从提交结构读出的内容分类 |
| `text` | 必要结果 | 已提交正文（唯一的正文副本） |
| `source_map` | 可选明细 | SourceMap 全文段表；未选时不构造 `Vec<Segment>` |
| `regions` | 可选明细 | RegionDetails 的公开记录（`RegionRecord`）；核心缺口经 `diagnostics`/`fallbacks` 保留 |
| `lambdas` | 可选明细 | RuleDetails（`lambda@1`） |
| `concats` | 可选明细 | RuleDetails（`concat@1`） |
| `accessors` | 可选明细 | RuleDetails（`accessor@1`） |
| `bridges` | 可选明细 | RuleDetails（`bridge@1`） |
| `news` | 可选明细 | RuleDetails（`new@1`） |
| `fields` | 可选明细 | RuleDetails（`field@1`） |
| `enum_switches` | 可选明细 | RuleDetails（`enumswitch@1`） |
| `init` | 可选明细 | RuleDetails（`init@1`） |
| `declaration` | 可选明细 | `declaration@1` 的公开记录；正文 envelope 由内部计划 `declaration::Declaration` 写，不依赖这条记录，故关闭它不损失正文 |
| `fallbacks` | 必要结果 | 拒绝码（不是完整理由链） |
| `aliased_names` | 可选明细 | NameDetails；非法名字这一事实已由 `syntax_status` 保留 |
| `diagnostics` | 必要结果 | 核心缺口与真实停止的现有词汇 |

小计：算法输入 **1** / 内部计划 **0** / 必要结果 **13** / 可选明细 **13**。

### 6.3 规则计划与区域计划（内部计划族）

拥有者：`jarde-java` 各规则模块；生命周期：`RecoveryReport` 构建期间局部存在（`recover()` 的局部绑定），返回前释放；计费：区域遍历 `AnalysisSteps`，语句构建 `IrItems`，解码 `MethodBodies`/`CodeBytes`/`AttributeBytes`（读侧），规则判定本身不单列维度。

| 结构（文件） | 字段数 | 算法输入 | 内部计划 | 必要结果 | 可选明细 |
| --- | --- | --- | --- | --- | --- |
| `concat::Chain` | 5 | 0 | 5 | 0 | 0 |
| `concat::Plan` | 3 | 0 | 2 | 0 | 1（`records`） |
| `bridge::Plan` | 2 | 0 | 1 | 0 | 1（`record`） |
| `init::Site` | 6 | 0 | 6 | 0 | 0 |
| `init::Sites` | 3 | 0 | 2 | 0 | 1（`records`） |
| `init::Prologue` | 2 | 0 | 2 | 0 | 0 |
| `init::Prologues` | 3 | 0 | 2 | 0 | 1（`record`） |
| `field::Plan` | 2 | 0 | 1 | 0 | 1（`records`） |
| `field::Evidence` | 6 | 0 | 6 | 0 | 0 |
| `field::Shape` | 2 | 0 | 2 | 0 | 0 |
| `enumswitch::Plan` | 2 | 0 | 1 | 0 | 1（`records`） |
| `declaration::Plan` | 2 | 0 | 1 | 0 | 1（`record`） |
| `declaration::Declaration` | 4 | 0 | 4 | 0 | 0 |
| `lambda::Plan` | 5 | 0 | 5 | 0 | 0 |
| `lambda::Member` | 4 | 0 | 4 | 0 | 0 |
| `lambda::Evidence` | 5 | 0 | 5 | 0 | 0 |
| `lambda::Verdict` | 2 | 0 | 2 | 0 | 0 |
| `guard::Plan` | 6 | 0 | 6 | 0 | 0 |
| `normal_flow::NormalFlowView` | 9 | 0 | 9 | 0 | 0 |
| `region::Recovered` | 3 | 0 | 3 | 0 | 0 |
| `region::Region`（6 变体载荷） | 23 | 0 | 23 | 0 | 0 |
| `build::Program` | 5 | 0 | 3 | 0 | 2（`lambdas`,`accessors`） |
| `names::NameTable` | 3 | 0 | 3 | 0 | 0 |
| `reuse::Plan` | 2 | 0 | 2 | 0 | 0 |
| `decode::Operations` | 1 | 0 | 1 | 0 | 0 |
| **合计** | **110** | **0** | **101** | **0** | **9** |

被计划持有、随后发布的记录类型（RuleDetails/RegionDetails 的实际拥有型记录；逐字段均为**可选明细**）：

| 记录 | 字段数 | 发布位置 |
| --- | --- | --- |
| `RegionRecord` | 6 | `RecoveryReport::regions` |
| `ConcatRecord` | 6 | `RecoveryReport::concats` |
| `BridgeRecord` | 4 | `RecoveryReport::bridges` |
| `NewRecord` | 7 | `RecoveryReport::news` |
| `InitRecord` | 6 | `RecoveryReport::init` |
| `FieldRecord` | 8 | `RecoveryReport::fields` |
| `EnumSwitchRecord` | 5 | `RecoveryReport::enum_switches` |
| `DeclarationRecord` | 7 | `RecoveryReport::declaration` |
| `LambdaRecord` | 13 | `RecoveryReport::lambdas` |
| `AccessorRecord` | 10 | `RecoveryReport::accessors` |
| **合计** | **72** | 全部为可选明细 |

### 6.4 emitter（`crates/jarde-java/src/emit.rs`、`source_map.rs`）

拥有者：`jarde-java`；生命周期：`Emitter` 只在 `emit()` 帧内，`Emitted` 的值按字段并入报告（`text`/`source_map`）后释放；计费：每次写入前计 `OutputBytes`（`written` 是已写字节数）。

| 结构 | 字段数 | 算法输入 | 内部计划 | 必要结果 | 可选明细 |
| --- | --- | --- | --- | --- | --- |
| `emit::Emitter` | 7 | 0 | 7 | 0 | 0 |
| `emit::Emitted` | 4 | 0 | 0 | 3（`text`,`written`,`statements`） | 1（`source_map`） |
| `source_map::SourceMap` | 1 | 0 | 0 | 0 | 1（`segments`） |
| `source_map::Segment` | 3 | 0 | 0 | 0 | 3 |
| `source_map::OriginSet` | 2 | 0 | 2 | 0 | 0 |
| **合计** | **17** | **0** | **9** | **3** | **5** |

`OriginSet`（`primary`/`derived`）是**内部计划**：来源必须参与恢复与核心位置；被物化的是 `Segment` 表（可选明细）。这与 design §4「origin | 内部恢复及核心位置保留 | SourceMap 给出全文或局部文本段映射」一致。

### 6.5 analysis / callee 报告

拥有者：`jarde-jvm`（`ir.rs`、`resolver.rs`、`callee.rs`、`method_ir.rs`）；生命周期：`MethodAnalysisReport`/`MethodIr` 随 `MethodIrAnalysis` 交给 facade，`CalleeReadReport` 在 `RecoveredMethod::callees` 中与恢复报告同寿命；计费：类读取 `ClassHeaders`/`ClassBytes`/`AttributeBytes`，body 解码 `MethodBodies`/`CodeBytes`，图与 pass `IrItems`/`IrEdges`/`AnalysisSteps`，发布项 `ResultItems`。

| 结构 | 字段数 | 算法输入 | 内部计划 | 必要结果 | 可选明细 |
| --- | --- | --- | --- | --- | --- |
| `ir::MethodAnalysisReport` | 18 | 0 | 0 | 17 | 1（`origin`） |
| `ir::StageResult` | 2 | 0 | 0 | 2 | 0 |
| `resolver::HeaderRead` | 3 | 0 | 0 | 3 | 0 |
| `callee::CalleeReadReport` | 5 | 0 | 0 | 5 | 0 |
| `callee::CalleeMember` | 3 | 0 | 0 | 3 | 0 |
| `callee::CalleeBody` | 3 | 0 | 1（`facts`） | 2 | 0 |
| `callee::CalleeRefusal` | 3 | 0 | 0 | 3 | 0 |
| `callee::CalleeCandidate` | 4 | 4 | 0 | 0 | 0 |
| `method_ir::MethodIr` | 7 | 2（`facts`,`bootstrap_methods`） | 5 | 0 | 0 |
| `method_ir::MethodDeclaration` | 7 | 7 | 0 | 0 | 0 |
| **合计** | **55** | **13** | **6** | **35** | **1** |

输入边界（本 change 不产出、只读取）：`RecoveryRequest` 4、`RecoveryFacts` 2、`facts::MethodFacts` 5、`facts::ClassMembers` 2、`ir::MethodAnalysisRequest` 3 —— 共 **16** 字段，全部为**算法输入**。

### 6.6 统计与结论

| 家族 | 字段数 | 算法输入 | 内部计划 | 必要结果 | 可选明细 |
| --- | --- | --- | --- | --- | --- |
| `RecoveryReport` | 27 | 1 | 0 | 13 | 13 |
| 规则/区域计划 | 110 | 0 | 101 | 0 | 9 |
| 发布的规则/区域记录 | 72 | 0 | 0 | 0 | 72 |
| emitter 与 source map | 17 | 0 | 9 | 3 | 5 |
| analysis / callee 报告 | 55 | 13 | 6 | 35 | 1 |
| 输入边界 | 16 | 16 | 0 | 0 | 0 |
| **合计** | **297** | **30** | **116** | **51** | **100** |

**内部 CFG/SSA 本已与报告分离，不重建第二套模型**（D01/D11 的复核结论）：

* `MethodAnalysisReport` 的 18 个字段里没有 CFG、帧、SSA、图或块表；它只发布请求阶段、plane、读取记录、覆盖、执行与诊断；
* 图与表在 `jarde_jvm::method_ir::MethodIr`（`canonical`/`frames`/`ssa`/`code` 各一个 `Option<Box<…>>`，加上只读句柄 `facts: Arc<ClassFacts>` 与 `bootstrap_methods`/`declaration`），由 `recover()` 通过 `RecoveryRequest::ir` 借用；`MethodIr::new` 的 `debug_assert!` 固定了“图 ⇒ 解码 ⇒ 同一读取句柄”的伴随关系；
* `RecoveryReport` 的 27 个字段里同样没有图或表：报告只有正文、平面、记录与诊断；
* 因此 D1/D3 的“证据选择”只需在同一 `MethodIr` 上选择**物化哪些对外记录**，不需要、也不得新建第二套 IR/AST 模型；本 change 未新增任何模型类型。

## 7. 计数见证（1.3，D01/D02/D04/D08/D11）

### 7.1 端口与钩子

新增 test-support 端口 `src/d0_counts.rs`（`src/lib.rs` 中 `#[cfg(any(test, feature = "test-support"))] pub mod d0_counts;`，普通构建里是私有无读数模块）。端口是**有界的**：不论运行多久都只有 5 个 `u64`，不保存任何被计数的工作记录；它不在任何报告、停止记录或 fingerprint 里（文件内没有 `Serialize`，也没有任何报告类型引用它）。

| 计数 | 一次自增 = | 钩子站点（`src/facade.rs`） |
| --- | --- | --- |
| `class_materializations` | 选定定义被物化为可信 read 一次 | `bind_class`（身份分支）与 `read_prepared_definition` |
| `class_preparations` | 一次 `PreparedClass::prepare` | `class_source` 的准备处（每请求一次） |
| `body_decodes` | 需求路径上解码一个方法体 | `body_result`（class_view）、`recover_method`、`recover_prepared_member`（后两者在运行确实发布了解码时计） |
| `recovery_runs` | 一次 Java 恢复呈现 | `recovery_presented` |
| `owned_records` | 本层发布构造的拥有型记录 | `body_result`（每条指令/处理器记录）与 `recovery_presented`（analysis report 副本及其 stage/read/diagnostic 记录） |

### 7.2 五项见证的实际方式与冻结观测值

| 见证项 | 见证方式 | 冻结 revision 的实测值 |
| --- | --- | --- |
| class 物化/准备 | 端口计数 + `usage.class_headers` | 成员只读 `list_members`：0/0，`class_headers=1`；`class_view`（身份，0 body）：1/0，`class_headers=1`；`class_view`（1 body）：1/0，`class_headers=1`；`class_source`（身份，8 成员）：**2/1，`class_headers=2`**（D02 的基线） |
| body 解码 | 端口计数 + `usage.method_bodies`/`code_bytes` | 成员只读：0，`method_bodies=0`、`code_bytes=0`；`class_view`（1 body）：1，`method_bodies=1`、`code_bytes=4`；`class_source`：8，`method_bodies=8`、`code_bytes=72` |
| consumer work | **测试侧公开面**：`QueryCoverage::scanned_items` + `usage.{archive_entries,read_bytes,code_bytes}` + `QueryPage` | 池探针（整范围）：items=2、scanned=2、entries=5、read_bytes=723；页=1：items=1、scanned=1、entries=3、read_bytes=532、`has_more=true`。调用消费者（`MentionsSymbol`/`Invocation`）：单 unit scope 的页=1 与整范围 `code_bytes` 相等（=72，即**整 unit 的全部正文**都被解码）；双 unit scope 的页=1 只读到第一个 unit（read_bytes=532 < 723）。`scanned_items` 只计**已发布/重放**项，故“unit 内构造但未发布”的部分不可由现有公开面观察（见 §7.4） |
| 可选拥有型记录 | **测试侧公开面**：报告的 `regions/lambdas/concats/accessors/bridges/news/fields/enum_switches/init/source_map.segments/rules/fallbacks/aliased_names/diagnostics` 计数；facade 侧另有端口 `owned_records` | `class_source` 的 8 份 `RecoveryReport` 共发布 **109** 条：regions=11、segments=60、rules=20、diagnostics=17、init=1，其余类别为 0；`class_view`（1 body）的 facade 侧 `owned_records=7`、`class_source`=64；成员只读=0 |
| （可观察的）释放 | **测试侧公开面**：`Arc::strong_count(PreparedClass::facts_handle())` | 准备与观察者两个持有者时 =2；`drop(prepared)` 后 =1；`drop(read)` 后仍 =1（读取不持有该句柄）；最后一个句柄由 `prepared` 交出并由测试释放。观察值见测试输出 `release: facts strong_count after drop(prepared) = 1` |

`jarde::d0_counts::Counts` 提供 `since()` 差值读数，门禁因此不依赖“进程内其他测试没有计数”，但同一测试二进制内的计数测试仍用一把 `Mutex` 串行执行（端口是全局的）。

### 7.3 人为变异（真实执行，随后还原）

**变异 M1「同一类重复构造两次」**：在 `class_source` 的准备处多调用一次 `read_prepared_definition`（读入结果丢弃）。

```text
class_source: class_materializations=3 class_preparations=1 body_decodes=8 recovery_runs=8 owned_records=64 | class_headers=3 method_bodies=8 ...
thread 'a_class_source_request_frozen_baseline_two_materializations_one_preparation' panicked at tests/d0_demand_counts.rs:376:5:
assertion `left == right` failed
  left: 3
 right: 2
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 4 filtered out; finished in 0.00s
```

即 `class_materializations` 2→3、`class_headers` 2→3，门禁在第一条冻结断言处变红。

**变异 M2「先全量构造再过滤」**：在 `class_view` 里先把该类的**每个**成员体都 `body_result` 一遍，再照常解析请求的 body。

```text
class_view (one body): class_materializations=1 class_preparations=0 body_decodes=9 recovery_runs=0 owned_records=89 | class_headers=1 method_bodies=9 code_bytes=76 ...
thread 'one_requested_body_decodes_one_body_and_runs_no_recovery' panicked at tests/d0_demand_counts.rs:341:5:
assertion `left == right` failed
  left: 9
 right: 1
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 4 filtered out; finished in 0.00s
```

即 `body_decodes` 1→9、`method_bodies` 1→9、`owned_records` 7→89。

**还原方式**：两处变异都是就地编辑（`src/facade.rs`），还原为原始单次调用/删除探测循环；还原后 `grep -n MUTATION src tests` 无匹配、`git diff --stat src/` 只剩钩子改动，`cargo test --test d0_demand_counts --all-features --locked` 回到 **5 passed**。（上面两段 panic 行号是变异当时的行号；测试文件其后只删掉了一条恒真的断言与一个未读的静态计数器，断言本身没有放宽。）

### 7.4 未接端口的事件（因文件冻结改为测试侧统计）

| 事件 | 事件所在 | 本轮见证方式 | 后续需要的 hook 位置 |
| --- | --- | --- | --- |
| consumer work | `crates/jarde-query/**`（三个流的冻结面） | 公开面：`scanned_items`、`usage`、`QueryPage` | D4 在 `scan_unit`/consumer 的逐项产出处计数；否则“整个 unit 构造后再截断”不可观察 |
| 恢复报告的可选拥有型记录**构造** | `crates/jarde-java/**`（冻结面） | 公开面：发布计数（下界）+ facade 侧 `owned_records` 证明“构造后丢弃”可被计数捕获 | D3 在记录构造点（`report.rs`/各规则模块的 record 物化处）计数 |
| 直接 `recover_method`/`recover_target` 的类读取 | `crates/jarde-jvm/src/engine.rs`（冻结面） | 公开面：`usage.class_headers` | D2 若要覆盖该路径，需在引擎的读取处或改用同一 prepared 交接后计数 |
| 名字选择（`ClassRef::Name`）的候选搜索读取 | `src/facade.rs::search_named_classes` | 公开面：`usage.class_headers`（搜索费用独立计费，不并入 `class_materializations`） | 保持现状即可，语义已写明 |

## 8. 本轮命令与真实结果

| 命令 | 结果 |
| --- | --- |
| `cargo fmt --all` | 无输出（无格式变更；`git diff --stat` 中 Rust 文件仅 `src/facade.rs`、`src/lib.rs`） |
| `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` | `Finished`，**0 warning**（首次全量 8.95s，其后增量 0.54–3.04s） |
| `cargo test --test d0_demand_counts --all-features --locked` | `5 passed; 0 failed` |
| 变异 M1 / M2 | 各 1 failed（见 §7.3），随后还原并回到 5 passed |
| `cargo check --workspace --locked`（不带 feature） | `Finished dev profile ... in 6.23s`（端口在普通构建里无读数 API） |
| `cargo test --workspace --all-targets --all-features --locked` | 见 §8.1 |
| `openspec validate --all --strict --no-interactive` | **22 passed**；另有 1 项失败来自并行流的 `openspec/changes/fix-descriptor-slot-facts/`（未跟踪目录、本轮未触碰、缺 delta），本 change 单项 `openspec validate add-demand-driven-core-results --type change` = `Change 'add-demand-driven-core-results' is valid` |

本轮结束时的 dirty 复核（源码侧只有本 change 的四个文件，其余与 §1 的冻结集合一致）：

```text
$ git status --porcelain=v1 | grep -E "\.rs$|Cargo"
 M src/facade.rs
 M src/lib.rs
?? src/d0_counts.rs
?? tests/d0_demand_counts.rs
```

### 8.1 全 workspace 测试

四次全量运行的原始结果（同一源码；`tee` 到日志后汇总每行 `test result:`）：

| 运行 | 目标数 | passed | failed | ignored | 结果 |
| --- | --- | --- | --- | --- | --- |
| 第 1 次（修好 bulk 入口守卫后） | 96 | **1483** | 0 | 10 | 全绿 |
| 第 2 次 | 19（在失败目标处中止） | 163 | 1 | 1 | `p1_multi_release::physical_evidence_is_the_single_provider_result_it_reports` |
| 第 3 次 | 68（在失败目标处中止） | 738 | 1 | 4 | `p4_plugins::the_plugin_plane_leaves_the_structural_planes_own_answer_untouched` |
| 第 4 次（最终源码） | 96 | **1483** | 0 | 10 | 全绿 |

第 4 次跑的就是本节记录的四个文件哈希所在的源码；本节引用的 `d0_demand_counts` 与 `p5_corpus_fingerprint` 结果行也来自该次运行（第 1 次同样全绿，两次的汇总计数一致）。

第 2、3 次的失败是**既有测试自身的计时 flaky**，与本次改动无关，判据是失败比较的完整报告**只差 `elapsed_millis`**：

* `p1_multi_release` 的那条断言把 `select(...)` 内部枚举出来的 `ArtifactTreeReport` 与随后一次 `snapshot.enumerate_artifact_tree(...)` 的报告整体相等比较，两次运行的 `execution.usage.elapsed_millis` 分别是 0 与 2；
* `p4_plugins` 的那条断言把两次 `QueryReport` 序列化后整体相等比较，两次的 `elapsed_millis` 分别是 0 与 1（该比较没有像 benchmark 协议那样排除 `elapsed_millis`）。

两条测试单独运行 6/6 通过（`for i in 1..6; cargo test --test p1_multi_release --all-features --locked physical_evidence_...`），且它们走的路径（`snapshot.enumerate_artifact_tree`、`Engine::query` → `jarde_query::query::execute`）**不经过任何计数钩子**（钩子只在 `bind_class`、`read_prepared_definition`、`ClassSource` 的准备处、`body_result`、`recover_method`、`recover_prepared_member`、`recovery_presented`）。机器上同时有其它三条流在跑 cargo，负载正是这两次失败出现的条件；它们在本文件之外登记为既有 flaky，不由本 change 修复。

```text
$ cargo test --workspace --all-targets --all-features --locked     # 第 4 次
（96 个目标的 "test result:" 行汇总）
96 targets: 1483 passed; 0 failed; 10 ignored; 0 measured
non-ok targets: []

其中本 change 的目标：
     Running tests/d0_demand_counts.rs (target/debug/deps/d0_demand_counts-5403b6d3ad5e9909)
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

    Running tests/p5_corpus_fingerprint.rs (...)
    corpus_files_match_the_recorded_fingerprint ... ok
test result: ok. 5 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.03s
```

语料 fingerprint 目标（`tests/p5_corpus_fingerprint.rs`）在加入计数端口后仍通过，即**仪表没有进入领域 fingerprint**：端口不参与 `tests/fixtures/corpus-fingerprint.json` 的登记，也不出现在任何被指纹比较的报告中。

第一次跑全量时 `tests/p5_benchmark.rs::no_ordinary_entry_reaches_the_bulk_module_or_recover_all` 曾失败（新文件的文档里出现了 `crate::bulk` 字样，被守卫当成了 bulk 模块的第二个入口）；改写那句文档后该守卫通过。守卫本身保留且未被放宽。

## 9. 与 tasks 原文的偏差、未完成项

* **端口位置**：tasks 1.3 写的是 `crates/*/src/*` 内的计数口或测试内 harness。本轮的端口放在**根 crate**（新增 `src/d0_counts.rs`，`src/lib.rs` 一行声明，`src/facade.rs` 内只加计数调用），因为被计数的事件就是 facade 自己组合出来的需求路径（bulk 的同类端口 `src/bulk/observation.rs` 也在根 crate）。三个并行流的文件无一被改动；未改 `Cargo.toml`（模块声明不需要 feature 变更）。
* **诚实标注为下界的见证**：三项（consumer work、可选记录构造、release）本轮用公开面统计；其中“可选记录构造”只能看到**已发布**的记录，构造后丢弃的记录不可见（§7.4）。D3 的 hook 必须在 `jarde-java` 内放置，才满足 D04 的“构造完整表再截断必须失败”。
* **未完成**：D1–D4 的全部实现（1.2 的清点只是清单，不改语义；3.x/4.x/6.x 未开始）；`openspec/changes/add-demand-driven-core-results/tasks.md` 只勾选 1.1–1.3。
* **未复跑项**：活动容器跨 worker 交接本轮未复核（属 bulk 流）；T5 的 6 行差异表沿用既有记录，本轮只复跑其中两行（`'A'`）。

## D3（任务 4.1–4.6）验证记录（由主 Agent 复跑）

该阶段的实现 agent 在写报告时被宿主机中断（其工作树代码完整、可编译）；以下证据由主 Agent 在其最终文件状态上复跑：

| 命令 | 结果 |
| --- | --- |
| `cargo test --workspace --all-targets --all-features --locked` | **1535 passed / 0 failed / 11 ignored** |
| `cargo test --test p3_execution_comparison --locked -- --ignored` | 3 passed（受控 JDK 编译执行对照） |
| `cargo test -p jarde-java --locked` | 77 / 32 / 43 passed |
| `cargo test --test d1_evidence_selection --all-features --locked` | 8 passed |

接缝：`src/facade.rs` 的 `publish_read_details`（选中且产出才发布读取明细）、`crates/jarde-java/src/emit.rs:393`（仅选中 `SourceMap` 时记录 span）、`src/bulk.rs::result_weight`（按选择核算拥有字节，未选不计）。未做：agent 侧的两条变异反例没有留下原始输出，本记录不把它们写成已验证；D3 的"局部=完整投影"由 `the_driver_range_selects_the_records_it_intersects` 在 region 记录上给出。

## D3'（任务 5.1–5.3）验证记录

| 命令 | 结果 |
| --- | --- |
| `cargo test --workspace --all-targets --all-features --locked` | **1552 passed / 0 failed / 11 ignored**（103 targets） |
| `cargo test --test p3_execution_comparison --locked -- --ignored` | 3 passed |
| `cargo test --test d3_artifact_binding --all-features --locked` | 13 passed |
| `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` | 干净（合并时修掉 3 处 `useless_format`、1 处 `too_many_arguments`（按 reader crate 既有惯例显式 allow）、1 处 `large_enum_variant`（CLI 的 `Operation` 改为 Box 该字段）） |

**集成期修正（由主 Agent 记录）**：`tests/bulk_recovery_retention.rs` 原断言"无 store 的运行会解析更多目录"在 D2 之后不再恒真（成员绑定改由 prepared 类定位，无主的目录查询只剩少数且依赖取类顺序）；改为断言契约本身——**保留绝不会比不保留解析更多**（`zero ≥ roomy`）且 records 逐项相同，并注明该计数与顺序有关。反例（取消准备交接）仍由 `d2_prepared_handoff` 与 `bulk_recovery_serial` 承担。

**未覆盖**：CLI 未加 `expected_artifact` 旗标（报告字段随 schema 传播）；绑定上无可数 `Arc`，释放见证以结构性事实（无法持 IR/AST 编译通过）与逐请求重建计数替代；一次全量运行中 `export_cli` 偶发红，单跑与复跑均绿，按既有 flake 记录。
