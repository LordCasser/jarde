# return 穿副作用 finally 巡查（2026-10-05 root）——新窄缺口

## 判别矩阵

| 形 | 结果 |
|---|---|
| continue 穿副作用 finally（循环内） | 恢复（等价分布：finally 双路径内联） |
| break 穿副作用 finally（循环内） | 恢复（分布 + break 保留） |
| return 穿**空** finally | 恢复 |
| finally **返回值**形（早前巡查） | 恢复（if/else 等价） |
| **return 穿副作用 finally**（循环内 noCall/finReturn） | **拒**（整方法） |
| **return 穿副作用 finally**（**无循环** loopless） | **拒** |

边界：**try 返回 + finally 落穿带副作用** = 全位置拒绝（推翻"循环外 finally-return 健康"的简化印象——那是 finally 返回值形）。

## 语义序基准（FO.java）

`try { return bump(); } finally { i = 100; }` → `1/100`：**return 表达式先求值（一次）、finally 副作用后执行**——朴素把 finally 体复制到 return 语句**之前**会错序；正确形需要 temp 绑定（`t = bump(); i = 100; return t;`）。这是本片的核心可证性要求。

## jadx 三方

jadx 以 **finally 体复制到各路径**解决（loopless 行为精确 `3000/107/3000`——与 jarde 既有 finally-return/continue 分布等价法**同族**：机制已存在，缺这一员）。jadx 对含调用 return 表达式是否保序未验证到 FO 形（我们的 spec 以 FO 为判别测试）。

## 归属

诊断文本（local crosses quoted fallback region）与 local-scope 家族同；呈现等价法与 finally-return/continue 分布同族。spec 要求实现者插桩两关系（同门则合并）。

## 处置

登记 + 立项 `recover-return-through-effectful-finally`（MVP：单 return 出口+单副作用 finally 先行；FO 求值序为判别锚）。
