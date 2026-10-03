# Lambda 呈现巡查（2026-10-03）——合成方法名冲突致整类不可编

lambda/method-ref 域巡查（主线 `c7b7b767`）。固定转录 [fixture](fixture/)（Y1：用户接口 lambda、Supplier/Function、removeIf/sort 双参、捕获局部；SHA 见 [results/fixture-sha256.txt](results/fixture-sha256.txt)），行为基线 orig.out（`hi!`/`45`/`[b, aa]`/`8`）。

## 现状矩阵

| 场景 | 主线 Jarde |
| --- | --- |
| lambda 恢复（invokedynamic + LambdaMetafactory） | **健康**——零引用，lambda 语法呈现（含双参、捕获、接口 cast 包装） |
| `String::length` 方法引用 | 行为级恢复（lambda 体内联 `((String) p0).length()`），`::` 语法未呈现 |
| **lambda 表达式 + 合成 `lambda$…` 方法同时呈现** | **不可重编**：javac 为 lambda 重合成同名方法 → "符号 lambda$viaLambda$0(String) 与 Y1 中的 compiler-synthesized 符号冲突" → **整类编译失败**（Y1 固定复现，jx/Y1.java 编译记录） |

## 根因

呈现策略是"lambda 调用伴生合成方法"（忠实字节码：invokedynamic 引用 MethodHandle(lambda$…)+伴生方法体）。但 javac 源码层对 lambda 表达式**自身**会再合成 `lambda$名$N`——显式呈现的伴生方法与其**命名空间冲突**，类不可编。任何含 lambda 的类都命中（Java 8 核心形态）。

## 处置方向

`recover-lambda-inline-bodies`（大颗粒，源码形状）：伴生 `lambda$…` 方法的**体**内联进 lambda 表达式（MVP：单表达式/直线体——参数按位绑定、无 return 语义差），伴生方法隐藏；复杂体（多语句/控制流）MVP 保守——伴生保留但改用**非冲突名**（如 `lambda$…$jarde` 后缀，private static 合成语义不变），保整类可编。方法引用 `X::m` 语法呈现为独立后续（行为已对）。

原 class 为行为基准。

## 处置结果（2026-10-03，`recover-lambda-inline-bodies` 落地）

实现侧取证、变体前后转写、Y1 命中输出、三方对照与 corpus 双腿扫描见 [results-li/](results-li/README.md)：
Y1 四位点内联 + 伴生省略，整类 `javac --release 8` 重编通过、`java -Xverify:all` 输出与 orig.out 逐字一致；
复杂体（分支/多语句）伴生重命名 `lambda$…$jarde`；多用途伴生保持现呈现并登记。
