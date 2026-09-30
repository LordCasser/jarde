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

- **CF-17a（`recover-twrcall-statement-bodies`）**：体语句子集接受"同块非 void 调用 + 紧随 pop"（呈现为调用语句），闭合 popBody 家族。
- **CF-17b（`recover-enclosing-named-catch`）**：包围具名行容忍（实验方向）+ Plan/Region/Builder 发射 `} catch (E e) { … }` 子句并覆盖 handler 块；守卫条件：行须完整覆盖 claim 跨度、handler 在 claim 外、claim 自身行完整。17a 落地后实施。

上游对照与既有证书（多资源、可空资源、multi-resource-twr change）零回退是两片共同门禁。原 class 为行为基准；JADX 参照不作为语义正例。
