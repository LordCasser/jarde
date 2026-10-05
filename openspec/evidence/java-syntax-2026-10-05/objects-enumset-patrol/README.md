# Objects 助手 + EnumSet 巡查（2026-10-05 root）

## 发现：EnumSet.of 枚举实参扩宽=平台扩宽族第 4 员（首个**类位**扩宽）

`EnumSet.of(Flag.A, Flag.C)` + retainAll + contains——**整方法响亮拒绝**（"parameter 0 … declared `java.lang.Enum` presents `OB$Flag`"）。`EnumSet.of(E, E...)` 擦除 `(Enum, Enum[])`，javac 发 `checkcast java/lang/Enum`——与 CharSequence/Comparable/Serializable（接口位）同机制、**父类位**新表行。jadx 直接解。已立 recover-enum-argument-widening（与三张姊妹表合并派发 → 四表）。

## 健康面（负结果）

- `Objects.equals/hashCode/toString(o,dflt)/deepEquals` 全恢复（`(Object)` cast 如实）；手写 null 守卫 hash 恢复；
- `Objects.requireNonNull(o, () -> "…"+nanoTime())` supplier lambda 形恢复（companion 内联）；
- `EnumSet.noneOf(Flag.class)`（Class 形参——无扩宽）在 main 正常。

## 处置

Enum 扩宽立项；其余不立项。行为 `true/false/0/0/dflt/true/hasA/no`。
