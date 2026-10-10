## 1. 基线与证明范围

- [x] 1.1 root执行独立基线验收，核原/JADX六成功腿、Jarde四真实编译拒绝、完整outer/A成员与来源、33命令126文件，保留失败并准确归因。
- [x] 1.2 核同次compound proof/AST/field登记/class-family源码及JADX真正convertFieldArith guard，完成本design与严格有效的proposal/spec/tasks；不照搬结构等价receiver。

## 2. 最小实现

- [x] 2.1 Luna交付private patch，root审后只扩准确imul+Multiply及既有AssignOp/field_write/read登记，真实Rust正例核*=、3字段access与完整BCI，旧add/sub/array恢复不回退。
- [x] 2.2 对抗与停止用例通过：显式双receiver读保持普通赋值，字段/宽度/消费者不满足时不乘法折叠，预算/取消保持既有Stop；测试如实记录实际停止阶段。

## 3. 完整类与主线验收

- [x] 3.1 冻结新CLI/产品与测试pins，原形outer/A完整源码双JDK/default-all重编-Xverify/raw一致，精确核全部物理方法/字段/OriginSet及class-family derived范围；小组溢出/null/effect对照通过，不手改生成源码或借原helper。
- [ ] 3.2 fmt/CI同范围Clippy/OpenSpec strict及相关回归通过，仅在20GiB机器/1GiB本仓target守卫满足后执行Rust；提交推送后独立验收本片确切产品CI双seed/JDK25/MSRV/fuzz/supply。
- [ ] 3.3 更新handoff/71账本与任务的真实完成范围，提交推送干净main，清本仓编译残留并保留全部源/class/raw/冻结CLI，不计EM23整单元完成。
