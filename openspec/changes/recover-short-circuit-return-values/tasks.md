## 1. 冻结证据与调用边界

- [x] 1.1 核对 `MixedLocalReturn` 的源码/class SHA、原/JADX 完整类 Java 8 重编和八路径 `java -Xverify:all` 结果；从当前 Jarde CLI 重新确认缺返回与全部 BCI 来源，不将字段图实现前的缓存当现状。
- [x] 1.2 审计 `ShortCircuitValue` 的 region、SSA/Phi、field consumer 与 emitter 所有调用者、预算/取消；固定 `ireturn Z`、`ireturn I`、额外入口、异常边、第二消费者/独立效果的准入与拒绝对照。

## 2. 同一短路图的返回消费

- [x] 2.1 在现有 bounded 图 recognizer 中只增加直接 `ireturn` 消费锚点，保留两个 producer 的精确正常前驱、无额外入口/异常边和一次提交 owner。验证：返回正例一个 owner；外部入口仍 whole-body fallback。
- [x] 2.2 将现有 SSA/消费者证明限定扩至方法返回描述符 `Z`、opcode `ireturn`、唯一同槽 Phi use 与精确 `1/0` 输入；拒绝 `I`/不明类型、第二消费、独立效果及预算停止。验证：负例没有部分 `return`。
- [x] 2.3 复用 `Conditional` 和 `Return` 发射惰性值，沿已证明 BCI 映射测试、goto、producer、ireturn，并保留方法后缀。验证：完整类 Java 8 重编，八行返回值与两 RHS 次数逐字一致，方法 `structured/java` 无引用。

## 3. 独立验收

- [x] 3.1 复跑现有两/三测试短路字段、混合字段 16 路径、额外入口/异常边/双消费负例及 loop/try/jsr 相邻控制，确认只扩展直接返回消费者。
- [x] 3.2 运行定向测试、格式、适用 Clippy、`openspec validate recover-short-circuit-return-values --strict`，记录真实通过/失败与 Cargo target 清理；仅证据支持时勾选，不自动归档。
