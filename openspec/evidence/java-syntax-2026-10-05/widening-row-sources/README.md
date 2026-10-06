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

## change `recover-temporal-argument-widening` 的表行（2026-10-06）

同一 rt.jar、同一命令的续录，转录行在 [javap-headers.txt](javap-headers.txt) 末尾十二行。

| 表 | 行（呈现类型 → 目标） | 依据（转录行） |
| --- | --- | --- |
| Temporal 族（**十四行**：七个 java.time 具名类型 × {`java.time.temporal.Temporal`, `java.time.temporal.TemporalAccessor`}） | `java.time.LocalDateTime` / `java.time.LocalDate` / `java.time.LocalTime`（MVP 三型）与 `java.time.Instant` / `java.time.ZonedDateTime` / `java.time.OffsetDateTime` / `java.time.OffsetTime`（提案扩展四型） → `java.time.temporal.Temporal`；同七名 → `java.time.temporal.TemporalAccessor` | 七个类的 header 各自**逐字**声明 `implements java.time.temporal.Temporal, …`（`Temporal` 行）；`TemporalAccessor` 行经 `java.time.temporal.Temporal` 自身 header 的 `extends java.time.temporal.TemporalAccessor` 到达。链（七型同形）：`java.time.LocalDateTime --implements--> java.time.temporal.Temporal --extends--> java.time.temporal.TemporalAccessor`。javadoc 式 implemented-interface 列表读法，与 java.util 表 `Properties → java.util.Map`（经 `Hashtable`）同规。**七个类型都直接声明 `Temporal`，故扩展四型的 `TemporalAccessor` 对同样有 header 依据，无一对需要靠记忆补**。 |
| 并发（**两行**，批 2，[CompletableFuture 巡查](../../../evidence/java-syntax-2026-10-05/completable-future-patrol/README.md)） | `java.util.concurrent.CompletableFuture` → `java.util.concurrent.CompletionStage`、`java.util.concurrent.Future` | header 逐字 `public class java.util.concurrent.CompletableFuture<T> implements java.util.concurrent.Future<T>, java.util.concurrent.CompletionStage<T> {`——两个目标接口在同一行声明。 |

自检：[`results/selfcheck-javap-rows.sh`](../../../changes/recover-temporal-argument-widening/results/selfcheck-javap-rows.sh)
对十二个类型重跑 `javap`、按本文件索引列（短名补到 33 列、长名单空格）重排后与该文件**逐字节 diff 相同**。

