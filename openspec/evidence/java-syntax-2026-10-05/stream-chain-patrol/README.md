# Stream 链巡查（2026-10-05 root——现代 Java 最惯用模式）

## 探针

[fixture/ST.java](fixture/ST.java)（`--release 8`）：`stream().map(String::toUpperCase).collect(toList())`（**unbound 实例方法引用**）、`filter(lambda).sorted().collect`、`Arrays.stream(int[]).filter(IntPredicate).sum()`（**IntStream 原生特化**）、`groupingBy(String::length)`、`joining(",")`。

## 结果：4/5 恢复 + 扩宽族新位点

- **unbound 方法引用** `String::toUpperCase` → `(Function) ((Object p0) -> ((String) p0).toUpperCase())`——lambda 包装+cast 精确；谓词 lambda（filter）内联；**IntStream 原生特化全链**（`Arrays.stream(new int[]…).filter((IntPredicate)((int a) -> …)).sum()`）完整恢复；`groupingBy` + 方法引用恢复——泛型 Signature 拒（raw 退化=既有域）但体全恢复；
- **`joined()` 拒**：`Collectors.joining(",")` 的 String→CharSequence 实参——"declared `java.lang.CharSequence` presents `java.lang.String`"——**与已立 CharSequence 扩宽片同因**，但 `Collectors.joining` 是 java.util.stream 方法不在 DIRECT_EDGES（java.util-only）表内——扩宽片实现时以本形为第 5 行位点（jadx 直接解）。

## 处置

`joined` 锚补入 recover-charsequence-argument-widening；其余不立项（负结果）。
