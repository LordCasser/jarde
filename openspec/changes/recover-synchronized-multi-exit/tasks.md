## 1. 冻结完整类与边界

- [x] 1.1 冻结 `synchronized-multi-exit/` 自写 Java 8 完整类646B/4Code、SHA-256 `6658190aa7575f8be5095d71aa3ee07aa453d464666b1d34dea6a962950c45f2` 与 runner；原/JADX 三项 `-Xverify:all` 输出相同，Jarde 六处引用、整类缺return。root 复制到 `/tmp/jarde-sync-multi-root-4sq6hO` 独立重放，summary 逐字节相同。
- [ ] 1.2 固定两臂不同锁、缺少正常/异常退出、保护范围漏掉值生产者、竞争 handler 与额外出口的 JVM 合法或受控负样本；先验证其 class 可执行，区分可恢复与必须拒绝，记录精确 BCI/来源。

## 2. 受已证 handler 限制的区域闭环

- [ ] 2.1 在现有 monitor 规则中证明两条正常退出、同一锁、共享重抛 handler、两段完整保护与所有可达路径恰好一次退出；保持单出口回归，负边界完整拒绝。
- [ ] 2.2 让 Plan/Region 在已认证 handler 边界下承载同步正文的双臂结构，复用现有 If/返回表达式发射；仅本 Plan 已证明的异常边可由子 walk 忽略，保留递归深度、预算、取消和 fused-block 半开区间归属。
- [ ] 2.3 在各臂原返回消费点呈现值并单次认领生产者，异常路径保留同一异常对象；默认/完整来源正文相同，真实 BCI/成员涵盖进入、分支、两臂值/退出/返回与异常退出/重抛。

## 3. 整类执行与主代理验收

- [ ] 3.1 完整 Engine/CLI 原样生成并编译/执行 fixture，对照原 class/JADX 三行及 1.2 负样本；复跑单出口 synchronized、资源与 typed catch 回归，不能通过手改生成源码或删除失败方法验收。
- [ ] 3.2 root 独立审查 certified handler 边界、分支归属和拒绝来源，冻结重建 CLI 重放整类；跑 Rust/Java、reader census/fingerprint、fmt、Clippy、OpenSpec strict 与磁盘核查，分开登记既存债务。
