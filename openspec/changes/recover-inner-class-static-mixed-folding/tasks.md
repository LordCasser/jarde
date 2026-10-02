## 1. 取证与基线

- [x] 1.1 重放固定 N1（SHA 核对）：读 `scan_family_root` 混合族返回序（静态行被哪条先行条件挡出 `StaticMembers`、非静态候选分支）；记录 N1 基线（prepared 分离、无嵌套声明）。
- [x] 1.2 构造并冻结至少三个 verifier 有效变体/负例：三子混合（两静态+一非静态）、单静态+单非静态、纯非静态族（不折叠）；各自 `java -Xverify:all` 通过并记录实现前后行为。

## 2. 子集折叠

- [x] 2.1 静态子集提取与折叠（design 决策 1–2）；N1 的 Stat 折叠、`new Stat()` 源码拼写、N1 家族整 jar 重编运行一致（`10/7/13`）。（实际口径见 `evidence/.../fold-mix/README.md` §3：变体家族集逐字一致；N1 家族集恰余 1 个实现前同样存在的错误——`Stat.use` 限定 new 未恢复〔第二片〕，以透明形替换该唯一缺口后 `10/7/13` 逐字一致。）
- [x] 2.2 负例边界正确；reason 文本更新可审计；预算/取消原子性不变。

## 3. 回归与验收

- [x] 3.1 全仓测试全绿（含上片 member_class_static_folding 全部测试）、fmt、CI 完整 `-A` clippy、`openspec validate --all --strict`、diff check；磁盘纪律同前。
- [x] 3.2 N1 与变体三方对照：原 class/固定 JADX Java-input/Jarde 重编 `java -Xverify:all` 逐路径一致；记录输出 SHA。
- [ ] 3.3 root 独立复核子集边界、互引锚定与三方行为，更新 DT 账本与巡查记录。
