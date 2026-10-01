# 复合守护形态巡查：TWR 内显式 finally 与嵌套 synchronized（2026-10-02）

静态初始化域与 instanceof 链巡查均健康（K1/K2/K3、T4.pick 完整恢复且行为一致——顺带扩验记录）。固定 [fixture](fixture/)（T4 含两缺口形态 + K 系健康对照；SHA 见 [results/fixture-sha256.txt](results/fixture-sha256.txt)），行为基线 o4.out（`body[b]mid[a]`、`3:42:null`、`10`）。

## 结果矩阵

| 场景 | 主线 Jarde |
| --- | --- |
| `pick`：instanceof + cast 链（String/Integer/其它） | 完整恢复 |
| **`nested`：`try (a) { try (b) { body } finally { mid } }`——TWR 内显式 finally** | 整方法拒绝：`jre_guard_finally_copy`@64（"lacks the complete straight-body…"）→ 9 未覆盖块 |
| **`sync`：嵌套 synchronized（外层块内循环前再入内层 monitor）** | 整方法拒绝：`jre_guard_monitor`@4（"not entered once and left on every path"）→ 6 未覆盖块 |

## 根因

1. **TWR×finally 复合**：内层显式 finally 的副本候选（javac 两行 catch-all + 内层 close 已并入 TWR 降低）不满足 `prove_finally_copy` 的"完整直体+副本"形（其正文本身是 TWR 体的一部分、清理含资源 close），guard 家族证书两两独立、无复合通道。
2. **嵌套 monitor**：`monitor` 证书要求"enter 一次 + 每路径 exit 一次"；外层块内含**第二对** monitorenter/exit（内层块），被当作外层自身的不配对 exit 拒绝。单 monitor 对假设被合法嵌套打破。

## 切片划分（串行，同触 guard.rs monitor/finally 路径）

- **`recover-nested-monitor-regions`（先）**：monitor 证书接受体内完整第二对——外层扫描遇配对的内层 enter/exit（enter 在外层 enter 后、exit 配对且都在外层体内）时按嵌套呈现 `synchronized (…) { … synchronized (…) { … } … }`；不配对/跨出外层仍拒绝。T4.sync 恢复 `10`。
- **`recover-twr-inner-finally`（后）**：guard 家族复合——TWR 证书的正文证明接受"内层显式 finally"子证书（内层 finally_copy 在 TWR 正文语境下证明），呈现 `try (…) { try (…) { … } finally { … } }`。T4.nested 恢复 `body[b]mid[a]`。

原 class 为行为基准。
