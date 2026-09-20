## 1. 恢复正确性

- [x] 1.1 修正嵌套表达式的实际求值上下文（P3-R8）：用 `(x + 1) + ++x` 公开入口反例验证输入 7 时生成 Java 返回 16，或明确降级并保留完整依赖；无跨写入的嵌套算术、bump/doubleIt 与单次调用继续通过。把递归上下文退回 producer BCI 的变异必须使回归失败（A10、A13）。
- [x] 1.2 修正 fallback 的可观察生产者保留（P3-R9）：被拒 cast 前的 `getstatic External.value` 必须出现在可靠语句或低级引用中，BCI 0/3/6 有正确物理方法映射；FieldRecord.presented 与实际产物一致。覆盖初始化/异常、调用和字段链，删除 getstatic 的变异必须失败，保留原调用只执行一次及预算/取消回归（A12、A13、A14、A16）。

## 2. 独立验证与完成判定

- [x] 2.1 将 R8/R9 的受控 javac CLASS、来源/编译命令/摘要和正向对照纳入现有恢复语料及 fingerprint 清单；运行实际编译/执行对照，明确可执行 Java 的返回值/调用与 Mixed 的 effect/origin 验收分别成立；公开库/CLI 使用同一输入逐字段比较，保留 accessor/X1 和查询隔离回归（A10、A12、A13、A16、A17）。

  独立 oracle（jadx，最小范围）：对**同两个反例**做行为核对——用固定版本的外部 jadx 反编译受控 CLASS，编译其产物并执行，与原 CLASS 基准逐项比对（R8 `nestedLocal(7)`=16；R9 `fieldCast()=="ok"` 且 `External` 类初始化 1 次），记录 jadx 版本与命令。**只做行为对照，不做文本 diff**：jadx 会重写（实测 `(x + 1) + ++x` 被折成 `i + 1 + i + 1`），文本比较没有判别力。jadx 与 `javac` 同类，是**外部 oracle**：固定版本、需在 PATH、**不作为构建依赖**，缺席时明确报告而非静默跳过。**它通过不构成 jarde 正确性的证明**；只有不一致才是线索，且仍需受控确认。两者共享 JVMS 假设，共同误读不会被它暴露——该限制与 `verification.md` 已记录的「差分抓不到两侧共存的错误」同源。
- [x] 2.2 对修正后的固定提交执行 fmt、clippy、全量测试、两条显式 P3 编译执行对照、benchmark smoke、OpenSpec strict 与适用根/fuzz/MSRV/supply-chain/JDK25/依赖门禁，记录精确 CI SHA 和结果；两条反例与新引入回归均关闭后同步 delta、更新支持矩阵/完成判断并归档本 change。旧 CI 绿色不得作为新实现完成证据。
