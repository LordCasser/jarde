# `recover-temporal-argument-widening` 的冻结 fixture

巡查锚的源（`JT.java`/`DT.java` 取自
[java-time-temporal-patrol](../../../openspec/evidence/java-syntax-2026-10-05/java-time-temporal-patrol/README.md)
的 fixture，`CF.java` 取自
[completable-future-patrol](../../../openspec/evidence/java-syntax-2026-10-05/completable-future-patrol/README.md)
的 fixture，逐字节复制）与本片负例 `TWX.java`，以及两条 javac 腿的 class 文件：

```sh
javac --release 8 -Xlint:-options -d v8 *.java                       # javac 23.0.1
/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home/bin/javac -d v8-javac8 *.java
```

`v8` 腿的 `JT`/`DT`/`CF` class 与两条巡查各自 jar 里的同名 class **逐字节相同**（`cmp`，本片实测）：
巡查的 jar 就是同一条 `javac --release 8` 命令的产物，故锚的字节与巡查记录同源；真 javac 8 腿是同一份源的
另一条编译产物（字节不同：真 javac 8 自己写常量池与 `StackMapTable`）。

## 锚（两条腿渲染逐字一致）

- `JT.fmt`：`DateTimeFormatter.format(LocalDateTime)` 的**两个位点**（`f.format(dt)` 各写一次）
  由拒绝转为 `(java.time.temporal.TemporalAccessor)`；`LocalDateTime.parse(f.format(dt), f)` 的
  `CharSequence` 位由先前的实现者表作答（本片不动）。
- `JT.spans`：`Duration.between`（`LocalDateTime`）与 `ChronoUnit.MONTHS.between`（`LocalDate`）
  写 `(java.time.temporal.Temporal)`；`Period.between(LocalDate, LocalDate)` 同型、不引入 cast。
- `DT.direct`：直接实参判别位（非数组读来源）的 `TemporalAccessor` 位；`DT.parse` 为第二对照。
- `CF.combined`：`thenCombine(CompletionStage, BiFunction)` 首参写
  `(java.util.concurrent.CompletionStage)`（批 2 两行的第一行；`Future` 行由单元测试钉住）。
- 零回退对照：`JT.basic`（工厂+链）、`DT.parse`、`CF.chain`/`CF.recover`/`CF.main` 与巡查记录逐字节一致。

## 负例（`TWX`，仍拒；两条腿逐字一致）

- `month`：`java.time.format.DateTimeFormatter.format(java.time.MonthDay)` —— `MonthDay` 的 release-8
  header 逐字声明 `java.time.temporal.TemporalAccessor`，但**不在本片钉的封闭十四行**，故保持拒绝
  （BCI 10，`presents \`java.time.MonthDay\``）；
- `year`：`java.time.temporal.ChronoUnit.DAYS.between(java.time.Year, java.time.Year)` —— `Year` 的
  header 逐字声明 `java.time.temporal.Temporal`，同样不在十四行（BCI 15，`presents \`java.time.Year\``）。

两行均为**保守**拒绝（少放行，绝不误放行）——与 CharSequence 片对 `javax.swing.text.Segment` 的处置同规。

## class 文件 SHA-256

| 文件 | sha256 |
| --- | --- |
| `v8/JT.class` | `fed6ed96911001713b5993851405ba17714a215e79a5747246c6d54c501c7878` |
| `v8/DT.class` | `ee23ead0cd83b97c0cabb8a525fac2156b470beb9536bce0af51c9962eae1ae4` |
| `v8/CF.class` | `2e6089e80fe18f8390c235cf145dba05ba20160433214d026f98bf7c9baa6bac` |
| `v8/TWX.class` | `2d5865545c8273b9c2579239df1d1f1c075386e562baaf52bf5cb00cb5a096f4` |
| `v8-javac8/JT.class` | `a37281d2cfde634b5fe8acafed335038eef4a946fe3369e9814d3d4a70df1f49` |
| `v8-javac8/DT.class` | `bd348c3aa82de2566f5481fa5d2a660a2e34de1d2d1f87d6762c8e4d7cb887ab` |
| `v8-javac8/CF.class` | `f1b66274032286e3b951864ee35df780876f683456562e033fcdd2857acfe06b` |
| `v8-javac8/TWX.class` | `fc5f082f6a614c50439d6eca803f9a0397e2474b11b82d7427ac11617070302b` |

## 行为（ignored replay：剥离→编译→`-Xverify:all` 运行，与原 class 逐字节一致）

| 类 | 原 class 与剥离文本的运行输出 |
| --- | --- |
| `JT` | `2026-10-06/2026-11-01/09:30` / `2026/01/02 03:04 -> 3:4` / `287` |
| `DT` | `03:04/03:04` |
| `CF` | `DATA!` / `fallback:java.lang.IllegalStateException: x` / `6` |

`TWX` 是文本锚（整类仍有拒绝，故不参与 replay），其渲染记录见
[change 的 results](../../../openspec/changes/recover-temporal-argument-widening/results/)。
