## 1. 基线与负例

- [ ] 1.1 重放固定 C2.alias：核 fixture SHA、catch 体两条被引语句与 `no safe reference conversion evidence` 诊断、级联的 local3 声明拒绝；记录可重放基线。
- [ ] 1.2 构造并冻结至少六个 verifier 有效负例：用户自定义异常类 → Throwable（表外，保持拒绝）、非祖先 java.lang 对（如 `String → Throwable`）、原始/引用混形、数组异常类型、构造参数 0（消息位）类型错配、重载多候选下的窄化要求；各自 `java -Xverify:all` 通过并记录实现前后行为。

## 2. 闭集上转型实现

- [ ] 2.1 在 `build.rs` 实参转换分派中，数组/overload/platform 回答之后增加 java.lang Throwable 闭集回答（design 决策 1–2）；呈现保留要求类型；以 C2.alias 命中（catch 体三语句完整、方法无被引语句、整类可重编）与 1.2 负例保持拒绝验收。
- [ ] 2.2 包装重抛变体族（RuntimeException/IllegalArgumentException/IOException 包装、嵌套双层包装、用户静态 `log(Throwable)` 收异常实参、多实参位次）逐项恢复且行为正确；既有转换回归（Object 目标/数组闭集/`reference_overload_calls`/List→Iterable）逐项不变。

## 3. 回归与验收

- [ ] 3.1 `cargo test --workspace --tests --locked --no-fail-fast` 全绿（新增回归入 `p3_` 测试文件族）、fmt、CI 同款 Clippy 新码零新增、`openspec validate --all --strict`、diff check；磁盘低于 15Gi 先 `cargo clean`，完成即清。
- [ ] 3.2 C2.alias 与变体族三方对照：原 class/固定 JADX Java-input/Jarde 完整 Java 8 类重编，`java -Xverify:all` 正常路径与注入异常路径（包装消息、cause 链、双层包装 identity）逐路径一致；记录输出 SHA。
- [ ] 3.3 root 独立复核闭集表对 JDK 层级的逐对正确性、呈现与负例，更新 CF-15/EM-10 清单与巡查账本。
