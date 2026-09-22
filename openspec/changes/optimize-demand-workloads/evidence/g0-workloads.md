# G0 1.2：W1–W5 固定工作负载与 W6a 1/2/4/6 worker 序列

## 1. 放在哪，为什么

harness 是**新的测试 target** `tests/p5_optimize_workloads.rs`（本专项新增），
由 `evidence/run-baseline.py` 每个样本一个新进程地驱动。理由：

- 任务要求"不依赖临时 benchmark 目录（仓库内 fixture 或测试内确定性构造）"。本仓库的确定性
  fixture 约定就是"内存里用 `rawzip` dev-dep 写 ZIP + 已提交 `.class` 字节"（`tests/bulk_support.rs`、
  `tests/p5_bulk_corpus.rs` 已如此）；测试 target 能同时用 `rawzip` 与 `bulk_support`，示例不行
  （示例看不到 `tests/` 模块，要么自带第二份 fixture 构造器）。
- 每个样本仍是**独立进程**：campaign 脚本按 cargo 自己的 `--message-format=json` 取到可执行文件，
  直接调用它（`--exact measure_workload --ignored --nocapture`），不像素级地经过 `cargo test`
  的外层启动；W1 与 W6a 的"冷进程"因此是进程事实而不是标签。
- 写盘模式写 **cargo 自己的** `CARGO_TARGET_TMPDIR`（仓库 `target/` 内），不用 `/tmp`；`write`
  与 harness 都不创建临时 benchmark 目录。

```text
verify:  cargo test --test p5_optimize_workloads --locked [--features test-support]
measure: python3 evidence/run-baseline.py --tag <label> --repeats 10 --out <raw.jsonl> \
             [--features test-support] --run <workload>[:key=value...] ...
```

## 2. 确定性 fixture（仓库内，174 行字节到 24 类）

`optimize_fixture()` 在内存里构造一个 ZIP：根目录 5 个类 + 一个 STORED 嵌套容器 `lib/more.jar`
（其内 19 个类，前缀 `p/`），全部是已提交的 `.class` 字节。**不新增任何二进制 fixture**：输入
是 `tests/fixtures/**` 里已有的文件，其 blake3 由 `tests/fixtures/corpus-fingerprint.json` 固定。

| 位置 | 类（条目名） | 字节 | blake3[:16] |
| --- | --- | --- | --- |
| 根 | `Scope.class` | 532 | `e00e168ec30b8699` |
| 根 | `Shape.class` | 191 | `31487d3ff6e5a4a9` |
| 根 | `LambdaSample.class` | 731 | `92f2b3bcf6c6849d` |
| 根 | `Holder.class` | 359 | `a2e7af190439b1cb` |
| 根 | `HistoricalControlFlow.class` | 303 | `188e1103d1f4eccb` |
| `p/` | `BooleanContexts.class` | 1114 | `1d884b34a2b86372` |
| `p/` | `Guarded.class` | 3814 | `fc4f3cb74079240c` |
| `p/` | `ConcatConversion.class` | 1461 | `00ac14bf1906344f` |
| `p/` | `IntComparisons.class` | 647 | `4b1362ac75fadf6e` |
| `p/` | `SlotTypes.class` | 605 | `c06e57e9e7730ea4` |
| `p/` | `RequiredConversions.class` | 1031 | `96ed4aee49026f64` |
| `p/` | `ReceiverGrouping.class` | 1038 | `5e16d691d1ef6700` |
| `p/` | `HoistedBoolean.class` | 743 | `3cced9b171c5d791` |
| `p/` | `LocalRewrite.class` | 650 | `7aa56f8589c4c1c3` |
| `p/` | `ModLike.class` | 568 | `a603a3884e18c4eb` |
| `p/` | `NestedEval.class` | 361 | `3697d4178bbed31e` |
| `p/` | `ArrayTypes.class` | 511 | `69bcd9b9dcf040a5` |
| `p/` | `RefusedCast.class` | 666 | `2652eb2c60298f7d` |
| `p/` | `Res.class` | 733 | `a9e6a3bfb34f95c9` |
| `p/` | `ConcatJava8.class` | 1087 | `cecc0aa6452dc1c0` |
| `p/` | `CodeOnly.class` | 285 | `1c401c66ec9220f4` |
| `p/` | `External.class` | 427 | `8a94fba71b6f7b13` |
| `p/` | `MissingDependency.class` | 264 | `577522903c6f23d8` |
| `p/` | `Flags.class` | 597 | `b5eaefaf761d5d3b` |

