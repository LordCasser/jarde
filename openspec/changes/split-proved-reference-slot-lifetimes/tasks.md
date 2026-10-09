## 1. 冻结与因果

- [x] 1.1 root 独立核对八类32腿的输入、JDK/CLI/JADX hash、全部源集/命令/双流/exits、槽复用 javap 和隔离运行；永久保存不可变快照，16 no-debug 失败、8不同名 LVT成功、8同名 LVT失败分别统计；首轮 harness 文件名失败不计能力结果。（`evidence/baseline-root-verification.json`：1368文件、432实际命令、32原/JADX全源运行双流一致；`baseline-slot-lines-root.json`：32/32方法确有两条astore_2，16 no-debug无LVT；不可变archive SHA见snapshot.json。）
- [x] 1.2 实现者复核现有 reuse/声明/命名链，记录至少数组→集合和对象→对象两例的 SSA store/read owner、精确类型来源与段界，证明不是 generics/constructor/concat 引起的拒绝；以 baseline 源文本与完整重编日志核验。

## 2. 实现与边界

- [x] 2.1 在现有引用/数组分段所有者中补齐无 LVT 普通引用规则，复用唯一 owner、旧值 uses、LocalVariable 与声明路径；加入正常 CFG 无后返、预算/取消与未知/null 来源门；聚焦测试确认八类十六 no-debug 生成独立声明且无引用错误类型，existing array/Ref-Int/monitor/LVT 源文本保持。
- [x] 2.2 冻结并实测至少六族边界：同类型多写、参数/header、未知/null、cross-phi、旧值跨段栈使用、后段返回前段/handler；新规则不错误分段，旧恢复/拒绝与 source-map anchors 保持；essential/all 源文本相同，低预算/取消不发布半计划。

## 3. 完整验收

- [x] 3.1 root 对32永久输入重放候选：16 no-debug 完整类两 JDK 隔离重编且 `-Xverify:all` 行为与原/JADX一致；另16 debug腿保持旧源文本与结果；保存实际 runner/工具/输入/源码 hash、双流及exit，不删拒绝成员、不借原jar、不覆盖历史失败。
- [x] 3.2 root 运行单共享 Cargo 全仓固定seed门禁、MSRV1.88、fmt、CI-exact Clippy、strict OpenSpec、diff check及预算/语料指纹；记录真实新增计费并核对无无关退化，磁盘可用空间不得低于20GiB，完成后清理Cargo残留。
- [ ] 3.3 root 对抗审查 SSA/CFG/类型来源与所有负例、验收上述完整对照；更新 EM-20/summary/handoff 的本片边界，不宣称完整LG/EM-20完成；提交推送并核对实际 main CI。
