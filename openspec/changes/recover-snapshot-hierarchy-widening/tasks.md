## 1. 取证与基线

- [x] 1.1 重放固定 I1（fam.jar，SHA 核对）：定位 `reference_overload_calls` 生产 pass 挂点与类 header（super/interfaces）读取面；记录 BCI 39 基线与通道结构事实。
- [x] 1.2 构造并冻结至少五个 verifier 有效变体/负例：两级继承到接口、多实现接口、匿名类传接口、快照内无关联两类（拒绝）、final 类传 Object（既有分支）；各自 `java -Xverify:all` 通过并记录实现前后行为。

## 2. 证明集合与分派

- [x] 2.1 per-BCI snapshot-hierarchy 证明（design 决策 1–2：pass 收集、有界 walk、计费）；build.rs 分派消费（平台对之后）；I1 四路径完整恢复、重编行为一致（`hi:d`/`hello:d`/`static`/`hello:v`）。
- [x] 2.2 变体逐项恢复；负例保持现拒绝文本；平台闭集/数组/同名/Object 分支顺序与行为零回退。

## 3. 回归与验收

- [x] 3.1 `cargo test --workspace --tests --locked --no-fail-fast` 全绿（含 throwable-widening、collection-widening、array-invocation-widening、reference-overload 全部既有测试）、fmt、CI 完整 30 项 allowlist clippy、`openspec validate --all --strict`、diff check；磁盘低于 15Gi 先 `cargo clean`，完成即清。
- [x] 3.2 I1 与变体三方对照：原 class/固定 JADX Java-input/Jarde 重编 `java -Xverify:all` 逐路径一致；记录输出 SHA。
- [x] 3.3 root 独立复核通道结构、walk 边界与三方行为，更新 EM-06/10 账本（勾销两处登记的升级路径首例）与巡查记录。（root 于合并主线 310e5350 复核：I1 调用点呈现 `viaInterface((I1$Greet) new I1$En(), "v")`、提交内测试完成真实编译运行对照（四行 SHA=orig.out=JADX）；全仓 2798/0〔首跑 1 例已知 flake 单跑/复跑均过〕、fmt/openspec 239/239。实现者结构发现复核认可：既有 overload 通道 walk 本就能走快照 header 链，真缺口是单引用参门——抽出共享 `snapshot_header_chain_widens` 而非复制；classpath 定义构造性不可达由读取面保证。单边快照外继续登记。）