固定的读数是**这些字节的函数**，由 `the_fixture_is_the_pinned_bytes` 钉住：整个 fixture 15824 字节、
blake3 `145f7903f52e16d087584477c2d30c413023547c16e6d168bea3c558b9030854`；枚举得 24 个
`.class` 条目（含嵌套容器内）与 `lib/more.jar`；`list_class_declarations` 得 24 类；一次
whole-scope sweep 交付 **183 条方法记录 / 233 条记录 / 24 类**，状态 `complete`。任何增删类、
改压缩方式或改条目顺序都会移动摘要并让这个用例变红——这就是"固定工作负载"的门闩。

## 3. W1–W5 与 W6a 的落地形态

| ID | 样本（JSON 里的 `sample`） | 实际执行 | 主读数 |
| --- | --- | --- | --- |
| W1 | `w1-cold-method` / `w1-second-request` / `w1-second-snapshot` | 同进程内：open→发现目标→一次 `recover_target_with_evidence(essential)`→序列化；随后同 snapshot 再来一次（控制组）；再对**同一输入开第二个 snapshot** 重复 | open/prepare/request/output 四段、`first_result_micros`、`text_bytes`、usage、store、`counts`；campaign 每样本一进程，故首样本是进程冷态 |
| W2 | `w2-navigation-sequence` | 前 8 个类各一次 `class_view`（不带 body），随后对第 0 个类再来一次（控制组） | 每请求 micros 序列、序列总时长、逐请求 `class_headers`/`method_bodies`、`returned_bytes`、store 报告 |
| W3 | `w3-all-bodies-one-request` / `w3-per-method-requests` / `w3-fixed-set-across-classes` | 一个类的**全部 body 一次请求**；同一批 body **每方法一次请求**；前 8 个类各取固定一个方法（固定方法集合） | 每方法/每请求 micros、`method_bodies(_total)`、`class_headers_total`、`archive_entries_total`、`counts`（class_materializations / class_preparations / body_decodes / recovery_runs） |
| W4 | `w4-small-page` / `w4-large-page` / `w4-multi-consumer` / `w4-abandon-after-one-page` | `query(ConstantPoolContains java/lang/Object)`，`max_items=2` 与 `64` 各续到耗尽（页数上限 `PAGE_CAP=4096`，样本自带 `pages`/`has_more` 说明是否触顶）；1 个 consumer 与 4 个 consumer；只取一页后丢弃 cursor | 逐页 micros、`items`/`pages`/`scanned_items`/`has_more`、`returned_bytes`、条目身份集合摘要 |
| W5 | `w5-sweep` / `w5-round-trip` | 整 scope sweep（`capacity=tiny` 1 entry/4 KiB 或 `roomy` 1<<14/1<<27）+ 同一 store 上 10 次单方法恢复 | sweep 的 usage/分类/store 报告；round trip 的逐请求 micros 与 store 命中/目录解析 |
| W6a | `w6a-export` | `Engine::recover_all` 整 scope，worker ∈ {1,2,4,6}，sink ∈ {`discard`,`encode`,`write`} | 冷进程到流结束的 `window_micros`+外部 wall/RSS、`first_result_micros`、`records`/`methods`、`encoded_bytes`/`written_bytes`、`outcome_*` 分类、`traversal_complete`/`final_delivered`、stream 指纹 |

