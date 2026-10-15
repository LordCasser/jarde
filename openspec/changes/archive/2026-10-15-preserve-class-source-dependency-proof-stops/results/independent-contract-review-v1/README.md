# Class-source dependency proof 停止契约复核

建议另立有界的 **class-source dependency proof stops** 变更，只纳入已证实的 snapshot 前置证明和 double-brace 后置投影。既有狭义 interface-super 计划保持独立；reference overload、direct-parent accessor/field 三处仍是静态债务，不承诺本片实现。没有新建或修改 OpenSpec。

已亲读并验算固定 c814 CLI 的 cap8-v3 与 cap15 GDB receipt、脚本、原始流 bytes/SHA。cap8 恰五次真实 Budget::exceeded：前四次经过 snapshot_header_chain_widens → prove_snapshot_hierarchy_widenings → wrapper → recovery_from，第五次经过 double-brace 投影；inferior 正常退出，class/main Complete，main explanation_only。cap15 仅一次真实拒绝，经过 double-brace 投影且无 snapshot 栈；class/main 仍 Complete，main contains_statements。两个 debugger 完整 JSON 去除 elapsed_millis 后分别等于普通 CLI 对照；cap15 main report 去除 elapsed_millis 后等于 wide，main/class source 与 wide 逐字相同。脚本只有断点、bt30、continue，没有修改返回值或变量。

真实丢失位置不止外层吞 Err。`src/facade.rs:21621–21628` 将 resolver 的非 Complete 合并到 proof 局部 execution 后返回 Ok(None)。snapshot collector 最终只返回 Vec，局部停止不进入 recovery；double-brace 的 `4851–4863` 在 let-else 直接 Ok(None)，所以 `2097–2115` 已有的 Err 聚合根本看不到。只改 catch 为 `?` 不足，也不能重 charge/poll 猜回原 Error 或伪造原 limit/requested。

最窄接线应传递原 execution/诊断。**前置阶段**复用最小 stopped-before-recovery 构造，保留 raw analysis/callee 身份与 pending evidence，artifact not_produced，在正文工作前停止。**后置阶段**保留已提交正文与可靠兄弟前缀，将原停止汇总为 class Partial/Cancelled，并结束后续收费投影；不能把已有正文改成 not_produced。double-brace 后的 anonymous-interface 分支 `2118–2157` 尚无 Complete gate，实施时需确保新停止不继续进入该分支。child 非 Complete 后 Ok(None) 的 `4992–5006` 是同函数内静态待核交接，不称为另一条动态复现。

JAR 内 H1.main 的 9 条 invokespecial 全为 CP tag10 Methodref `<init>`，没有接口特殊调用、putfield 或静态 access$ 调用。物理 digest 已与报告绑定。因此本例不属于接口 super 证明，亦不能据此声称直接父类 field/accessor 路径已动态复现。

现行 analysis-contracts、java8-recovery、conservative-output 与架构 §17 要求真实预算/取消停止独立于质量/content，并汇入组合结果；不完整 CLI 执行不能 exit0。原 snapshot delta 允许缺失、不可达和算法固定深度上限的普通保守拒绝，不能据此吞实际配置 Budget。既有 snapshot-proof-stop-debt 已记录静态风险；本轮新增的是上述两条公共消费路径的动态证据。

后续独立变更仍需原始原因/元数据、直接 Err 与非 Complete report、真实收费内取消/期限、普通合法/拒绝来源对照，以及停止后无后续恢复工作。当前四工程 closure 与 c814 CI 不为此预存缺陷扩大门槛。详细条文、证据身份和边界见 review.json；42 项读取 pins 在本轮末尾保持不变。未运行 Cargo/JDK/GDB/新 CLI、未碰 target、未修改生产或规划；没有宣称修复或新增测试通过。
