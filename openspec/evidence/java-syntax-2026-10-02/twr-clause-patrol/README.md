# TWR 子句形态巡查：自带 finally 与 catch 体调用语句（2026-10-02）

[compound-guard 巡查](../compound-guard-patrol/README.md) 登记遗留的定向取证（主线 `92b3d47c`）。固定转录 [fixture](fixture/)（P1 复合体 / P2 判别 / P3 void 体判别；SHA 见 [results/fixture-sha256.txt](results/fixture-sha256.txt)），行为基线 orig.out。

## 结果矩阵

| 场景 | 主线 Jarde |
| --- | --- |
| TWR 体经调用分支（`boom()` 内 if-throw）、TWR 直体调用 | 恢复 |
| 17b 冻结 fixture（TWR+包围 catch，体为 `return`） | 恢复（无回归） |
| **A：TWR 直接携带 finally 子句**（`try (r) { … } finally { … }`，void 体 P3 / 复合体 P1 同败） | 整方法拒绝：`jre_guard_finally_copy`（finally 副本候选抢占失败） |
| **B：TWR+包围 catch，catch 体含调用语句**（`catch (e) { log.append("E"); }`——17a 的丢弃调用支持未进包围子句白名单） | 整方法拒绝（P1.twrCatch 因体分支与 B 叠加先败于 body；P2/P3 判别钉死 B 单变量） |

## 字节码锚（取证）

A（P3.voidBodySoloFin）：TWR 降低（close 正常/异常副本@13/21 + suppression@30）后接 **finally 降低**（正常副本@35-44 goto 59、异常副本@47-58 athrow）——TWR 语句与 finally 子句两套副本串联，任一现有证书单独都不拥有全表。
B：17b 的 `enclosing_clause` 证明白名单（return/调用/局部语句）中"调用"仅收 void 形；`log.append("E");`（append 返回值 pop）未入。

## 切片划分（串行，同触 guard.rs clause 通道）

- **`recover-twr-direct-finally`（先）**：TWR 语句自带 finally 子句——finally 降低作为 TWR 证书的尾随子证书（两套副本串联表一并拥有），呈现 `try (…) { … } finally { … }`。
- **`recover-enclosing-catch-call-bodies`（后）**：包围子句白名单接受丢弃调用语句（17a 判据接入 enclosing_clause 的语句枚举）。

原 class 为行为基准。
