# Benchmark 架构复核与变更顺序

这次数据最明确地暴露了三个边界：按需请求仍在重复做容器发现；产物存在与含语句缺少可直接消费的区分；物理布局已识别，却无法声明带前缀的加载位置。源码核对另外确认了一个更小的交接缺口：driver Header 已读，恢复入口却拿不到声明类的 name/flags。

这是一份基于历史 `cd6f2f0` 测量的架构调查。下面四个 change 现已分别实现并归档，原观察与旧数字保留用于解释动机，不描述当前实现或修正后收益。当前状态以 [完成复核](completion-review.md)、[路线](roadmap.md) 和 [重新测量协议](benchmark-protocol.md) 为准；任务操作停止缺口已在 `8586356` 关闭，后续恢复反例分别验收，性能专项仍未完成。

2026-09-21：T1–T4 的约定修复判据已关闭，新增 T5（`append(int)` 消费 char）和 CLI 文档预算限制单独登记。多线程全量导出已有[独立设计](changes/add-parallel-bulk-recovery/design.md)，实现仍 0/22；以下历史表中“需求待确认/本轮不引入”不代表当前并行需求缺失，也不构成已完成性能对照。

## Changes

| 建议顺序 | Change | 架构修正 | 核心验收 |
| --- | --- | --- | --- |
| 1，可先落地 | [expose-recovery-content](changes/archive/2026-09-20-expose-recovery-content/proposal.md) | 给最终交付产物增加内容分类，保留 Produced 的存在语义 | 仅解释/含语句/未产出可对账，停止时不报告未交付 AST |
| 2，性能主线 | [bound-container-lookup](changes/archive/2026-09-20-bound-container-lookup/proposal.md) | 按 container 定向访问；复用权威目录、locator 与 nested backing | 不展开未搜索 sibling；保留期间第二次目录解析/父容器解压为零；完整结果语义不变 |
| 3，输入能力 | [bind-prefixed-load-roots](changes/archive/2026-09-20-bind-prefixed-load-roots/proposal.md) | 用 container+raw prefix 声明加载位置，CLI 补树枚举 | WEB-INF/classes 可绑定，raw origin 不变，root 顺序/重复项/损坏边界保留 |
| 独立小修，可与 1 先做 | [carry-declaring-class-evidence](changes/archive/2026-09-20-carry-declaring-class-evidence/proposal.md) | 同次 Header 的 class name/flags 贯穿只读交接 | 公开入口声明 form 正确，Header/Body 读取不增加 |

前缀 root 与容器优化没有语义硬依赖，但都会改 providers，建议在定向访问稳定后集成前缀迁移。内容分类和声明交接不依赖 cache，也不依赖彼此；评估质量变化时建议先固定内容分类，避免改动前后分母漂移。

既有 R8/R9 修复单独验收。本报告的性能/分布基线来自 `cd6f2f0`；分析期间观察到 `fd0aae8` 提交修复，`7024e0e` 完成其归档，复核时 HEAD 为 `25ed621`。后续实现以实际固定候选 SHA 和归档验证为准，重新建立 direct 基线；不重开旧正确性 change，也不把老 corpus 分布当成修复后实测。

## 历史调查（以下源码形态与数字均为交付前）

四项实现依次为 `7d095ce`（content）、`618de49`（声明交接）、`b22ea04`（定向访问）、`fbcf06b`（prefix roots）。下文“当前/现有”指原调查时点；旧外部原始样本已丢失，不可据此复算，也不可当成新基线。

## 1. 重复工作发生在哪一层

当前成本链可以分为：

```text
recover_method
  → driver definition binding / Header demand
  → 每个被搜索 root 的 raw-name 查找
      → flat: enumerate
      → tree: enumerate_artifact_tree → 找指定 container → 过滤 entries
  → read nested entry
      → 重放 origin chain → 定位 parent entry → 再物化 nested JAR
  → CP/Header parse → IR → recovery → text
```

源码入口：[`zip_candidates/tree_candidates`](../crates/jarde-jvm/src/providers.rs)、[`read_nested_entry_with_accounting`](../crates/jarde-reader/src/artifact.rs)、[`FactsCache`](../crates/jarde-reader/src/facts_cache.rs)。Atlas 对 tree_candidates 的文件级结构定位已完成；其余结论由这些具体源码直接核对，不依赖尚未完成的目录级 Focus。

S2-001 单方法请求计费约 5,900 entries / 7.0 MB，对照一次全树的 2,799 / 3.67 MB；同进程重复六次没有明显下降。这说明重复工作没有随 snapshot 生命周期摊销。它足以支持消除重复枚举/物化的设计，但还不能精确证明它占了多少墙钟时间，也不能证明优化后一定低于 1 ms。

