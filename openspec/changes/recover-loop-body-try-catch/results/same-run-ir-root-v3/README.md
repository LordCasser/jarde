# 当前 DF/DV 同次 IR 观察

root-v3 实际执行 debug/none × javac8/23 的四次确切 diagnostic test，每次 1 passed/0 failed/0 ignored；最后 exact cargo clean exit 0。当前产物只用于观察，未验收产品实现。完整 4822 来源闭包按字节恢复，target absent，无资源守卫中止，默认 Rust stack。

root 独立读取所有原始流并核验 SHA、四 lane × 四 method 精确身份，与当前 CLI 基线同 class BLAKE3/length/name/descriptor；确认无 Stop，完整 Code/canonical/SSA、现有证明与假设续接排除结果、foreach 实际写拒绝均齐全。见 [instrumentation.md](../../instrumentation.md)。

前两次临时插桩构建失败完整保留：root-v1 因 SlotUse 未实现 Debug，root-v2 因误对 &SlotUse 调用 map；均 cargo 101、源码精确恢复、clean 0，无运行时 IR 事实。root-v4 私有准备改成实际五字段 tuple 后，root-v3 四次观察实际成功。两份失败不拼接为成功，不修改产品判断。压缩包包含全部候选、脚本、原源码、收据和 stdout/stderr。

无限 retry/break 代表尚需另外实测；假设 handler→join 排除能使归约通过，不等于该边已经认证或允许删边。Q-i/Q-ii 已确认具体拒绝；Q-iii 独立 SSA 门已确认，设计完成之前任务 1.2 保持开放。
