## 1. 冻结来源与边界

- [x] 1.1 以 [正例 replay](../../evidence/java-syntax-2026-09-25/static-method-local-generic-throws/replay.py) 的 `--mode baseline` 在 `-g`/`-g:none` 下核对原/JADX/Jarde 的完整类、反射、强类型调用、版本与摘要；退出 0 且 [分析](../../evidence/java-syntax-2026-09-25/static-method-local-generic-throws/analysis.md) 记录差异。
- [x] 1.2 构造返回来源/副作用、方法异常界和本类 Methodref 等负例，重放已证拒绝与物理回退，覆盖两份严格 JVM 验证可装载的伪造 Signature；root 独立重放 [negative/replay.py](../../evidence/java-syntax-2026-09-25/static-method-local-generic-throws/negative/replay.py) `--mode recovered` 退出 0，逐方法错误码见[分析](../../evidence/java-syntax-2026-09-25/static-method-local-generic-throws/negative/analysis-negative.md)。

## 2. 方法局部异常的原子声明投影

- [x] 2.1 在现有静态直接返回泛型方法路径复用 reader 签名/擦除证明与同轮 `GenericReturnCandidate::Parameter`，对严格方法局部单 `throws X`、JDK Throwable 根界、类层级与调用绑定作源级门，组成完整头后一次发布；root 独立运行[正例 replay](../../evidence/java-syntax-2026-09-25/static-method-local-generic-throws/replay.py) `--mode recovered` 两种 debug 退出 0，完整类重编、反射及调用均通过。
- [x] 2.2 定向 Rust 回归覆盖局部方法变量/异常（含参数/返回/异常同一 `T`）、无调试表、错误界、泛型预算/取消；reader 签名回归与负例 replay 覆盖未绑定/擦除冲突，副作用和本类调用继续拒绝；相邻普通静态泛型、无正文和空 `void` 泛型异常保持通过。root 独立 125 项定向测试与两份 replay 均通过，见[独立验收](verification-root.md)。

## 3. 独立验收与收尾

- [x] 3.1 root 审读声明/正文/层级/调用证明及原子预算路径，独立运行三方正反例、相关 Rust 测试、格式/静态检查与 `openspec validate recover-proved-static-method-local-generic-throws --strict`，结果见[独立验收](verification-root.md)；本项私有编译产物已清理。
