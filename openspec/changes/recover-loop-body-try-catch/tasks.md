## 1. 取证与基线

- [x] 1.1 当前 CLI 复跑固定 DF/DV，保存 debug/none、Java8/23 原/JADX/Jarde 完整类、物理异常表和来源。独立记录各方法的实际第一处拒绝；失败保留 no-runtime，不预设与旧 README 一致。
- [x] 1.2 按 DV.plainFor 的同次 IR 回答 Q-i/Q-ii，并分别核 DV.forEach/DF.loopCatch 的 Q-iii。完整 handler/row/header/join/普通与异常边及 terminal 路径已写入 `instrumentation.md`；retry 两变体基线与八lane同次树已独立接受，参考 JADX splitter 与区域所有权算法，`design.md` 已完成。观察接受不表示产品通过。

## 2. 实现

- [ ] 2.1 在现有普通 catch 认证补齐 finite terminal-body 证明，由同一次事实供 join/scope/coverage/induction 使用；并在现有 Walker 证明 retry header/exit 的不同边界，复用 Endless/Try/LoopBreak/LoopContinue。保持原 CFG/异常派发/来源与停止计费；不得删边、禁用 overlap 或放宽 fragmented 特化门。
- [ ] 2.2 对照组零回退：无 try、循环内 try-finally、fragmented-loop-catches fixture 文本不变。固定 DF.loopFinally 的 handler51 uncovered 独立记录，不能从此诊断推断 javac 失败；保留完整类真实编译/运行证据，不得删除方法或修补生成源码满足3.1。
- [ ] 2.3 在现有 presented_reference_store_value 仅为同 block、单消费、无间隔效果的实际 reference Load 复用呈现证明，保留 constructor/call protected-arm 门和全读 SSA/phi/词法 owner 门；完成 foreach，不能以准确拒绝关闭三形。

## 3. 验证

- [ ] 3.1 原/JADX/Jarde 双 JDK、双 profile 完整类独立编译运行，值/异常/效果同原；新增覆盖 JADX 无限重试/break 代表。历史 JADX 错码须在当前实测后分类，不能预定结果；拒绝边界准确保存。
- [ ] 3.2 永久回归、fresh 双 seed workspace、fmt/当前 CI Clippy/global strict、预算/取消及 source-map 门禁；5 GiB/target1 GiB/1 秒守卫/default stack/finally clean。
- [ ] 3.3 提交推送确切产品、独立验收自身 CI，更新账本与 handoff，最后真实 HEAD 干净 main 交付；CF-18/71/612 完成数仅按单元完整证据更新。
