## Why

[SB 巡查](../../evidence/java-syntax-2026-10-05/charsequence-arg-widening-patrol/README.md)实证：`static String join(String[] xs){ return String.join("-", xs); }` 整方法拒绝——"the parameter 0 of the invocation at BCI 3 is declared `java.lang.CharSequence` presents `java.lang.String`"。**jadx 完整恢复**同调用（有解）。

**根因（root 零构建读码）**：实参引用扩宽的三条既有通道——`platform_reference_argument_widens` 的 `DIRECT_EDGES`（**只覆盖 java.util 集合树**，build.rs ~24948 注释自述"its `java.util` counterpart"）、`java_lang_throwable_widens`（java.lang 的 Throwable 族）、`snapshot_hierarchy_widenings`（参数自身快照陈述的链）——都不含 `java.lang.CharSequence` 接口族。`String`/`StringBuilder`/`CharBuffer` 实现 `CharSequence` 是 javadoc 级平台事实，与集合表同一性质。

**高频命中面**：`String.join(CharSequence, CharSequence…)`、`Appendable.append(CharSequence)`、`CharSequence` 参数位的一切 JDK8 API。



> **root 追加（2026-10-05，[generics-edge 巡查](../../evidence/java-syntax-2026-10-05/generics-edge-patrol/README.md)）**：平台扩宽族第 3 员 **Serializable**（多重界调用点 `both("a","b")` 同因拒）已登记——三员（CharSequence/Comparable/Serializable）同机制同落点，**实现时作为同一表族的三张封闭表合并处理**（javadoc 各自核对实现者集；数组型的 Serializable 走 snapshot 待插桩）。



> **root 追加锚（2026-10-05，[stream-chain 巡查](../../evidence/java-syntax-2026-10-05/stream-chain-patrol/README.md)）**：`Collectors.joining(",")` 的 String→CharSequence 实参（java.util.stream 方法——java.lang 族扩宽在 DIRECT_EDGES java.util-only 表外的第 5 位点）；同巡查证健康面：unbound 实例方法引用（String::toUpperCase→lambda 包装）、IntStream 原生特化全链、groupingBy——扩宽片实现时同名 javadoc 行补入。



> **root 追加（2026-10-05）**：第 4 员 [recover-enum-argument-widening](../recover-enum-argument-widening/)（`EnumSet.of` 枚举常量——**首个类位（java.lang.Enum 父类）扩宽**，接口三表外的同构新表）已立项；合并派发范围由三表扩为**四表**。

## What Changes

在 `platform_reference_argument_widens`（或并列的同族小函数，实现者按代码结构定）新增 **java.lang.CharSequence 实现者封闭表**：`java.lang.String → java.lang.CharSequence`、`java.lang.StringBuilder → java.lang.CharSequence`、`java.nio.CharBuffer → java.lang.CharSequence`（javadoc 声明的实现者全集；release 8 无新增）。命中即 `cast_argument` 呈现——与既有两条通道**完全同构**，零新机制。

## Impact

- **代码**：`crates/jarde-java/src/build.rs` 实参扩宽区（锚点名 `platform_reference_argument_widens` / `java_lang_throwable_widens` 附近并列）。
- **测试**：`SB` fixture（巡查已冻结）+ 集合/Throwable 扩宽既有测试零回退 + 负例（非 CharSequence 实现者的 String→接口形仍拒）。
- **账本**：summary.md 登记行关闭。

## Non-Goals

- **不**扩其它 java.lang 接口（`Comparable`/`Runnable` 等）——逐一按实测取证后再说（Comparable 泛型形有独立复杂性）；
- **不**动 `DIRECT_EDGES` 集合表与 Throwable 通道；
- **不**做用户类的接口扩宽（snapshot 通道的域）。
