# 任务 1 证据：javap 转录（change `recover-temporal-argument-widening`）

## 转录对象与命令

- JDK：Corretto 1.8.0_432，`jre/lib/rt.jar` shasum -a 256 =
  `b27515a608ee447566b688e2bbb2257b1f0d8eceb96c307b87eb28d90a6630f4`（与 widening-row-sources
  README 记录一致，本次实测复核）。
- 命令：`javap -classpath <rt.jar> <type>`，取第 2 行（`javap` 的声明行）。
- 原始输出（每种类型一行命令 + 一行输出）见 [javap-raw-lines.txt](javap-raw-lines.txt)；
  转录进 `openspec/evidence/java-syntax-2026-10-05/widening-row-sources/javap-headers.txt` 末尾十二行。
- 自检：`results/selfcheck-javap-rows.sh` 重跑十二个类型、按该文件索引列重排后 `diff` 逐字节相同，
  输出见 [selfcheck-javap-rows.out](selfcheck-javap-rows.out)（`SELF-CHECK OK`，exit 0）。

## 行集决定（按 pin 的行集协议）

七个 java.time 类型**各自 header 逐字声明** `java.time.temporal.Temporal`；`TemporalAccessor`
没有一个是"平台级联里的类型经别处的 header 到达"以外的推断：七型都经 `java.time.temporal.Temporal`
自身 header（`public interface java.time.temporal.Temporal extends java.time.temporal.TemporalAccessor {`）
到达该接口，链逐型相同：

```text
java.time.<T> --implements--> java.time.temporal.Temporal --extends--> java.time.temporal.TemporalAccessor
```

因此 **14 行**（MVP 3 型 + 提案扩展 4 型，各 2 目）全部有 header 依据，扩展四型**无一对**因缺依据被拒；
本片不写任何 header 未支持的推测行。

并发批 2 两行由 `CompletableFuture` 自己的 header 声明（两个目标接口在同一行）：

```text
public class java.util.concurrent.CompletableFuture<T> implements java.util.concurrent.Future<T>, java.util.concurrent.CompletionStage<T> {
```

## 落表行（实现与测试逐行对应）

```text
java.time.LocalDateTime -> java.time.temporal.Temporal, java.time.temporal.TemporalAccessor
java.time.LocalDate     -> java.time.temporal.Temporal, java.time.temporal.TemporalAccessor
java.time.LocalTime     -> java.time.temporal.Temporal, java.time.temporal.TemporalAccessor
java.time.Instant       -> java.time.temporal.Temporal, java.time.temporal.TemporalAccessor
java.time.ZonedDateTime -> java.time.temporal.Temporal, java.time.temporal.TemporalAccessor
java.time.OffsetDateTime-> java.time.temporal.Temporal, java.time.temporal.TemporalAccessor
java.time.OffsetTime    -> java.time.temporal.Temporal, java.time.temporal.TemporalAccessor
java.util.concurrent.CompletableFuture -> java.util.concurrent.CompletionStage java.util.concurrent.Future
```

## 旁证（不进表，如实记录）

`java.time.temporal.Temporal` 的实现者不止这七型（`java.time.chrono.*` 的
`ChronoLocalDateTime`/`ChronoZonedDateTime` 等接口族、`java.time.temporal.TemporalAdjuster` 的实现
型等）。本片按 spec 钉的**封闭十四行**落表，其余保持拒绝（保守省略：少放行、不误放行）；如需扩行，
是另一个行集决定——与 CharSequence 片对 `javax.swing.text.Segment` 的处置同规。
