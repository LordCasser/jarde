# CF-17 巡查：TWR 正文语句形态与包围具名 catch（2026-09-30）

[CF-15 巡查](../cf15-crossing-patrol/README.md)登记的 `twrNamed` 缺口（TWR + 外层具名 catch）在主线 `19371ab7` 上展开为两个独立根因。固定转录 [fixture](fixture/)（T1/T2/T3，SHA 见 [results/fixture-sha256.txt](results/fixture-sha256.txt)）；基线 JSON 与实验捕获在 [results](results/)。上游对照组：`P5TwoResources`（void 调用体 + try 内 return）恢复正常。

## 结果矩阵（主线 19371ab7）

| 场景 | 形态 | 主线 Jarde |
| --- | --- | --- |
| T2.voidBody / voidBodyReturnInside | TWR 体为 **void 调用**（return 位置无关） | 恢复（`try (T2 local0 = new T2()) { touch(local0); }`） |
| T2.popBody / popBodyVoidTouch | TWR 体含**结果被 pop 的非 void 调用**（`r.toString();`） | 整方法回退："local 0 crosses a quoted fallback region" |
| T3.voidNamed / voidNamedRecover | TWR（void 体）+ **外层具名 catch 行** `[0,36)→38` | 整方法回退：`jre_guard_unexplained_row`@0 |
| T1.twrVoidNamed / C4.twrNamed | 两根因叠加 | 整方法回退 |

## 根因

1. **CF-17a：TWR 正文语句子集不收"调用语句"（statement-expression）。** `guard.rs` TWR 体证明的语句子集覆盖 void 调用与 return，不覆盖**非 void 调用 + pop**（Java 的 `r.toString();` 语句形态）。该形态在真实代码极常见（任何把方法调用当语句写的 TWR 体）。
2. **CF-17b：包围具名 catch 行被 `enclosing_clauses` 显式排除。** `guard.rs::enclosing_clauses`（约 13506 行）过滤条件 `!(row.start_bci <= shape_start && row.end_bci >= handler_end)` 把"覆盖整个 TWR 降低的具名行"排除在包围子句之外——而 javac 对 `try (r) { … } catch (E e) { … }` 的具名行恰好覆盖 init 到 cleanup 全部。主线上该形状因此 `jre_guard_unexplained_row` 拒绝（规则的保守选择：防止把 TWR 伪装成用户 catch 丢清理语义）。**scratch 实验**（[results/T3-voidNamed.exp-relaxed.json](results/T3-voidNamed.exp-relaxed.json)）：仅放宽该排除后 guard 拒绝消失、TWR claim 成立，但呈现层不发射 catch 子句 → 具名 handler 块 `[38]` 未覆盖 → 方法仍回退。即 17b 需要"guard 容忍 + catch 子句呈现 + handler 块覆盖"三步，实验证实了第一步方向。

## 切片划分（串行实施，同触 TWR 证明流）

- **CF-17a（`recover-twrcall-statement-bodies`）**：体语句子集接受"同块非 void 调用 + 紧随 pop"（呈现为调用语句），闭合 popBody 家族。**已落地**（root 验收于合并主线 aa279ec6：popBody 恢复 `local0.toString();`、void 对照逐字不变、四负例拒绝、全仓 2717/0）。
- **CF-17b（`recover-enclosing-named-catch`）**：包围具名行容忍（实验方向）+ Plan/Region/Builder 发射 `} catch (E e) { … }` 子句并覆盖 handler 块；守卫条件：行须完整覆盖 claim 跨度、handler 在 claim 外、claim 自身行完整。**已实施**（[enclosing-17b](enclosing-17b/README.md)）：全跨度具名行 + 单直行 handler 证明后，T3 两形、T1.twrVoidNamed/twrPopNamed、C4.twrNamed 与 `Guarded.withCatch` 全部恢复；六负例（catch-all 包围行、部分跨度、own-handler 交叠、分支 handler、双 catch、multi-catch）保持既有拒绝逐字不变。已知边界：handler 体把参数传给形参收紧的静态方法（槽复用类型决策）属 Catches 扩展片域。

**新登记巡查点（2026-10-01，17a 验收时发现；同日已闭合）**：TWR saved-return 呈现不可编译——`voidBodyReturnInside` 输出 `Object local1 = "in"; return local1;`，方法返回位是 `String`，`return local1`（Object）不可编译。根因（[诊断记录](saved-return-typing/README.md)）：frame 层 `ldc` 常量推 `RefType::Unknown`，`written_type` 未按值生产者细化——与 Test2 已修的 `array_of_value` 同族。已由 [recover-saved-return-value-typing](../../../changes/recover-saved-return-value-typing/) 闭合（root 验收于 0e1ef5ca：`java.lang.String local1 = "in";`、整类可编、null/Class 边界正确、全仓 2726/0）。

上游对照与既有证书（多资源、可空资源、multi-resource-twr change）零回退是两片共同门禁。原 class 为行为基准；JADX 参照不作为语义正例。
