# 递归泛型界与调用点推断巡查（2026-10-05 root）

## 健康面（负结果）

[fixture/RG.java](fixture/RG.java)（`--release 8`）：
- **递归界类 Signature 投影成功**：`abstract class RG$Node<T extends java.lang.Comparable<T>>`——自引用界**完整投影**（含 field `T val`）；
- 具体化子类体：`((Integer) this.val).compareTo((Integer) arg1.val)`——擦除字段 cast + compareTo 恢复良好；
- 泛型方法本体 `max`：raw 呈现（`Comparable` 擦除参数）+ Signature 拒绝注释（既有域）+ `compareTo((Object) arg1)` 如实；
- `Collections.singletonList` 泛型工厂消费恢复。

## 发现一：String/Integer → Comparable 实参扩宽缺失（第 7 个新证缺口，与 CharSequence 同族同机制）

`max("a","b")`/`max(1,2)` 调用点拒绝："the parameter 0 … is declared `java.lang.Comparable` presents `java.lang.String`/`java.lang.Integer`"——与 [CharSequence 巡查](../charsequence-arg-widening-patrol/README.md) 完全同因：`platform_reference_argument_widens` 的 `DIRECT_EDGES` 只覆盖 java.util，java.lang 侧仅 Throwable。Comparable 的 java.lang 实现者封闭集 = String + 8 个装箱类型（Byte/Short/Integer/Long/Float/Double/Character/Boolean）——有界可枚举（用户类的 Comparable 实现走 snapshot 通道——其层次在 jar 内）。

## 发现二：合并头片新锚（`$` 拒绝家族）

`RG$IntNode extends RG$Node` 呈现**裸形**（`RG$Node<Integer>` 参数化头缺失；class Signature `LRG$Node<Ljava/lang/Integer;>` "not projected"）——父类二进制名含 `$`，正是合并头片（`recover-parameterized-interface-headers` 的父类池形 MVP）要放宽的两处 `$` 拒绝家族的又一实例。

## 处置

Comparable 缺口登记第 7 个 + 立姊妹窄片（与 CharSequence 片同机制同落点，可合并派发）；RG$IntNode 锚补入合并头片 proposal。

## 处置（2026-10-06，change `recover-comparable-argument-widening` 落地后重渲染）

两个推断调用点恢复（`refusals = 0`）：[`results/jarde-RG-after-comparable-argument-widening.txt`](results/jarde-RG-after-comparable-argument-widening.txt)
的 `callGen` 写出 `max((java.lang.Comparable) "a", (java.lang.Comparable) "b")`、`callGen2` 写出
`max((java.lang.Comparable) java.lang.Integer.valueOf(1), (java.lang.Comparable) java.lang.Integer.valueOf(2))`
——装箱是既有域，装箱后的 `Integer` 实参入表判定（九行含八装箱）。`RG$IntNode.cmp` 的
`((java.lang.Integer) this.val).compareTo((java.lang.Integer) arg1.val)`（本巡查判定的健康面）逐字不变；
`RG$IntNode` 的裸父类头（合并头片）与 `Collections.singletonList` 消费面同样不变。
