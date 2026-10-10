# 浅层循环与保存返回值的 finally 接缝

这份样例是 2.4 深度探针提前回退时发现的组合候选，先保存事实，不作为新语法实施任务。用户要求先完成 JADX 已有 71 单元；当前 CF16 既有剩余验收优先，不能因一个深度测试输入失败就扩展机制或宣布深度验收通过。

## 实际完整类对照

原 N=2 双 JDK 完整程序在本次 root 深度探针中编译、验证、运行 2/2 成功，原 class/Runner/raw 按 hash 复用。root 审阅 Luna 脚本并收紧 JADX 版本检查后实际执行：15 条命令，fresh JADX default/none 四腿完整源码重编运行 4/4，Jarde 两份冻结 CLI 完整 source 原样重编 0/2。两套 javac 都在 `DeepFinally.java:33` 报 `缺少返回语句`，没有借原 classes 执行失败候选。root verifier v2 实际 **232 checks/0 errors**，完整文件 inventory 和原始流封闭；v1 英文 diagnostic 假设错误及其失败 raw 保留。

原程序两次调用 raw 为 `run(0)=0 trace=1`、`run(40)=1 trace=2`。JADX 将纯 int 循环头放在 try 外、循环体保留 catch(Throwable) 清理，再保存返回值并清理；此固定程序完整比较正确，不表示一般可抛错循环头也可移出保护范围。完整源码与失败在 `shallow-comparison-root-v1`，独立验收入口为 `shallow-comparison-root-verification-v2.json`。

## 入口调度事实与待证部分

固定 class 的异常行是 `[0,21)→26 any`。正常代码在 BCI19 读取 x、20 保存返回值、21 调用 cleanup、24 重载返回值、25 ireturn；handler 在26保存异常、27调用相同 cleanup、30重载同一异常、31athrow。BCI0 和8分别是外/内 natural-loop header，控制结构具有回边。

`region.rs` 的 `region_at` 先问 `shared_finally_candidate`，再读普通 loop，后面才执行 generic `guard::examine`。`guard.rs` 的单行 shared candidate 早筛只接 cleanup 首条为 `Load(slot0)` 的固定形状；本例 BCI21 是 invokestatic，且没有 lock acquisition，故不会接手。BCI0随即进入 loop reader，最终返回 LoopShape；后续 BCI5 的 generic guard 起点已晚于 protected start，`resources` 明确以 `start > proof.protected.0` 拒绝。root 实际读取对应源码，Luna 的只读分析与报告一致。

因此报告中的 finally-copy fallback 不证明 SavedReturn 证书本身失败：完整入口 BCI0没有先运行它。下一候选验证是让现有完整证书在 protected entry、尚未 visited 时取得判定，再复用 `finally_body` 的有界 child scope 和普通 loop reader。SavedReturn 已有保存值、两份 cleanup、Throwable 身份和完成优先级证明，暂不需要新的 finally 完成实体。静态事实不能保证调整顺序后 child loop成功；若仍回退，应分别定位证书和 loop条件，不能放松 ownership、范围或所有权计费。

准确实现位置：`region.rs::region_at` 约2588/2722/2743、`guard.rs::shared_finally_candidate` 约11113、`guard.rs::resources` 约15233、`region.rs::finally_body` 约5114。现有测试分别覆盖 ImplicitCleanup 分支/throw/saved return、Test2 三循环 void completion、Test5 多保护段多 saved return；它们不能充当此单保护段组合的精确验收。

N=33 同样提前回退，execution complete、explanation_only/fallback；没有 recursion_bound/Stop。它保留为失败探针，不计 depth-limit pass。当前 change 仍 **8/11**，2.4/3.1/3.2 保持未完成。
