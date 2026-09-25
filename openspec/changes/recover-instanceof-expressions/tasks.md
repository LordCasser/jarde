## 1. 固定表达式与静态类型输入

- [x] 1.1 从80行审计及type-boundaries收成最小Java8正面fixture、source-only helper/runner和独立negated汇合负例；原class与JADX实际编译/运行逐项记录，保留JADX四处不相容类型编译失败；root统一冻结hash、Code数和census。
- [x] 1.2 构造合法未使用结果、重复消费、旧局部值及boolean整数消费者边界，以-Xverify:all和真实SSA/BCI来源确认拒绝前提，不能用非法class误判恢复能力。证据：[五份定长 method_info 变体](../../evidence/java-syntax-2026-09-22/instanceof/refusal-boundaries/README.md)的 pop/duplicate/stale/retained/int class SHA 与预期输出由 root 从 `tests/p3_instanceof.rs` 当前补丁字节独立重建，五类均经 `java -Xverify:all` 执行；定向测试检查 `instanceof`、消费者及延期生产者的 BCI 来源，不能呈现的形状保留引用。2.3 的预算/取消与完整负边界验收仍独立未勾。

## 2. 接通类型测试表达式

- [x] 2.1 添加忠实instanceof操作事实与boolean表达式节点，复用CP目标、引用/数组拼写及发射优先级；解码、呈现及Not分组回归通过，不增加pass或层级resolver。
- [x] 2.2 以现有Cast保留合法左操作数静态类型上下文，接入共享boolean判据与最终消费点；Object/null不重复包装，具体引用安全上溯Object且内部checkcast不消失，复用已证明函数式工厂目标再加宽，局部/返回/分支/参数及不相容类型回归通过。
- [x] 2.3 复用延期消费和quoted生产者协议，验证丢弃、多用、失败消费者及调用/旧值效果；补真实BCI/成员、默认无来源和正文/来源预算停止回归。

## 3. 三方对照与独立验收

- [x] 3.1 实际恢复正面完整类原样重编译，与原class对照null、数组、相关/不相关类型、调用次数、CCE及producer自身抛错；不删除负面方法冒称混合类完整通过。
- [x] 3.2 root审读共享类型/消费边界并独立重放type-boundaries；复跑boolean/cast/调用/旧值/来源预算相邻回归，集中验census/fingerprint、fmt/clippy/OpenSpec strict，分别记录既存债务。
