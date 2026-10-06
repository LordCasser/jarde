# Objects 助手 + EnumSet 巡查（2026-10-05 root）

## 发现：EnumSet.of 枚举实参扩宽=平台扩宽族第 4 员（首个**类位**扩宽）

`EnumSet.of(Flag.A, Flag.C)` + retainAll + contains——**整方法响亮拒绝**（"parameter 0 … declared `java.lang.Enum` presents `OB$Flag`"）。`EnumSet.of(E, E...)` 擦除 `(Enum, Enum[])`，javac 发 `checkcast java/lang/Enum`——与 CharSequence/Comparable/Serializable（接口位）同机制、**父类位**新表行。jadx 直接解。已立 recover-enum-argument-widening（与三张姊妹表合并派发 → 四表）。

## 健康面（负结果）

- `Objects.equals/hashCode/toString(o,dflt)/deepEquals` 全恢复（`(Object)` cast 如实）；手写 null 守卫 hash 恢复；
- `Objects.requireNonNull(o, () -> "…"+nanoTime())` supplier lambda 形恢复（companion 内联）；
- `EnumSet.noneOf(Flag.class)`（Class 形参——无扩宽）在 main 正常。

## 处置

Enum 扩宽立项；其余不立项。行为 `true/false/0/0/dflt/true/hasA/no`。

## 处置（2026-10-06，change `recover-enum-argument-widening` 落地后重渲染）

`flags` 整方法恢复（`refusals = 0`）：[`results/jarde-OB-after-enum-argument-widening.txt`](results/jarde-OB-after-enum-argument-widening.txt)
写出

```java
java.util.EnumSet local1 = java.util.EnumSet.of((java.lang.Enum) OB$Flag.A, (java.lang.Enum) OB$Flag.C);
local1.retainAll((java.util.Collection) arg0);
return local1.contains((java.lang.Object) OB$Flag.A) ? "hasA" : "no";
```

**两行机制分工（实测判别）**：BCI 6 的 `java.lang.Enum` 位由**快照单边证明**落地——`OB$Flag` 是快照内的类，
其 class-file header 逐字写着 `java/lang/Enum`（同一容器内存在 `OB$Flag.class` 时该行即消失；`--policy
single-class` 去掉快照内的枚举类后该行逐字回归，见 change 的 verification）；本片新表承担的是**级联伴行**
`EnumSet presents java.util.Collection`（BCI 12）——枚举族的 `java.util.EnumSet` 自身 javadoc 行（extends
`AbstractSet`，implemented-interface 列表达 `Set`/`Collection`/`Iterable`）。故 OB 锚**不需要** `java.lang.Enum`
表行（用户枚举类的名字无法封闭枚举，且该位已有证明通道）。`main` 的拒绝集与巡查记录逐字节一致（7 行，全部是
BCI 79/85/87 的嵌套数组/copy 家族，无 widening 行）；`Objects.*` 健康面逐字不变；平台枚举
（`java.lang.Thread$State`）仍拒——不在快照、不入表。
