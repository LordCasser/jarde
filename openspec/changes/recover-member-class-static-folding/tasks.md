## 1. 取证与基线

- [x] 1.1 重放固定 M1/M2（SHA 核对）：读 `FamilyRootScan` 多子拒绝位与 facade 投影装配消费；确认枚举折叠名字重写机械的复用面与孙代/嵌套行顺序；记录双家族基线（M1 refused reason、M2 无折叠）。
- [x] 1.2 构造并冻结至少四个 verifier 有效变体/负例：接口型子、子类 extends 兄弟接口、孙代 `Inner$Leaf`（不折叠登记）、非静态子（保持分离）；各自 `java -Xverify:all` 通过并记录实现前后行为。

## 2. 多子扫描与折叠投影

- [x] 2.1 静态行列表化（design 决策 1）+ 折叠嵌入与名字重写复用（决策 2）；M1/M2 家族整 jar 重编通过、运行与基线逐字一致（`hi`/`ok`、`7`）；折叠作用域内类头/字段/throws/体内引用为源码拼写。
- [x] 2.2 分离呈现（逐类单类输出）、匿名/枚举投影通道 diff 逐字不变；单类输入负例保持；预算/取消原子性不变。

## 3. 回归与验收

- [x] 3.1 `cargo test --workspace --tests --locked --no-fail-fast` 全绿（含 enum 折叠全家族、member-family 身份测试、nested-spelling 全部既有测试）、fmt、CI 完整 `-A` clippy、`openspec validate --all --strict`、diff check；磁盘纪律同前。
- [x] 3.2 M1/M2 与变体三方对照：原 class/固定 JADX Java-input/Jarde 重编 `java -Xverify:all` 逐路径一致；记录输出 SHA。
- [x] 3.3 root 独立复核折叠序、名字重写边界与三方行为，更新 DT 账本（嵌套声明里程碑）与巡查记录。（root 于合并主线 9910e88c 复核：M1 折叠五嵌套声明（含 `static interface Ctrl` 隐式 static 抽象接口行准入）、域内 `static Deep deepField`/`throws Err`/`new Deep()` 全源码拼写、M1/M2 整 jar 重编行为逐字一致（`hi`/`ok`、`7`）；全仓 2870/0、fmt/openspec 251/251。投影复用复核认可：`java_code_name_spans`+source-map 锚定+重排四件套四处泛化（CP 条目扩面/覆盖段放宽/Exceptions 声明门/field_texts 通道）；单子走既有窄证书回退、匿名/枚举/注解通道优先与防线（`X$1` 家族拒绝、Signature 拒绝）保持。遗留登记：孙代域内池拼写、带引注子类照常折叠、初始化/数组辅助投影保守拒绝。）
