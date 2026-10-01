## 1. 取证与基线

- [x] 1.1 重放固定 B1–B5（SHA 核对）：javap 确认拼接消费的实际 append descriptor（Z vs 装箱 Object）；读短路值切片实现定位消费方枚举点与 "no SSA proof" 拒绝路径；记录判别链三档基线。
- [x] 1.2 构造并冻结至少三个 verifier 有效负例/变体：布尔装箱拼接（`"" + hasA` 头位）、双短路双拼接、拼接前局部被二次读（保持退化）；各自 `java -Xverify:all` 通过并记录实现前后行为。

## 2. 消费方扩展

- [x] 2.1 消费方集合纳入拼接 append 位（design 决策 1，descriptor 按取证）；B5.s1/s2、B4.v1–v3、B2.compound 全恢复、整类重编运行一致；B3/s3 diff 断言逐字不变；负例保持退化。
- [x] 2.2 装箱/头位变体边界正确（按既有装箱通道呈现）；预算/取消原子性不变。

## 3. 回归与验收

- [x] 3.1 `cargo test --workspace --tests --locked --no-fail-fast` 全绿（含 short-circuit 全家族与 concat 切片）、fmt、CI 完整 30 项 allowlist clippy、`openspec validate --all --strict`、diff check；磁盘低于 15Gi 先 `cargo clean`，完成即清。
- [x] 3.2 B5/B4/B2 三方对照：原 class/固定 JADX Java-input/Jarde 重编 `java -Xverify:all` 逐路径一致；记录输出 SHA。
- [x] 3.3 root 独立复核消费方边界与三方行为，更新 EM-19/CF-20 账本与巡查记录。（root 于合并主线 d4dad468 复核：实现者因模型配额终止于机械收尾段，root 亲自完成收尾验证——全仓 2771/0〔两轮各 1 例已知 flake 单跑干净〕、fmt/openspec 236/236、完整 30 项 allowlist clippy 干净；B5.s1/s2 恢复为布尔赋值 + 拼接、B2/B4 全恢复、B5/B2 重编运行与 fixture 基线逐字一致、B3/s3 diff 不变。实现要点复核认可：append 为原生 `(Z)` 非装箱（javap 取证）、双链形态经 region tail 内嵌 + 路径登记修复、`p3_hoisted_boolean` 计费回漂修正。JADX 腿为 harness 包名机械问题非产品缺陷。）
