# 嵌套构造作构造实参巡查（2026-10-02）

异常 cause 链域巡查引出的构造呈现缺口（主线 `ed727b0d`）。固定转录 [fixture](fixture/)（X1：cause 链三形态；X2：判别探针；SHA 见 [results/fixture-sha256.txt](results/fixture-sha256.txt)）。

## 结果矩阵

| 场景 | 主线 Jarde |
| --- | --- |
| `fmt(new Exception("solo"))`——单参构造作**方法**实参 | 恢复 |
| `String.valueOf(new StringBuilder("sb"))`——同上 | 恢复 |
| `new RuntimeException("wrapped", e)`（throw 位，非嵌套） | 恢复（throwable-wrap 切片） |
| `readCause(new Exception("top", new Exception("inner")))`——**构造实参位是另一完整构造** | 拒绝：`jre_new_shape` "the constructor at BCI 12 is called on a value this allocation did not produce" → 整方法退化（X1.main 第三行 `inner` 缺失、X2.nested 同） |

## 根因

`crates/jarde-java/src/init.rs` 的构造站点走查（`Sites`/BCI 0 的 `new; dup; …; invokespecial` 序列证明）：外层构造从自己的 dup/实参推送扫到 invokespecial 时，**实参位上先完成了一个完整内嵌构造**（`new; dup; ldc; invokespecial`@6–12）；走查把内层 ctor 当作外层构造的调用点，接收者检查（"allocation did not produce"）误拒。即走查未建模"实参生产者可以是（递归证明的）完整构造站点"。

高频形态：异常包装链（`new X(msg, new Y())`）、包装对象构造、builder 初始值。

## 处置方向

`recover-nested-ctor-argument-sites`：构造站点走查接受实参位的嵌套构造——外层实参扫描遇 `new; dup; …; invokespecial` 完整内嵌站点时递归证明之（单用途、值恰为外层该实参），跳过该区间继续外层序列；内嵌站点呈现为构造实参表达式（复用构造呈现，嵌套拼 `new Y(…)` 于实参位）。X2.nested/X1.main 恢复；单参作方法实参、throw 位等既有形态逐字不变；内嵌非完整站点（无 ctor/双用途/跨块）保持拒绝。

原 class 为行为基准。

## 实施结果（`recover-nested-ctor-argument-sites`）

走查已扩展：外层实参扫描按同一站点判据递归证明内嵌构造（深度上限 2），成功则跳过其闭区间续扫并由既有构造拼写呈现于实参位；内嵌非完整站点（双用途、跨块、三层）保持拒绝并登记。X1.main/X2.nested 及变体前后、三方对照与门禁记录见 [results-nested/nested-replay.md](results-nested/nested-replay.md)（变体源与 SHA 在 [variants-nested/](variants-nested/)）。
