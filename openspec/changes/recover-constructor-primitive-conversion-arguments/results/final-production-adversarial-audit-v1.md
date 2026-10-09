# 生产路径对抗审查 v1

审查范围：本片 `init.rs` / `report.rs` 生产 diff，重点看普通 constructor Site 的共享 Budget、Stop 传播与原子发布，以及 `PrimitiveConversion` 依赖标记和 handler 覆盖。只读审查；未运行 Cargo。

## Stop 与 pending Site

`report::recover` 在 `init::sites_after_array_composition` 返回 `Err(stop)` 时立即走 `stopped(...)`；该报告明确返回 `Stopped`、`NotProduced`、空 text/source map/regions/news，没有把已构造的局部 `Sites` 计划写出去（`report.rs` 9394–9409、12934–12979）。`sites_after_array_composition` 先将 array builder 的 pending map 一次性 `take_pending_sites()`，随后把 map 按值传给 census；普通扫描每读一个 SSA 指令先 poll/charge，`verify_metered` 内各计费 helper 的 Stop 经 `VerifyFailure::Stop` 直接返到外层 `Err`，不会被记成 Refusal 后继续（`init.rs` 527–545、577–617）。因此在 pending map 已被取出、甚至局部 plan 已含若干 Site 后发生 Stop，调用方仍只能得到停止报告；局部 map/plan 随错误返回而析构，不会发布半份计划。代价是这条接口为单次移交：若未来调用方在 Stop 后想重试，原 array pending 已消费；当前唯一生产调用方立即终止本次 report，没有重试路径。

现有普通 Site 预算测试验证了 census 在 `AnalysisSteps=1` 处返回明确 Stop，并验证预取消返回 `Cancelled(at=0)`（`init.rs` `ordinary_constructor_conversion_uses_the_shared_census_budget`）。这两个输入的 array map 为空；没有单独的 focused case 同时构造非空 pending map、在移交后触发 Stop，再断言停止报告没有 records。因此“没有对外半计划”由错误返回和 stopped 构造的结构保证，不是由该测试组合直接覆盖。

## Conversion 依赖与 handler 边界

普通 constructor 参数先从 SSA operand 根沿 instruction reads 和 phi inputs 计算依赖 BCI；该 walk 对 block 指令、value/reads、phi 查找和 phi inputs 都使用同一 meter（`init.rs` 2663–2718）。后续只在 allocation `dup` 到 constructor 之前的物理区间扫描：一个 `PrimitiveConversion` 只有其 BCI 也在参数依赖集合中才被接受并置 `embedded_primitive_conversion`；不相关 conversion 落入普通 `StatementFree` 拒绝路径，不会因 opcode 本身触发扩大 handler 接受范围（`init.rs` 1031–1084）。已存入 local、在构造区间外完成的 conversion 不会被 marker 当成新表达式的一部分。成员构造器仍走专用 `verify_member` / `member_ordinary_arguments` 路径，未由此处普通参数分支扩围。

marker 只决定是否运行原来的 handler coverage 检查，不代替 SSA 身份或参数证明。检查以 allocation BCI 的 handler ordinal 集合为基准，逐一比较 allocation 至 constructor 的每条 block 指令，并另比较唯一 consumer 的集合（`init.rs` 1233–1269）。consumer 来自完成实例的 SSA 外部 use：普通 Site 先要求读者仅一个、且该读者是可写入文本的指令（`init.rs` 1090–1128 附近、1144–1175）；Array Site 另要求它就是配对 `aastore`，并验证 `new` 的唯一栈输出是本次 NewSite、`dup` 读取该输出且写两个不同 SSA 值、constructor receiver 命中其中一个、保留栈槽的 constructor 输出恰为 store 的 ValueId，且该 ValueId 在同 block 仅被该 store 使用（`init.rs` 2362–2511）。

真实 handler fixture `sameHandler(I)Ljava/lang/Long;` 的表范围为 `[0,19)`，覆盖 allocation、`i2l`、constructor、`astore` consumer；按 CodeSpan 定位仅将 start 改为 8 后，allocation 的 handler 集与 conversion/constructor/consumer 不同，Site 保持拒绝，诊断为 `jre_new_primitive_conversion_exception_boundary`（`init.rs` 3817–3891）。这个变体确认 conversion marker 单独触发边界核验；实现本身也比较每条指令及 sole consumer，而非只比较 constructor 或 conversion。没有发现 handler loop 把不同覆盖的路径误收为一个 Site。

## 留存风险

有一处计费边界需要 root 决定是否记为后续债务：普通 census 的新循环和 verifier 是共享 Budget 的，但 census 收尾仍调用 `receiver_tails(ssa, operations)`；该 helper 会展平、排序并扫描整个 SSA 序列，且其 stack/use 检查没有 `VerifyMeter` 参数（`init.rs` 622–634、`receiver_tails` 定义）。这部分未被本片的 ordinary Budget 测试覆盖，也不会产生自己的 Stop/计费。该 helper 是原有辅助证明，本片没有改其算法；因此这不是“局部 Sites 可被半发布”的问题，但若“普通 Site 计费”要求涵盖收尾的全部实际扫描，它仍是一个未闭合点。pending Site 的 `plan.owned.extend(site.owned.iter().copied())` 同样没有在移交处逐项 charge；其 Site 证明此前由 array composition 用同一 request Budget 完成，且 map 只是 move 后汇入本地计划。

除此以外，本次审查未发现 PrimitiveConversion 依赖判定或 handler 覆盖检查的拒绝路径被降格、或者 Stop 被吞成 Refusal 的问题。
