## 1. 固定名称改绑的实际反例

- [x] 1.1 将默认包 arg0 的 static field read/write 与外部 static call 对照整理为最小永久 Java8 主类，helper/runner source-only；保存完整原/JADX/jarde 基线、hash/Code，并保留 null/非 null 下成员目标和双方字段值的逐行结果。
- [ ] 1.2 补充 debug 包首段 java、arg0/arg0_2 连续冲突和无冲突对照；未使用 CP owner 不应影响名字。真实字段遮蔽与方法引用保持独立 source-only 边界，root 统一冻结 census/fingerprint。

## 2. 扩充既有命名约束

- [ ] 2.1 从当前方法已解码的静态 owner 与字段 claim 有界收集可拼写类型路径首段，合并既有 final 字段 reserved 输入；以默认包、包前缀、同类无 qualifier 及无关 CP 测试验证范围，不新增解析器或 pipeline 阶段。
- [ ] 2.2 沿用 NameTable 和已统一非槽名称候选，覆盖默认/debug/分段局部/后缀冲突；保留 raw 与 Collision 证据，确认声明/引用一致且不会通过删除 owner 或伪接收者改写成员语义。
- [ ] 2.3 为本项新增收集、条目和冲突工作接入既有预算/取消，实际测试低预算/取消/多冲突停止；验证 essential/all 正文相等、真实 BCI/CP/member 来源及 source-map replay 同名。

## 3. 完整执行与 root 验收

- [ ] 3.1 原样重编译实际恢复完整类，执行冻结 fixture 及 root 两份12项反例和包前缀3项；校验静态返回值、双方字段、null 参数、无冲突名字，不手改恢复文本或删除成员。
- [ ] 3.2 root 独立审读名称约束来源及限定符选择，复跑 final-static、static-call、field、invocation、alias、lambda、deferred-order 和命名/预算相邻回归；新增方法引用或类级成员债务单列。
- [ ] 3.3 root 统一 Java 包、census/fingerprint、fmt、严格 clippy 和 OpenSpec strict；记录确切既存门禁债务，不把本次可编译范围扩大成项目级编译承诺。
