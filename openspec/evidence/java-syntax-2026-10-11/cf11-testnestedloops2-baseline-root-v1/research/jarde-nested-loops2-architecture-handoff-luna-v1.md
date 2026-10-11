# NestedLoops2 架构接续记录

按固定账本推进：先有独立基线和可复核的观察，再讨论实现。证据只覆盖 CF11 `UpstreamNestedLoops2.test(List)Z` 这一方法和当前记录的输入，不代表整个 CF11 样本集或 71 个单元。

## 已核实行为

`/private/tmp/jarde-cf11-testnestedloops2-baseline-root-v1/checker-root-v1.json` 状态为 `verified_observation_capture`，对应 29 条命令、58 个原始流；`execution.json` 中 29 条均 observed 且退出码成功。JDK 8 与 JDK 23 原始程序输出相同，九行分别为 empty、same6、ten、eleven_then_zero、zero_then_eleven、ascending、descending、null_list 和 null_element（见 `raw/2.stdout.raw`；`raw/5.stdout.raw` 同哈希）。JADX 1.5.6 default 与 rename-none 输出均保留 `List<String>`、外层 indexed `for (int i = 0; i < list.size(); i++)` 和内层 `while`；两种源均在两个 JDK 编译且运行输出匹配原始程序。

Jarde default/all 在 JDK 8/23 均成功渲染、编译，运行也与原始九行一致，default 与 all 的输出身份相同。但 default 输出 `public boolean test(java.util.List list)`，丢失 `List<String>`，并把外层 indexed-for 表达为 `while (i < list.size())` 加尾部 `i = i + 1`；内层仍为 while。这里是两个独立质量缺口：循环呈现形式，以及泛型签名投影。运行语义相等不能消除这两项差异。

## 架构定位与证据边界

现有候选门槛文档 `/private/tmp/jarde-cf11-testnestedloops2-for-gate-luna-v1.md` 根据实际 class/Javap 和当前源码推导：`Walker::loop_region` 的一般循环路径尝试 `header_tested_loop`（`crates/jarde-java/src/region.rs:11541-11549`）；单条件形状不走至少两个测试条件的链式分支（`:13313-13315`），候选路径会到 `prove_for_header`（`:12783`）。真实字节码条件含 `List.size()` 调用（BCI 6），而 `for_header_candidate` 条件扫描静态上允许 Load/Push/算术/比较并拒绝其他 operation（`:13668-13695`）；`test_is_pure` 又允许调用作为条件表达式生产者（`:9586-9594`）。这只是源码候选 gate，不是同次运行的 gate trace：目前没有证据确认真实 IR 进入 `prove_for_header`、具体 SsaInstruction 序列或实际首拒分支。后续报告不得把 BCI 6 推断写成运行观测。

JADX 的 indexed-for 是一般 `LoopRegion` 建成后的窄 `LoopRegionVisitor.checkForIndexedLoop` 识别；Jarde 已有 `ForHeader` SSA/CFG 证明和投影。因此先做同 IR 的最小诊断，确认方法/块/指令与各既有门槛结果；若 trace 证实条件调用是首拒，再考虑对既有 `ForHeader` 条件表达式接受范围作最小扩展。保留已有 init、phi、update 和循环不变性证明，不另造识别器或机制。源码定位可参考候选 gate 记录中的 `region.rs` 行号；实现前需以当前 live source 核对。

泛型退化是分开的 source projection/fallback 问题。当前实现入口在 `src/class_source.rs:7178-7540` 的 `ordinary_parameterized_declaration`；其中 7401-7403 在缺少 same-run body proof 时要求 `GenericReturnCandidate`，否则以 `same-run Program/SSA cannot prove the body under parameterized types` 拒绝。该拒绝消息也出现在 Jarde baseline 输出。把它列为独立架构债务，本轮先不实施，也不把它并入循环门槛修改。

## 接续约束

固定账本基线不覆盖：先实现循环最小闭环，再按新观察结果记录。不可声称已验证整个 CF11/71 单元；不可声称已获得 same-IR gate trace。Atlas cache 已知陈旧，不作为证据，也无需运行索引或同步。
