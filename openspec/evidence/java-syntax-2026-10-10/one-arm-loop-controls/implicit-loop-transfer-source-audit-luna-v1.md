# 隐式循环回边的 source-map 来源审计

本页只记录已经观察到的来源缺口及最小架构接缝，不提出实现或 OpenSpec，也不涉及 one-arm loop 续接。

## 可复核的缺口

输入是 `one-arm-loop-controls/inputs-prepared-luna-v1/PlainOneArmLoops.java` 的 `noPrefix(boolean,int)`。原始 JDK 23 `javap -p -c -s -v` 中，该方法的指令 BCI 为 `0,1,2,3,6,7,8,11,14,17,18`；其中 BCI 14 是 `goto 6`。对应实际运行记录 `baseline-root-v1/cases/javac23-jarde-render/class-source-all.json` 显示 `noPrefix(ZI)I` 为 `produced / structured`，输出了完整 `if`、`while` 和 `return`，没有 bytecode fallback；但 source-map origins 只有 `0,1,2,3,6,7,8,11,17,18`，缺 BCI 14。也就是说这是结构化源码正确但循环隐式回边未被来源表呈现的独立债务。

## 来源在哪一层丢失

- `region.rs:519–535` 的 `Region::Loop.gateway_origins` 已明确表示“由 loop 结构代替语句呈现的隐藏 transfer”。普通 header-tested loop 在 `Walker::header_tested_loop`（`region.rs:10086–10328`）确认 body coverage 后创建 `Region::Loop`。简单 `while` 的创建分支（约 `10146–10155`）将 `gateway_origins` 设为 `Vec::new()`，故 `noPrefix` 的 BCI 14 没有随 loop region 保留。
- 下游接缝已存在：`Builder::region` 的 loop arm（`build.rs:17806–18069`）以 loop test 建立 loop statement 的 `OriginSet`，再将 `gateway_origins` 逐项作为 derived origin（`18067–18069`）并发射该语句。`emit.rs` 的 `Emitter::node` 在实际 span 非空时按 `origin.bcis()` 记录 source-map segment（约 `620–642`）；所以将一个已证明、确由结构吸收的 latch BCI 放进已有字段，就会把它挂到整个 `while` statement span。Formatter 不会自行重建或推断隐式 `goto`。
- 已有精确来源路径可复用但范围不同。特殊 endless/latch 形状在 `region.rs:10072–10080` 将已验证的 `latch_transfer.bci()` 放进 `gateway_origins`；`loop_exit_gateway_pair`（`11870–11985`）严格验证 header exit 与 latch 回边，返回包含 `latch_bci` 的来源对，普通 header-tested loop 在 `10307–10316` 将 exit-gateway 来源和 fragmented-for update-transfer 来源并入字段。这些证明只覆盖各自专用形状，不会给普通 `while` 自动提供 BCI 14。

Atlas 已先 `project(open)` 本仓并作 `region.rs` 局部查询；`Walker::header_tested_loop` 的 scoped symbol context 显示它由 `Walker::loop_region` 调用，且调用 `loop_body_sequence`、`loop_exit_gateway_pair` 等局部证明。一次按 `loop_region` 字符串的 scoped search 没有返回符号，因此精确分支和来源流以上述当前源码为准；Atlas 查询没有被用作“缺少符号”的证据。

## 最小闭环与安全边界

最小候选是在普通 header-tested loop 的成功构造点复用 `gateway_origins`，只登记确实被 loop 语法吞掉的隐式正常回边：来源必须是该自然循环的 latch block 中经指令事实确认的终端 `goto/goto_w`，其正常目标必须精确等于当前已证明 loop header，并且该 transfer 没有由子 `Region::LoopContinue` 等显式语句承载、也没有已由 exit-gateway / for-update 路径登记。然后沿用现有 derived-origin 折叠，不创建新的 source-map 机制。只按 BCI 相等收集所有 `goto` 不安全：嵌套 `continue` 可指向外层循环，且已显式呈现的 transfer 不应误挂到内层或外层 loop span。候选枚举和边核验应沿用现有 poll/预算化扫描；证据不完整则保留结构化文本、保持来源缺项并由验收明确失败，或按既有该形状 refusal 收敛，不能伪造 owner。

永久测试最贴近的入口是 `crates/jarde-java/tests/p3_loop_exit_gateways.rs`：`cf08_two_gateways_have_one_loop_owner_and_complete_sources` 已同时检查 loop 来源与唯一 block owner；加入一个简单 header-tested `while` 控制（优先复用冻结 `PlainOneArmLoops.noPrefix` 的完整 class bytes）并断言 BCI 14 映射到 loop statement span，可直接覆盖此缺口。它不是去改现有 gateway 语义。两个必要反例沿用现有边界：

1. 错误 loop 目标：`p3_loop_body_double_jumps.rs::two_jump_loop_bodies_present_both_edges` 和 `jump_transfer_blocks_have_one_owner_and_complete_sources` 中 `L5.brkSelfContMid` / `DJLoops.dblContLabels` 的跨层 `continue`，确保显式 transfer 仍归其实际 loop/source span，不因回边采集误记到相邻 loop。`p3_loop_exit_gateways.rs::cf08_unproved_gateways_keep_physical_quotes` 的 `differentTarget` 也可作为拒绝错误 loop-exit target 的既有控制。
2. 预算 Stop：沿 `cf08_gateway_budget_and_cancellation_publish_no_partial_source` 的无部分发布断言，令预算在新增 latch/target 扫描处停止；结果必须是 stopped outcome，且 text/source map 均为空，不把扫描中断降为普通 shape refusal。

**范围限制：** 上述事实只证明 `noPrefix` 的 BCI 14 被遗漏，以及已有字段足以表达一个经证明的隐藏 transfer 来源；尚未证明所有普通 loop 形状都漏回边，也未证明任何修复已实现或通过测试。没有运行 Git、Cargo、rustfmt、JDK、JADX 或 Jarde CLI。
