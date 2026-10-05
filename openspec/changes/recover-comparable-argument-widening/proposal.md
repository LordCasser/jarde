## Why

[递归泛型巡查](../../evidence/java-syntax-2026-10-05/recursive-generic-patrol/README.md)实证：泛型方法调用点 `max("a","b")`/`max(1,2)`（擦除后参数声明 `java.lang.Comparable`）拒绝——"the parameter 0 … is declared `java.lang.Comparable` presents `java.lang.String`/`java.lang.Integer`"。与 [CharSequence 缺口](../charsequence-arg-widening-patrol/README.md)**完全同因**：`platform_reference_argument_widens` 的 `DIRECT_EDGES` 只覆盖 java.util 集合树，java.lang 侧仅 Throwable 族。

**为什么平台表是必须的**（与用户类不同）：`String`/`Integer` 等 JDK 类的层次不在被反编译 jar 的快照内——snapshot 通道（参数自身快照陈述的链）对平台类型**结构性不可用**，这正是 java.util 表与 Throwable 通道存在的原因；Comparable 是同一性质的第三张表（CharSequence 第四张，见姊妹片）。

**实现者封闭集（release 8 javadoc，java.lang 内实现 Comparable 的类型）**：`String` + 8 个装箱类型（`Byte`/`Short`/`Integer`/`Long`/`Float`/`Double`/`Character`/`Boolean`）——9 行有界表。用户类的 Comparable 实现**不在本片**（其层次在 jar 内，snapshot 通道已覆盖）。



> **root 追加（2026-10-05，[generics-edge 巡查](../../evidence/java-syntax-2026-10-05/generics-edge-patrol/README.md)）**：平台扩宽族第 3 员 **Serializable**（多重界调用点 `both("a","b")` 同因拒）已登记——三员（CharSequence/Comparable/Serializable）同机制同落点，**实现时作为同一表族的三张封闭表合并处理**（javadoc 各自核对实现者集；数组型的 Serializable 走 snapshot 待插桩）。

## What Changes

与 [recover-charsequence-argument-widening](../recover-charsequence-argument-widening/) **同机制同落点**的姊妹表：`java.lang 装箱型与 String → java.lang.Comparable` 的 9 行封闭表，命中即 `cast_argument` 呈现。两片可合并派发（同一文件、同一判定函数族、同一验证协议），但保持独立 change 目录以便独立回滚。

## Impact

- **代码**：与 CharSequence 片相同（`platform_reference_argument_widens` 并列区）。
- **测试**：`RG` fixture（`callGen`/`callGen2` 主锚）+ 姊妹通道零回退 + 负例（非 Comparable 实现者仍拒）。
- **账本**：summary.md 登记行关闭。

## Non-Goals

- **不**做 `Comparator`/其它函数式接口参数位（未实测，先取证）；
- **不**动用户类 snapshot 通道；
- **不**把两片合并成一个 change（独立回滚边界）。
