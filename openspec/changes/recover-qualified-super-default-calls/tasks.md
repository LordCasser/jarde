## 1. 取证与基线

- [ ] 1.1 重放固定 F1 家族 fam.jar（SHA 核对）：读 build.rs:21191 附近 interface-special 证明选择，确认失败环节与消费点；记录双形基线。
- [ ] 1.2 构造并冻结至少四个 verifier 有效变体/负例：菱形+自身覆写混合、带参/void 默认方法限定调用、限定符经 extends 间接（拒绝）、目标 abstract（拒绝）；各自 `java -Xverify:all` 通过并记录实现前后行为。

## 2. 快照事实证明与呈现

- [ ] 2.1 interface-special 证明接入快照 header/成员事实（design 决策 1–2）；F1$Diamond/F1$Reabstract$Impl 恢复、F1 家族整 jar 重编运行一致（`AB`/`I:A`）；负例保持拒绝。
- [ ] 2.2 普通 super/this、invokestatic 接口方法、既有 invocation 通道 diff 断言逐字不变；预算/取消原子性不变。

## 3. 回归与验收

- [ ] 3.1 `cargo test --workspace --tests --locked --no-fail-fast` 全绿（含 snapshot-hierarchy-widening、member-family、super/this 系既有测试）、fmt、CI 完整 30 项 allowlist clippy、`openspec validate --all --strict`、diff check；磁盘纪律同前。
- [ ] 3.2 F1 家族与变体三方对照：原 class/固定 JADX Java-input/Jarde 重编 `java -Xverify:all` 逐路径一致；记录输出 SHA。
- [ ] 3.3 root 独立复核直接性判据、成员事实读取与三方行为，更新 DT/EM 账本与巡查记录。