这里不宜直接写“请求成本只由 classpath 大小决定，与方法无关”。本次 sweep 每个方法只声明其所在 container 一个 root，providers 却遍历了整个 outer snapshot 的树；实际观测首先证明**被访问范围大于声明位置所需范围**。总成本仍包含 Header 闭包、方法指令、CFG/SSA、异常结构与输出；当前测量没有逐阶段 profile 去否定这些项。

只打开 P5 facts cache 也不够。现有 cache 的注释和调用位置明确说明：class bytes 仍物化、CRC 检查和计费，命中只跳过 CP/Header parse。只缓存整个 ArtifactTreeReport 则仍有两个缺口：每次过滤整份 entries 的线性成本，以及 read_entry 重放祖先链造成的重复解压。所以 change 选定向 container 访问、多值 raw-name 定位与 verified backing 的共用事实路径。

保留 backing 会增加驻留内存，这是必须一起验收的代价。设计复用现有 cache handle，显式 entry/byte 上限，满则不插入；不直接引入 LRU、持久层或另一套 session。cache 不成为正确性前提，默认开关仍关闭。P5 的“同 snapshot 两个请求”触发条件已满足，触发的是 optimized/direct 对照，不是默认开启或收益承诺。

验收同时保留两类 fingerprint：同 cache 初始状态的确定性比较仍只排除 elapsed_millis，预算计数也要一致；跨 direct/cold/warm 才比较去掉实际工作/cache 计数的语义结果。热路径节省工作额度是预期收益，但 nested depth、elapsed、取消和已终止 Budget 仍生效；已验证 DEFLATED backing 不能让后续实际 class 读取不计费。

## 2. 输出质量要先说清计量对象

25,853 次请求中的 99.4% 是“有文本”，约 83.4% 是脚本判定“含语句”，16.0% 只有说明。后两个类别来自 `compare2.py` 去注释后是否剩 token；这是有效的审计线索，还不是引擎提供的 AST 级事实。`Builder::push` 又把 Fallback 当 statement 计数，所以不能直接把 `program.statements > 0` 暴露给调用方。

内容 change 只加一个分类：NotProduced、ExplanationOnly、ContainsStatements。它不回答“恢复了多少语义”。例如 `if (arg0) {}` 包含条件求值，`return;` 是语句；两者都可以满足 ContainsStatements，同时整体仍是 Mixed/Fallback。源映射有 BCI 也不能替代该 BCI 的操作语义已经保留，R8/R9 的回归继续独立存在。

相似度统计也有选择条件。bcprov 的 10,977 是文本有效 pair，调用指标实际只有 6,132 个有效 pair；没有调用、未匹配方法和纯说明会改变各指标分母。中位 0.80–1.00 说明有效子集的调用文本相近，不能证明调用次数、求值顺序、异常优先级、初始化效果或所有请求的行为相同。

因此不以 token 相似度为恢复算法的优化目标，也不以 Structured/Produced 比例直接设置正确性门禁。高置信结果用受控编译/执行反例检验；Mixed 用原始操作、effect 和 origin 验收；语法糖与文本风格后置。

## 3. WAR 问题应修声明模型

`entry_name()` 构造 `<internal_name>.class`，候选匹配对 raw name 精确比较。`WEB-INF/classes/` 是物理 entry 名的一部分，现有 Snapshot/ArtifactTree root 没有相对这个前缀加载的含义。158 次 unbound 首先是运行环境无法表达所选定义，不能并入 IR/恢复算法失败。

独立 change 将 ZIP 加载位置统一成 container origin + raw prefix；普通 JAR 是空 prefix，WAR classes 是显式 `WEB-INF/classes/`。物理名字、ordinal 和完整嵌套链不变；既有 driver binding 仍验证这个 loader 是否真的选择该物理定义。使用“直接接受目标物理 entry”、全局剥前缀或按类名去重都会把 runtime selection 与 physical identity 混在一起，因此不用这些捷径。

CLI 缺少树枚举，使用户难以取得构造请求需要的真实身份。补一个薄 operation 即可完成发现→显式声明→方法恢复，不需要引擎替用户猜 Servlet/Boot 加载顺序。Boot/MR、classpath.idx 和重复定义的运行时专项仍要各自有证据；一个通用 prefix 样本不能充当这些专项验证。

## 4. 高频声明诊断来自事实丢失

[`engine::raw_facts`](../crates/jarde-jvm/src/engine.rs) 已有目标 Header；[`MethodDeclaration`](../crates/jarde-jvm/src/method_ir.rs) 只保留 member flags 等字段；[`facade::recovery_facts`](../src/facade.rs) 没有填声明类，最后 [`declaration::plan`](../crates/jarde-java/src/declaration.rs) 因缺 class flags 无法判断 interface/default 等形式。

