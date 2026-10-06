# Stream 链巡查（2026-10-05 root——现代 Java 最惯用模式）

## 探针

[fixture/ST.java](fixture/ST.java)（`--release 8`）：`stream().map(String::toUpperCase).collect(toList())`（**unbound 实例方法引用**）、`filter(lambda).sorted().collect`、`Arrays.stream(int[]).filter(IntPredicate).sum()`（**IntStream 原生特化**）、`groupingBy(String::length)`、`joining(",")`。

## 结果：4/5 恢复 + 扩宽族新位点

- **unbound 方法引用** `String::toUpperCase` → `(Function) ((Object p0) -> ((String) p0).toUpperCase())`——lambda 包装+cast 精确；谓词 lambda（filter）内联；**IntStream 原生特化全链**（`Arrays.stream(new int[]…).filter((IntPredicate)((int a) -> …)).sum()`）完整恢复；`groupingBy` + 方法引用恢复——泛型 Signature 拒（raw 退化=既有域）但体全恢复；
- **`joined()` 拒**：`Collectors.joining(",")` 的 String→CharSequence 实参——"declared `java.lang.CharSequence` presents `java.lang.String`"——**与已立 CharSequence 扩宽片同因**，但 `Collectors.joining` 是 java.util.stream 方法不在 DIRECT_EDGES（java.util-only）表内——扩宽片实现时以本形为第 5 行位点（jadx 直接解）。

## 处置

`joined` 锚补入 recover-charsequence-argument-widening；其余不立项（负结果）。

## 处置（2026-10-06，change `recover-charsequence-argument-widening` 落地后重渲染）

第 5 位点恢复（`refusals = 0`）：[`results/jarde-ST-after-charsequence-argument-widening.txt`](results/jarde-ST-after-charsequence-argument-widening.txt)
的 `joined` 写出 `(java.lang.String) names().stream().collect((java.util.stream.Collector) java.util.stream.Collectors.joining((java.lang.CharSequence) ","))`
——CharSequence 表按**目标类型**命中，与方法所属包无关，故 `java.util.stream` 的方法与 `java.lang` 位点同一判据；
巡查记录的其余四形（unbound 方法引用、IntStream 特化、lambda 谓词、groupingBy）逐字不变。
