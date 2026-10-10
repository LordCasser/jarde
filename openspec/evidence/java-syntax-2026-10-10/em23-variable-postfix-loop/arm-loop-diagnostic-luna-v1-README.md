# 临时 arm-loop 诊断 v1（未应用）

此私有 patch 只用于短时定位 EM-23 的一次真实控制流路径。它不修改恢复语义、规则、文本或 source map，只在 `JRE_ARM_LOOP_PROBE` 存在时向 stderr 写诊断。root 负责审阅、应用与受资源守卫的临时编译/运行；本准备任务没有应用 patch，也没有运行 Git、Cargo、rustfmt、JDK、JADX 或 Jarde CLI。

Patch 记录三个层次：

- `report.rs` 在每个物理方法进入/离开 `region::recover` 时打印 owner/name/descriptor 和结束状态，供相邻 region 日志归属。
- one-arm walk 返回处打印 branch/start/join、各 Region 及其 block BCI、next、boundary/scope/own-loop/loop-target BCI。
- loop-header 的 prefix-return 处和 fresh `loop_region` 返回处打印 header/prefix/regions/next 及 frame 边界信息。Region payload只作为临时 Debug 输出，诊断不影响结果。

执行前后用同一已冻结完整类和 Runner 比较无环境变量与 `JRE_ARM_LOOP_PROBE=1` 的 stdout、stderr 与 exit。解释只看 stderr 的 `JRE_ARM_LOOP_PROBE` 行；若 begin/end 之间没有候选方法的 one-arm 行，便不能据此推断该方法经过了该分支。诊断仅供定位，得到结果后应立即撤销整个 patch；不得进入产品提交、永久测试或验收证据。
