## 1. 冻结家族正例与拒绝边界

- [x] 1.1 固定无 `Outer.super` 方法桥的 Java 8 命名成员家族，含同类型 `other`/捕获 Outer、非 public 成员构造器及字段 getter；记录源码、class/JAR 哈希、`javap`、原/JADX/Jarde 完整类编译与 `-Xverify:all` 结果，并用校验和复核证据。**验收**：[阶段一证据](../../evidence/java-syntax-2026-09-26/named-member-family-stage1/README.md)与 [Root 复核](verification-1.1.md)固定原/JADX 的 `2011/20`、身份阶段 Jarde 根独编的 `20` 和根/成员合编失败；不将 `prepared` 冒充可编译家族。
- [x] 1.2 准备双向 `InnerClasses` 错配、同名多物理定义、非捕获同类型接收者、额外字段使用/写入、异常边界与预算/取消的 verifier-valid 或 proof-unit 负例；逐项记录证据等级、期望拒绝位置和保留的物理来源。**验收**：[1.2 负例与边界记录](verification-1.2.md)列明各样本、拒绝点、保留来源、验证命令和局限。

## 2. 精确家族身份与同请求准备

- [x] 2.1 从已绑定根定义读取双方 typed 嵌套事实，在选定环境中唯一解析 child，证明非静态命名关系和 Java 修饰符来源；以正例及冲突/缺失/local/static 反例验证不按 `$` 或 raw `ACC_PUBLIC` 猜测。
- [x] 2.2 把单物理类准备/恢复抽为可在同一请求预算下顺序调用的内部过程，让 root/child 报告分别保留 definition、成员表、coverage、execution 和停止；以低预算/取消/同名多来源测试证明不重复重置总账或合并 BCI。

## 3. 捕获与构造的局部证明

- [x] 3.1 证明唯一 synthetic/final Outer 字段、物理构造器首参、构造 prologue 写入及 child 读取的 SSA 身份闭包；用 `other` 同类型正反例、额外写入/读取和无法证明的 `this()` 链确认只投影捕获值。
- [x] 3.2 对家族内成员构造调用复用现有 `new@1` 的实例、null-check、参数与效果顺序证明，并用已证嵌套关系判断非 public 成员的源码可访问性；用 null、前置/内部效果及错关系负例验证拒绝不删除调用来源。**验收**：[3.2 证据等级与边界](verification-3.2.md)。

## 4. 家族文本和派生来源

- [x] 4.1 让 writer 在一个 package/根头之下写经证明的成员声明、构造器简名和完整方法体，仅按证明结果隐藏捕获字段/隐式首参/写入；以 Java 8 完整类重编、`-Xverify:all` 比对原/JADX 的值、效果、异常，检查无重复声明或被吞掉的方法。**验收**：[4.1 家族文本与三方执行复核](verification-4.1.md)。
- [x] 4.2 在根报告保留 child 物理报告及独立 coverage/source map，并为隐藏构件和改写的 `Outer.this` 记录 caller/child/字段/构造器的派生投影；以两个 owner 都有 BCI 0、默认/all 输出及拒绝样本核对映射与 JSON 身份。**验收**：[4.2 派生来源与物理身份复核](verification-4.2.md)。
- [x] 4.3 对本阶段不能证明的 `access$` super 桥、外部使用、部分成员恢复和预算停止保留物理文本及显式未闭合状态；以冻结 `20:10:1:3` 变体确认未将普通 `Outer.value()` 冒充 `Outer.super.value()`。**验收**：[4.3 外部消费者与拒绝边界](verification-4.3.md)。

## 5. 独立验收

- [ ] 5.1 Root 用重建 CLI 独立重放正例和负例，运行相关 Rust/Java 回归、`cargo fmt --all -- --check`、`git diff --check`、`openspec validate assemble-proved-member-class-family --strict`，核对公开报告的身份、预算和来源并清理隔离 Cargo target；未验收的相邻架构债务另案记录。
