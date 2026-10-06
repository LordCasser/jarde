# 任务 2 证据：行集落地 + 双 javac 腿锚 + 零回退（change `recover-temporal-argument-widening`）

## 实现（file: crates/jarde-java/src/build.rs）

`platform_interface_argument_widens` 内新增两张 `const` 行表并在 `.any()` 链里入列：

- `TEMPORAL_FAMILY`：**14 行**——`java.time.{LocalDateTime,LocalDate,LocalTime,Instant,ZonedDateTime,OffsetDateTime,OffsetTime}`
  各 → `java.time.temporal.Temporal` 与 `java.time.temporal.TemporalAccessor`；
- `COMPLETABLE_FUTURE`：**2 行**——`java.util.concurrent.CompletableFuture` → `java.util.concurrent.CompletionStage`、`java.util.concurrent.Future`。

命中仍走既有 `cast_argument` 呈现（零第三种呈现），release-8 门与阵列位谓词（`platform_array_argument_widens`）
逐字未动。**diff 形态如实披露**：`git show --stat a53814a1` = 222 插入 / 3 删除——3 行删除是
`.any()` 链的数组字面量由 rustfmt 重排（`[CHAR_SEQUENCE, COMPARABLE, SERIALIZABLE, ENUM_FAMILY]` →
6 元素数组），即"membership in the `.any()` chain"本体；既有四张表的行、门与诊断文本逐字节未改，
文档注释是**追加**一段（既有段落逐字保留），见 `git diff` 复核。

## 单元测试（build.rs `mod tests`）

`the_temporal_family_and_completable_future_rows_reach_exactly_their_pairs`：16 条正例（每条新行一条，
release 8）+ 22 条负例（release 7/9 门、`TemporalAdjuster`/`java.time.chrono.*`/`ScheduledFuture`/
`RunnableFuture`/`FutureTask` 等表外实现者、下行与接口→接口方向、`Object`、原始型、数组位、泛型形、
用户类型）。实测（`cargo test -p jarde-java --lib --locked temporal_family`）：1 passed / 0 failed（286 过滤）。

## 冻结锚（`tests/fixtures/recover-temporal-argument-widening/`，两条 javac 腿）

源：巡查 fixture `JT.java`/`DT.java`/`CF.java` **逐字节复制** + 本片负例 `TWX.java`；腿：
`javac --release 8 -Xlint:-options -d v8 *.java`（javac 23.0.1）与真 javac 8（Corretto 1.8.0_432）`-d v8-javac8`。

**来源核验（实测）**：`v8` 腿的 `JT.class`/`DT.class`/`CF.class` 与巡查 jar（`jt.jar`/`dt.jar`/`cf.jar`）
里的同名 class `cmp` **逐字节相同**（巡查 jar 即同一条 `--release 8` 命令的产物）；`v8-javac8` 腿为同源
另一条编译产物（字节不同，行为一致）。sha256 见 fixture README。

## 锚渲染（两条腿逐字一致；`tests/recover_temporal_argument_widening.rs` 钉住整段文本）

| 锚 | base（本片行落地前） | patched（本片） |
| --- | --- | --- |
| `JT.fmt` | BCI 19/36 两条 `TemporalAccessor` 拒绝 | `local0.format((java.time.temporal.TemporalAccessor) local1)`（两处）+ `parse((java.lang.CharSequence) …, local0)`，0 引注 |
| `JT.spans` | BCI 27/46 两条 `Temporal` 拒绝 | `Duration.between((java.time.temporal.Temporal) …, …)`、`ChronoUnit.MONTHS.between((java.time.temporal.Temporal) local0, (java.time.temporal.Temporal) local1)`，0 引注 |
| `DT.direct` | BCI 19 一条 `TemporalAccessor` 拒绝 | `local0.format((java.time.temporal.TemporalAccessor) local1)`，0 引注 |
| `DT.parse` | 已恢复（CharSequence 表） | 逐字节不变 |
| `CF.combined` | BCI 24 一条 `CompletionStage` 拒绝 | `local0.thenCombine((java.util.concurrent.CompletionStage) local1, …)`，0 引注 |
| `CF.chain`/`CF.recover`/`JT.basic` | 已恢复 | 逐字节不变（测试钉住整段文本） |

## 负例（`TWX`，两条腿逐字一致）

```text
// the parameter 0 of the invocation at BCI 10 is declared `java.time.temporal.TemporalAccessor` presents `java.time.MonthDay` but the invocation requires `java.time.temporal.TemporalAccessor` and this layer has no safe reference conversion evidence
// the parameter 0 of the invocation at BCI 15 is declared `java.time.temporal.Temporal` presents `java.time.Year` but the invocation requires `java.time.temporal.Temporal` and this layer has no safe reference conversion evidence
```

`MonthDay`/`Year` 的 release-8 header 都**逐字声明**目标接口（见 `javap-raw-lines.txt` 同批核对），
但不在钉的十四行内，故保持拒绝（保守省略）。

## 行为验收（剥离 → 编译 → `-Xverify:all` 运行）

命令：`cargo test --test recover_temporal_argument_widening --locked -- --ignored --nocapture`
（每条腿：剥离 `//` 行 → 装机 `javac --release 8` 与真 javac 8 各编译一次 → `java -Xverify:all` 运行 →
与 fixture 自身 class 的运行输出比较）。实测：**1 passed / 0 failed**，真 javac 8 腿在场
（`/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home/bin/javac`）。

原 class 与剥离文本的运行输出（逐字节相同，已钉在 `REPLAYS.answer`）：

| 类 | 输出 |
| --- | --- |
| `JT` | `2026-10-06/2026-11-01/09:30` / `2026/01/02 03:04 -> 3:4` / `287` |
| `DT` | `03:04/03:04` |
| `CF` | `DATA!` / `fallback:java.lang.IllegalStateException: x` / `6` |

（手工复核同样通过：装机 `javac --release 8` exit 0、真 javac 8 exit 0、两者 `-Xverify:all`
输出与 fixture class 一致——整条命令与输出见 `results/cli-roundtrip.sh` 与 `results/02-acceptance-roundtrip.out`，
渲染前断言了 jarde 自述头 `// jarde: presentation of` 以免把错误文档当成"零引注渲染"。）

## 零回退：base vs patched 语料对拍

`results/corpus-sweep.sh`（**先自测已知正例/负例**再计数）：正例 `JT` 引注 4 → 0、负例 `TWX` 引注 2 → 2，
自测通过后才计数。语料：`openspec/evidence` 下 **1987** 个散装 class（与先前扩宽片同一语料）**加**
全部 jar 内的 class 条目。

实测：散装候选 **0**；jar 内候选 **3**（恰为本片三个锚 `JT`/`DT`/`CF`），全部移动、引注 **6 → 0**、
**新增引注 0 条**；其余语料逐字节不动。完整输出 `results/corpus-sweep.out`，三个锚的完整 diff
`results/corpus-anchor-diffs.txt`（新增行仅恢复语句与 fold 注记，删除行仅拒绝句及其 `@bytecode` 行）。