这是同次读取的两个字段没有传下去。增加依赖下载、全 classpath Header 扫描或跨类 Body closure 都不解决这条交接错误。小 change 只传 driver class raw name/flags，复用已有 DeclaringClass，不把全部 ClassFacts 复制进 IR。

清除 `jre_declaration_class_not_in_run` 不代表能把 50.9% Mixed 大幅转为 Structured。它记录的是成员声明 envelope 的事实缺失，字段访问、new、enum switch、handler/phi 的失败还有各自前置条件，必须分开归因。

## 5. 测量边界与后续债务

| 观察 | 本轮决策 | 另开工作所需证据 |
| --- | --- | --- |
| whole-artifact API 缺失 | 保持方法 API；多请求的容器复用先收敛 | 明确用户需要批量交付、逐方法预算/取消/背压契约，再设计薄批量入口 |
| 语句缺口集中 fallback | 保留缺失原因，先补内容计量与已读声明交接 | 按失败 rule/前置条件取最小反例；分别评估 stack phi、handler roots、跨类 member/body 证据，不一揽子放宽证明 |
| evil.jar handler、ASN1Boolean 分支样例 | 作为 triage 入口，不承诺一个算法修完 | 固定方法字节、降级原因与受控 oracle，确认是 IR/Region/AST 哪层的问题后单独 propose |
| 物理重复类与 jadx 去重 | 保留全部物理 origin，未作为性能债务 | 由显式 loader/root 顺序决定 runtime 选择，不采用全局去重 |
| cache 当前挂 Budget | 复用已有注入路径，不建立新 session 框架 | 只有多种共享服务确有生命周期需求时，再独立评估 API ownership |
| 并行、merged、持久索引、single-flight | 本轮不引入 | 消除重复工作后的 profile 仍有独立瓶颈，再按收益与资源约束提案 |
| 行为等价/Boot/MR/混淆/取消延迟未测 | 不从当前 corpus 推断通过 | 分别补固定 scope 的对照与边界；取消是新 cache 的必测门禁，不能等待性能完成后再补 |

几个需要保留在后续报告里的口径：

- flat JAR 为全量、WAR 为按 container 抽样，不能按表中每请求中位数外推整 WAR 耗时。
- 每次 recover 新 Budget，open/tree discovery 和 inspect_header 在它之外；同 snapshot 复用但 facts cache 未开。roots 只有目标 container，Java 8、MR disabled、Generic layout、providers 为空；这不是完整应用部署 classpath 的效果评测。
- `archive_entries/read_bytes/analysis_steps` 是实际工作的计费代理；累计计费不等于同时存活内存，sweep 的 81.8 MB 是进程峰值 RSS。
- 进程级 919 ms 与 8.7 ms 对齐了启动边界和按需场景，工作单位仍是“一个类”对“一个方法体”，且迭代次数不同。可以比较两种用户动作的观测延迟，不把它命名为严格等量工作的 ~100× 加速。
- 调用序列相似度不等于行为；jadx 本身也不是正确性证明。`<init>/<clinit>` 省略/折叠不是失败，尚未解释的未匹配项应保持 Unknown。

## Evidence

本次只核对报告、脚本、结果摘要和相关源码，没有重跑性能测试，没有导入或删除大体积原始输出。实施验收应使用仓库内可再生成的小型 fixture；下列 `/tmp` 资料是审计来源，不是 CI 依赖。

| 资料 | SHA-256 |
| --- | --- |
| `/tmp/jarde-bench/jadx-vs-jarde-benchmark.md` | `a6bfc45fc0e1add80207da6841b66e159cf99536f5d814084b63f461180799b1` |
| `/tmp/jarde-bench/out/inventory.json` | `60d162b084a4327952af625db4bd03e18e65be993891a4755174894e1c1f6e3a` |
| `/tmp/jarde-bench/out/comparison2.json` | `b052436661b28de8ded05670eee69d3e00dba0cd98adad89991463ff8c9aad58` |
| `/tmp/jarde-bench/sweep/src/main.rs` | `f84e205285365cd64c4feb8a67c2b1760b8ac569b8ae03b882c9dbc7d6acf6ec` |
| `/tmp/jarde-bench/compare2.py` | `2bb2e091a5263c914f683c956b55b9929b6fef7721d0764e332f6e2102103bd5` |

关键复核位置：sweep `main.rs` 64–115 为 limits/roots/profile，182–191 为 open/tree，270–271 为 Header Budget，352–355 为 recover Budget；`compare2.py` 30–37、116–153 为 token/语句启发式与 pair 过滤。P5 触发条件及范围见 [归档 verification](changes/archive/2026-09-20-p5-measured-optimization/verification.md) 的 196–204、242–283、401–412 行。行号仅用于这次内容摘要对应的文件，后续以符号与 SHA 定位。
