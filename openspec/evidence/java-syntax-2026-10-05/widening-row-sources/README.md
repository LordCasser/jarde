# 平台扩宽表行来源（`javap` 转录，2026-10-06）

三个姊妹 change（`recover-charsequence-argument-widening`、`recover-comparable-argument-widening`、
`recover-enum-argument-widening`）的表行**逐行**核对依据。转录对象是**release 8 的真实 class 文件**：

- JDK：Corretto 1.8.0_432，`jre/lib/rt.jar` sha256
  `b27515a608ee447566b688e2bbb2257b1f0d8eceb96c307b87eb28d90a6630f4`；
- 命令：`javap -classpath <rt.jar> <type>`；输出见 [javap-headers.txt](javap-headers.txt)。

| 表 | 行（呈现类型 → 目标） | 依据（转录行） |
| --- | --- | --- |
| CharSequence（四行） | `java.lang.String` / `java.lang.StringBuffer` / `java.lang.StringBuilder` / `java.nio.CharBuffer` → `java.lang.CharSequence` | 各自 header 逐字 `implements … java.lang.CharSequence`（`StringBuffer`/`StringBuilder` 经 `AbstractStringBuilder` 仍自声明该接口） |
| Comparable（九行） | `java.lang.String` + 八装箱（`Byte`/`Short`/`Integer`/`Long`/`Float`/`Double`/`Character`/`Boolean`） → `java.lang.Comparable` | 九型 header 各自 `implements java.lang.Comparable<…>` |
| Serializable（九行） | 同上九名 → `java.io.Serializable` | `String`/`Character`/`Boolean` 自声明；六个 `Number` 子类经 `java.lang.Number`（`implements java.io.Serializable`）到达——javadoc 的 implemented-interface 列表读法，与 java.util 表 `Properties → java.util.Map`（经 `Hashtable`）同规 |
| Enum（四行，枚举族集合类型） | `java.util.EnumSet` → `java.util.AbstractSet` / `java.util.Set` / `java.util.Collection` / `java.lang.Iterable` | header：`extends java.util.AbstractSet<E> implements java.lang.Cloneable, java.io.Serializable`；implemented-interface 列表达 `Set`/`Collection`/`Iterable`（`AbstractSet → AbstractCollection → Collection → Iterable`） |

## 实测到的两处“行集比 spec 措辞更宽/更窄”的事实（如实记录，未外推）

1. **`javax.swing.text.Segment` 是 release 8 的 CharSequence 实现者**（header：
   `implements java.lang.Cloneable, java.text.CharacterIterator, java.lang.CharSequence`）。change 的
   spec 记它为“9+、不入表”，实际 release 8 的 javadoc 实现者列表含它。实现按 spec 钉的**封闭四行**
   落表（`Segment` 仍拒），属**保守**省略（少放行、不误放行）；如需扩行，是另一个行集决定。
2. **`java.lang.Enum` 自身实现 `Comparable` 与 `Serializable`**（header：
   `implements java.lang.Comparable<E>, java.io.Serializable`）。change 的两个 java.lang 表按 spec 钉的
   九行（String + 八装箱）落表，不含 `Enum`（同样保守）。

两处都不影响本次锚（`String`/八装箱/`EnumSet` 均在表内），列在此供行集审计。
