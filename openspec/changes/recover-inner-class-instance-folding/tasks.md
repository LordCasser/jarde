## 1. 取证与基线

- [ ] 1.1 重放固定 N1（SHA 核对）：三项取证（design Context）——ctor 首参/this$0 序、access$000 桥签名族与体、限定 new 降低与拒绝码；读 mixed 装配缝；记录 N1 基线。
- [ ] 1.2 构造并冻结至少五个 verifier 有效变体/负例：捕获多字段族、桥写形（setter）、深一层链式 `a.new B().new C()` 形、分离呈现对照（逐字不变断言）、非直通桥体（不隐藏）；各自 `java -Xverify:all` 通过并记录实现前后行为。

## 2. 折叠与消隐

- [ ] 2.1 非静态子折叠 + this$0 消参消字段 + 构造限定形呈现（design 决策 1–2；限定 new 按取证结论处理）；N1 家族集重编运行一致（`10/7/13`）。
- [ ] 2.2 access 桥消隐与调用位重写（决策 3，读形 MVP）；非直通桥保守呈现；分离呈现与纯静态族 diff 逐字不变；预算/取消原子性不变。

## 3. 回归与验收

- [ ] 3.1 `cargo test --workspace --tests --locked --no-fail-fast` 全绿（含两片折叠、capture-ctor 分离家族、synthetic-ctor 序全部测试）、fmt、CI 完整 `-A` clippy、`openspec validate --all --strict`、diff check；磁盘纪律同前。
- [ ] 3.2 N1 与变体三方对照：原 class/固定 JADX Java-input/Jarde 家族集重编 `java -Xverify:all` 逐路径一致；记录输出 SHA。
- [ ] 3.3 root 独立复核消隐语境边界、限定形判定与三方行为，更新 DT 账本（嵌套声明第二层）与巡查记录。
