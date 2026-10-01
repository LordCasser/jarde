## 1. 基线与负例

- [x] 1.1 重放固定 N0/N1/N3：核 fixture SHA、N0 折叠基线与 N3 两探针/N1 全量形态的逐字段降级输出；在 `enum_constants.rs` 确认 descriptor 白名单拒绝点与 ctor 体/常量步骤证明的参数化点并记录。
- [x] 1.2 构造并冻结至少五个 verifier 有效负例/变体：实参种类不符（int 位是 ldc 字符串）、ctor 体额外语句、>3 用户参、char 参数窄化拼写、null 对象参；各自 `java -Xverify:all` 通过并记录实现前后行为。

## 2. grammar 参数化与折叠

- [x] 2.1 构造 grammar 扩展（design 决策 1–2）：N3$Simple/N3$Refs/N1$Numbers 全量折叠、整类 `javac --release 8` 通过且运行与 fixture 基线（n1.out/n3.out）逐字一致；N0 与四固定形输出逐字不变（diff 断言）。
- [x] 2.2 呈现拼写（design 决策 3）：`(byte) 1` 窄化、`NumString.ONE` 限定名、char/null 变体钉死；负例保持逐字段呈现。

## 3. 回归与验收

- [x] 3.1 `cargo test --workspace --tests --locked --no-fail-fast` 全绿（含全部 enum 相关切片：enum-constants/string-arg-enum-constant-bodies/enum-constructor-delegation/enum-user-initializer-suffix/enum-int-arguments/nested-enum-source 等）、fmt、CI 完整 30 项 allowlist clippy、`openspec validate --all --strict`、diff check；磁盘低于 15Gi 先 `cargo clean`，完成即清。
- [x] 3.2 N1/N3 与变体三方对照：原 class/固定 JADX Java-input/Jarde 重编 `java -Xverify:all` 逐路径一致（含嵌套类一起编译）；记录输出 SHA。
- [ ] 3.3 root 独立复核 grammar 边界、ctor 体纪律与三方行为，更新 DT-13/账本（收窄挂起项为匿名体形态）与巡查记录。
