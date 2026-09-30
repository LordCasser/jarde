## 1. 基线与负例

- [ ] 1.1 重放固定 T2：核 fixture SHA、popBody/popBodyVoidTouch 主线回退与 voidBody 恢复对照；记录可重放基线。
- [ ] 1.2 构造并冻结至少四个 verifier 有效负例：调用结果被局部接收（无 pop）、pop 读值非该调用结果、调用与 pop 间插入指令、pop 后同表达式续读；各自 `java -Xverify:all` 通过并记录实现前后行为。

## 2. 体语句子集扩展

- [ ] 2.1 Guard 正文语句枚举新增"非 void 调用 + 紧随 pop"分支（design 决策 1）；呈现走调用语句通道；固定类命中、1.2 负例保持拒绝、void 体对照逐字不变；预算/取消原子回滚。
- [ ] 2.2 变体族（纯调用语句体、void+调用语句混排、try 内 return 前后调用语句、静态/虚/接口调用各一）逐项恢复且行为等价。

## 3. 回归与验收

- [ ] 3.1 `cargo test --workspace --tests --locked --no-fail-fast` 全绿（含 TWR 家族、多资源、可空资源零回退）、fmt、CI 同款 Clippy 新码零新增、`openspec validate --all --strict`、diff check；磁盘低于 15Gi 先 `cargo clean`，完成即清。
- [ ] 3.2 T2 与变体族三方对照：原 class/固定 JADX Java-input/Jarde `javac --release 8` 重编，`java -Xverify:all` 正常与注入异常路径逐路径一致；记录输出 SHA。
- [ ] 3.3 root 独立复核判据、呈现与三方行为，更新 CF-17 清单与巡查账本；C4.twrNamed 仍拒绝属 17b。
