## 1. 取证与基线

- [x] 1.1 当前 CLI 复跑固定 DF/DV，保存 debug/none、Java8/23 原/JADX/Jarde 完整类、物理异常表和来源。独立记录各方法的实际第一处拒绝；失败保留 no-runtime，不预设与旧 README 一致。
- [x] 1.2 按 DV.plainFor 的同次 IR 回答 Q-i/Q-ii，并分别核 DV.forEach/DF.loopCatch 的 Q-iii。完整 handler/row/header/join/普通与异常边及 terminal 路径已写入 `instrumentation.md`；retry 两变体基线与八lane同次树已独立接受，参考 JADX splitter 与区域所有权算法，`design.md` 已完成。观察接受不表示产品通过。

## 2. 实现

- [x] 2.1 在现有普通 catch 认证补齐 finite terminal-body 证明，由同一次事实供 join/scope/coverage/induction 使用；并在现有 Walker 证明 retry header/exit 的不同边界，复用 Endless/Try/LoopBreak/LoopContinue。保持原 CFG/异常派发/来源与停止计费；不得删边、禁用 overlap 或放宽 fragmented 特化门。
- [x] 2.2 对照组零回退：无 try、循环内 try-finally、fragmented-loop-catches fixture 文本不变。固定 DF.loopFinally 的 handler51 uncovered 独立记录，不能从此诊断推断 javac 失败；保留完整类真实编译/运行证据，不得删除方法或修补生成源码满足3.1。实际v5 DV/DF四组同原且八控制方法文本恒同；两fragmented完整text恒同旧CLI，见 results/control-text-and-transfer-map-root-v1。
- [x] 2.3 在现有 presented_reference_store_value 仅为同 block、单消费、无间隔效果的实际 reference Load 复用呈现证明，保留 constructor/call protected-arm 门和全读 SSA/phi/词法 owner 门；完成 foreach，不能以准确拒绝关闭三形。实际四DV整类运行同原；385永久tests含四lane原位Load、真实dup/间隔effect/constructor-static门及full-proof内部BCI2预算Stop，见 results/candidate-focused-and-comparison-root-v1。

## 3. 验证

- [x] 3.1 原/JADX/Jarde 双 JDK、双 profile 完整类独立编译运行，值/异常/效果同原；新增覆盖 JADX 无限重试/break 代表。历史 JADX 错码须在当前实测后分类，不能预定结果；拒绝边界准确保存。
- [x] 3.2 永久回归、fresh 双 seed workspace、fmt/当前 CI Clippy/global strict、预算/取消及 source-map 门禁；5 GiB/target1 GiB/1 秒守卫/default stack/finally clean。
- [ ] 3.3 提交推送确切产品、独立验收自身 CI，更新账本与 handoff，最后真实 HEAD 干净 main 交付；CF-18/71/612 完成数仅按单元完整证据更新。

2026-10-12：2.1/3.1已按字段子变更 [integrated-comparison-root-v1](../recover-ordered-field-concat-operands/results/integrated-comparison-root-v1/README.md) 关闭；八retry内部Stop/物理与transfer拒绝反例实际通过，40命令12整类全部同原。父项6/8，full workspace/当前CI Clippy/global strict/自身CI与干净main仍待完成。

2026-10-13：workspace-v5 发现 sharedHandler 的捕获异常经引用 Load 写入 Runnable 声明；2.3 重新打开，原 SHARED/ESCAPED 断言保留。修复复用 catch SSA 别名证明并逐写核类型，须由新 focused/完整类对照重新接受。历史运行证据不覆盖新产品。

2026-10-13：2.3 已按最终 final-check-root-v9 的391库测试、保持原断言的 scope 拒绝门和 parent-runtime-root-v5 的40命令/12完整类同原重新接受。新 Load 核 caught 别名和精确 named verifier 类型/java.lang.Object，补空phi/directCaught及简名Object边界；父项恢复6/8。完整工作区及交付门仍未关闭。

2026-10-13：workspace-v6 命令59发现 C2W.main 的引用Load证明回退，wrap-family完整类原断言失败；2.3再次打开，父5/8。finally clean 已实际成功。不得用局部门禁替代完整工作区，也不得放宽恢复断言。

2026-10-13：撤回全局Caught拒绝、保留handler入口ValueId身份证明后，focused-v10实际518成功/4ignored、原scope+wrap-family断言通过；fresh parent-root-v6的12完整类运行同原，新增C2W双JDK8份全类编译/16次main与Runner运行同原，2.3再次接受，父6/8。见字段子变更 results/wrap-load-regression-fix-and-final-local-root-v1；完整工作区及自身CI仍待完成。

2026-10-13：3.2 已按fresh focused-v12和完整workspace-v9关闭。八focused命令705/0/4、当前CI Clippy/global strict通过；两seed各96批/364targets、合计7050成功/0失败/196ignored，195命令全0、4854来源恒同、5GiB/target1GiB/1秒守卫/default stack/finally exact clean/target absent均独立核验。见 [完整工作区验收](../recover-ordered-field-concat-operands/results/full-workspace-accepted-root-v9/README.md)。自身产品CI和真实HEAD干净main仍待完成；不增加CF18/71/612完成数。

2026-10-13：精确产品7e93已推送，自身CI38297425449实际4jobs/52steps全部成功，独立v4 verifier核两seed各364targets/3525-0-98与4854来源恒同，见字段子变更 results/own-product-ci-7e93-root-v1。最终真实HEAD干净main尚待检查点审计，交付checkbox保持未关闭。
