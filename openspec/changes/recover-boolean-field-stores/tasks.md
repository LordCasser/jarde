## 1. 固定 Z 独立语料

- [x] 1.1 root 从完整字段审计独立重放冻结 CLI：补丁 class 49 Code 不变、原/补丁 `-Xverify:all` 成功；380 行中 B/C/S 285 行全同，Z 95 行有 70 行不同，jarde `javac` 成功但空操作改变字段/调用/异常；记录真实 JVMS/JLS 语义与现有字段路径。
- [x] 1.2 冻结最小 Z Java 8 source-only fixture/runner，固定真实 descriptor、Code、奇偶/极值、实例/静态、producer 与 null，保留普通 boolean 证明对照及预实现 RED；root 独立重放 `put-field-fixture-48ed/`，40 行中当前 jarde 26 行不同，JADX 完整类编译失败。
- [x] 1.3 把最小补丁 class 与 runner 固定为永久 Rust fixture/test，断言真实 descriptor、Code 哈希、来源及修前 RED；只能在源/补丁输入侧裁剪，不能编辑反编译输出。

## 2. 字段消费位置表达低位

- [x] 2.1 在 `field_value` 只对真实 Z 字段或 verified accessor 的已呈现整数构造现有 `%` 与 `!=` AST；已有 boolean 证明/0/1保持旧路径，未知与不相容值拒绝，不改通用转换。
- [x] 2.2 校验 AST 静态类型、括号、receiver/value 顺序、producer 一次与 null/异常身份；不能让转换成为独立语句或提前 null 检查。
- [x] 2.3 固定直接 put/producer 来源，以及 verified accessor 的 call 与 callee put 来源；默认/all同正文、预算/取消、未验证 accessor 与非Z负例，失败保留现有 `quoted_bcis`/accessor 拒绝。

## 3. 整类对照与 root 验收

- [x] 3.1 原样重编译并执行永久完整类，Z 所有正面行与补丁 JVM 同值、同调用、同异常且 jarde 零引用；JADX仅在原样整类 `javac` 成功时比较运行。
- [x] 3.2 root 独立重放完整 380 行、审读准入/来源/求值顺序，复跑 B/C/S field、boolean contexts、return、array、invocation 相邻回归；`ireturn Z` 与 boolean[] `bastore` 仍独立。证据：`root-z-after-8cd1/` 的 380 行零差异/零引用、最小 `root-after-8cd1/` 的 40 行零差异；相关 Cargo 与四组 JDK 整类测试通过。`p3_narrow_integer_returns` 两项为既存规划 RED，冻结 908c CLI 同样引用 directByte。
- [x] 3.3 root 统一 reader census/fingerprint、fmt、适当 Cargo 与 OpenSpec strict，完成证据后勾选。reader 实测并重钉 (105,694,81,236,8) 后通过；fingerprint 仅新增八个文件、重生后 5 通过/1 ignored；fmt、diff check、strict 54/54 通过。