**四段与序列总成本**：`Stages` 账本把 `open`（读入并 hash）、`prepare`（调用方自己的发现：load root
声明 + `list_class_declarations` + 目标选择）、`request`（被测量的库操作）、`output`（调用方把返回
文档渲染成字节）测成**同一时钟上互不相交**的区间；`Stages::phase` 拒绝在上一个阶段结束前开始下一个，
`window_micros` 是"首阶段起点到末阶段终点"，`unattributed_micros` 是窗口减去各阶段之和。**启动**不在
账本里：它是进程自己的（exec/dyld/测试 harness 初始化），只有 campaign 的外部包裹能看到，登记为
`wall − window` 的残差。**嵌套阶段**（bulk 的 sink 及其编码、写文件）记为 `nested`，带 `parent:
"request"`，永不进入阶段总和——`the_stage_ledger_keeps_phases_disjoint_and_nested_stages_out_of_the_total`
把这两条规则本身当成被测对象。

配置来自 `JARDE_OPTIMIZE_*`（`WORKLOAD`/`ARTIFACT`/`WORKERS`/`SINK`/`CAPACITY`/`INSTRUMENT`）；
`ARTIFACT` 给出的是外部语料路径时用 `EnvironmentPolicy::PlainJar` 与整棵 tree scope，并按
`examples/bulk_scope_sweep` 与 CLI 默认的口径把计数维度抬到 1<<40（否则 bcprov 的 15,003 body 会
在 harness 自己的上限上停下，把"本 harness 的限制"读成运行属性）。

真实语料（次级证据，路径与摘要见 [g0-baseline.md](g0-baseline.md)）：bcprov 15,003 方法 /
19,865 记录 / 2,430 类；其 sweep 状态是 `partial`，分类为 produced 12,045、explanation_only 2,447、
no_body 508、not_produced 3、refused 0、oversized 0，`traversal_complete=1`、`final_delivered=1`。
fixture 的 sweep 是 `complete`。这两个状态都是运行自己的读数，不是 harness 的判决。

## 4. W6b 登记：跨请求并发 / single-flight —— **暂缓，且理由是"缺需求"**

- **实际需求证据：没有找到。** 仓库里与本项相关的记录只有三条，且都不是宿主需求：
  1. `openspec/changes/archive/2026-09-20-p5-measured-optimization/verification.md` §2.1：多次测量
     合并 / 细粒度并行 / single-flight 的裁决是**保持 disabled**，并记录了**重新进入的触发条件**：
     "出现「一个 snapshot 两个请求」的行、或某行资源每单位份额居首且差异越过同机带宽"；其中
     "single-flight：无对象可验（没有任何共享请求）"。
  2. `docs/support-matrix.md`（第 160、169 行）：触发条件同上，并写明"并行 / merged / single-flight
     路径：**普通入口：不存在**"（只存在显式 bulk 的类间并行）。
  3. `openspec/changes/add-parallel-bulk-recovery/proposal.md` 的非目标明列"跨独立调用的
     single-flight"；`optimize-demand-workloads` 的 design §10 也只要求"W6b 的相同 key 同时构造需
     另有证据"。
- **本次处置**：**暂缓该候选**，不实现、不建 key、不引入 async runtime，也不新增 W6b 工作负载。
  已确认的类间并行（W6a）不受它阻断：它的 1/2/4/6 序列已在本轮测完（见 [g0-metrics.md](g0-metrics.md)）。
- **状态可见性**：harness 的工作负载词表是 `WORKLOADS = ["w1","w2","w3","w4","w5","w6a"]`，
  `the_workload_vocabulary_is_the_one_task_1_2_fixes` 断言 `w6b` 不在其中，且向 `run_workload` 传入
  `w6b` 会被**显式拒绝**并指出"没有登记需求、触发条件在哪"——新增 W6b 必须同时改这条用例与本登记，
  不会以一个新字符串的形式悄悄出现。
- **重新进入条件**（沿用上条既有触发，不新编阈值）：出现一个宿主在一份 snapshot 上并发发起多个请求
  的实际场景（相同或不同目标），且其重叠对相对顺序对的差异越过同机噪声带宽；届时再按 design §10
  定义 key 字段、订阅者独立预算/coverage/取消与生产者状态表。
