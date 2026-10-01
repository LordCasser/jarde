# 平台集合层级实参上转型巡查（2026-10-02）

泛型/集合域巡查（主线 `96b1606f`）。固定转录 [fixture](fixture/)（G1：泛型方法 `max(List<T>)`/`pick(T,T)`、`ArrayList` 构建、`Integer` 装箱循环；SHA 见 [results/fixture-sha256.txt](results/fixture-sha256.txt)），行为基线 `zeta:a`/`6`。

## 结果矩阵

| 场景 | 主线 Jarde |
| --- | --- |
| `pick("a","b")`（Object 形参 + 结果 cast） | 恢复 |
| `new ArrayList<String>()` 构建与 `add`（Object 位） | 恢复（显式 cast 呈现） |
| **`max(xs)`：实参 `ArrayList` → 形参 `List`** | 拒绝："declared `java.util.List` presents `java.util.ArrayList` but … no safe reference conversion evidence"（BCI 36）→ 结果局部级联拒绝 → 整方法带引用残留 |
| `for (Integer i…)` 装箱循环 | 基本恢复（别名存储后 `aload;pop` 残留一处，BCI 44——次要点） |

## 根因

`crates/jarde-java/src/build.rs::platform_reference_argument_widens`（约 24386 行）的平台白名单当前仅 `List → Iterable` 一对（java_release 8）；集合实现类到接口（`ArrayList→List`、`HashMap→Map` 等）与接口到超接口（`List/Set→Collection`、`Collection→Iterable`）都落入拒绝。与已合入的 `java_lang_throwable_widens`（45 对闭集 + 传递闭包 walk，recover-throwable-wrap-arguments）同一通道、同一形态——**只是表未覆盖 java.util 集合层级**。

## 处置

`recover-platform-collection-widening`：`platform_reference_argument_widens` 的 java.util 集合闭集扩展——JDK 8 `java.util` 固定层级直接边（ArrayList/LinkedList/Vector→List；HashMap/TreeMap/LinkedHashMap/Hashtable→Map；HashSet/TreeSet/LinkedHashSet→Set；ArrayDeque/LinkedList→Deque/Queue；接口链 List/Set/Queue→Collection→Iterable），与 throwable 同款传递闭包 walk；逐对以 JDK 运行时 `getSuperclass`/接口表机械核对。用户类/非 java.util 不放宽（升级路径同前：resolution 层证明）。

次要观察（不在本片）：装箱循环别名存储的 `aload;pop` 残留——与 discarded-call 家族相邻但形态不同（被丢弃的是 load 而非调用结果），按需另片。

原 class 为行为基准。
