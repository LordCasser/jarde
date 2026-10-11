# TestNestedLoops foreach 同次真实入口诊断

Root 通过 Engine::class_source 对冻结 JDK23 class 执行真实完整入口（Java8/Generic/SnapshotAll/SingleClass），准确使用 declaration/debug facts；不以手工 parameter slots 替代。临时仅 build.rs 与 facade.rs 日志/test，guarded runner v2 实际一测试 1/0/0、两命令、峰值 346991534 bytes；4718 项闭包 before/after/live 与 finalbuild-v3 恒同，源码恢复，target clean。

实际 Builder 先处理内层再处理外层。内层 hasNext branch42、header35，body 首句 Declare，入口 gate 通过后返回 post_entry_candidate_proof_incomplete；尚未定位闭包内部的第一个拒绝条件。外层 branch13、header7，body 首句 Assign，被 body_first_not_initialized_declaration 明确拒绝。此处只确认入口事实，不能把原始 List 泛型和两个 foreach 拒绝合并成同一原因，不能声称现有 SSA/ownership 后续证明全部通过。

runner v1 实际只在前置 pin 校验失败（candidate pins 遗漏 facade::），未应用源码/未执行 Cargo；原失败 invocation 保留。candidate v1 在静态审查发现 moved body 和 dotted/internal 类名错误，未运行；v2/v3 Rust/patch 字节相同，v3 修正 pins/README 路径。归档 private patch 可在 receipt pin 的原 base 上还原 candidate，未重复保存两份数 MiB 完整基础源码。

原始完整类行为对照仍见相邻 cf11-testnestedloops-baseline-root-v1。此诊断不改变已验收 for-condition 产品或 CF11/71 完成数。下一步继续定位内层 gate，再决定复用局部声明/SSA/所有权的最小修复。
