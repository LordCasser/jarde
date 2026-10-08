# Root 独立验收 — 构造器函数实参与绑定引用时机

验收候选包含 `e47e7940` 构造实参实现、`a3ec10d6` 时机修正及后续最小收窄；以最终测试日志记录的 commit 为准。根验收不采用 subagent 的自我验收结论替代 JVM 对照。

## 已实测结论

- 九种构造实参（Runnable、捕获、Comparator、静态方法引用、Callable、原始 SAM、原始 SAM 静态引用、多实参副作用、重载选择）在真实 Corretto 8 与 OpenJDK 23 两腿恢复。完整 `FunctionalConstructors` **和** `IntBox` 均由本轮 CLI 输出重新编译，外部 Driver 不借原 jar 提供 IntBox；每腿八行行为与原 class、冻结 JADX 完整重编结果相同。
- `NoCheck`、`NoStand` 原 class 在两套 JVM 的 `-Xverify:all` 下均为 `creation=ok / invocation=NPE`。旧 Jarde 与 JADX 的引用语法会变为 `creation=NPE`。正式产物拒绝，不再输出 `arg0::start`。root 从完整 class-source JSON 的 make 方法来源集独立断言 NoCheck 恰为 `{0,3,4,5,10,13}`、NoStand 恰为 `{0,1,6}`。
- 真实 javac 的 `KnownBoundPositive` 整类重编回放，两 JVM 均为 `bound / 5`：实例 this 与直接 String 常量可以恢复；不是仅测文本。原三方法 KnownBound 中 Class 字面量的 `ldc;dup;getClass;pop;indy` 保留拒绝，未扩大复制值/检查所有权。
- 旧 `LG.pqLambda` 从完整呈现中原样取出方法，在独立类中重编执行，两 JVM 均为 `[3,2,1]`。**不宣称完整 LG 已恢复**：其 ArrayDeque 方法复用 int[] 局部变量为 ArrayList，以及 finalize 拒绝，导致完整类重编失败。stackOps/dequeOps/finalize/main 方法文本与冻结改动前 main 逐字节相同。当前 JADX 1.5.6 的完整 LG 也因 raw Comparator 的 Object 参数调用 intValue 而重编失败；已冻结失败日志并纠正旧巡查 README 过强结论。
- 47 个非空 guard/IO/循环/复制值完整呈现与改动前主线逐字节一致。实际计数是 47，不把 shell 函数定义或目录文件数当作样本。

可复跑脚本、完整输出、精确来源 JSON、成功/失败编译日志与 stdout 在 [results/root](results/root/)。普通 Cargo 目标还验证动态实参物理依赖、allocation/factory/constructor/consumer 四个 handler 覆盖边界、未知 bootstrap、无关动态值、弃置动态构造、先物化的 local、带真实 check 的可空接收者等。形式证明与 JVM 可执行控制分开记录，不把形状提早拒绝的样本冒充命中依赖门。

## 根审查与收窄

生产改动只放宽同一构造闭合跨度内的物理参数依赖，handler flag 也只在该跨度内设置；不能因为较早物化的工厂出现在传递依赖集合中，就把它当成构造表达式中重新执行的工厂。后续 lambda planner 仍独立验证 bootstrap、SAM、捕获、参数与返回适配；源构造调用使用物理 descriptor 的 SAM cast 固定重载。

直接与适配绑定引用共享已有 `receiver_nonnull` 参数。this 必须同时有实例方法事实与入口 this 身份；稳定分配复用已有 helper。String 常量类型读取复用已有 `constant_of_value`，保留 Unknown-frame 前提，不覆盖显式已命名 Object。曾尝试在通用常量 walker 跟踪 dup，全量测试发现它改变普通局部类型诊断（Object→Class），root 撤回了这处范围外放宽，旧断言不变；见 rejected-scope-expansion.txt。

拒绝表达式补齐的工厂/捕获来源是预期变化。BRN 三个负例双腿原 class 的 javap 与完整 CLI 已核对，只有来源行补入相应 indy/capture BCI；拒绝消息、解释性质量与其他文本仍相同，未将可空或重写的局部恢复成 lambda。

## 独立 CI 修正

全量测试检出主线已有的 worker panic 竞态断言。BulkSummary.traversal_complete 表示 discovery 是否走到末尾，execution 独立优先记录基础设施 Failed；EOF 与 panic 可以按任意顺序被观察。仅删错误的 false-only 调度断言，生产代码和错误/stop/终态/线程回收/类交付一致性断言均未改。root 精确回归 20/20 通过；两 seed 全库仍必须通过，不能把局部重复当作门禁替代。

## 门禁与剩余边界

最终 fmt/clippy、两个固定 seed 全库、ignored P3/构造整类 oracle 和 strict OpenSpec 的结果见 gates-summary.txt。首 seed 使用 no-fail-fast 收集全库失败；全通过时测试集合与 CI 相同。JDK 25 instruction-boundary oracle 由远端 CI 的 JDK 25 环境执行，本地 8/23 未冒充 25。

本片未恢复构造族的所有泛型返回 Signature，未解决 class-scope 泛型构造器、字段写投影，也未恢复 Class 字面量的 dup/check 形状。前两项已拆分排队；没有追加 planner、IR、全局 fixpoint 或 JDK owner 白名单。

主线收尾：全部实现与 root 证据已合入推送；实现分支已解除占用并删除，辅助 worktree 均 detached。cargo clean 移除 19.3 GiB，保护的干净工作树归档被 Codex 拒绝，未绕过。收尾 CI 的最终状态以 handoff 核对命令和远端 run 为准。
