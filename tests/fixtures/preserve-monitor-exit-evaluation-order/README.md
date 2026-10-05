# preserve-monitor-exit-evaluation-order — frozen fixtures

`NL.java` 与 `SR.java` 是 sync-return-timing 巡查的冻结原件副本
（`openspec/evidence/java-syntax-2026-10-05/sync-return-timing-patrol/fixture/`）；
两份 `.java` 的 SHA256：

| source | SHA-256 |
| --- | --- |
| `NL.java` | `4ecd95a5f3a49f648bb7737196708dd27d68c0aefa85c5fcf1970dad90a336cf` |
| `SR.java` | `262cd8403df6b59ebdc0046d3277033c6f641d4432b0ce23de6c15c8c8010f88` |

## 双腿（同一源码，两个编译器）

- `v8/`：**javac 23.0.1**，`javac --release 8 -d v8 NL.java SR.java`（默认调试信息，与巡查
  `nl.jar`/`sr.jar` 内的字节逐字节相同：`NL.class` `22d17174…`、`NL$Box.class` `cac0ded4…`、
  `SR.class` `fd376fac…`）。jarde 侧渲染读的是这组字节。
- `v8-javac8/`：**真 javac 8** — Corretto 1.8.0_432
  （`/Users/lordcasser/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home/bin/javac`，
  `javac -version` → `javac 1.8.0_432`，**无 `--release`**，默认目标即 Java 8），
  `javac -d v8-javac8 NL.java SR.java`。
  两腿的 `probe`/`nestedLock` 布局一致（常量池次序不同，BCI 相同），这正是"双腿都修、双腿都读"
  的意义：单腿无法暴露版本耦合。

| class file | bytes | SHA-256 |
| --- | ---: | --- |
| `v8/NL.class` | 1,034 | `22d1717404dd22291ef5f2e1a53c217956e01e94aba639d19433ff63aab0d98b` |
| `v8/NL$Box.class` | 448 | `cac0ded470341f7edf86f7c5be128f54f67a3c3996f332f5d9673d509298caa6` |
| `v8/SR.class` | 1,554 | `fd376fac78c3f2539fc3eaf32deeba926d59db4fc473125ee8c434c083353dc7` |
| `v8-javac8/NL.class` | 932 | `5e3c30706923f81469ad31dc8af04fec5e778041d1c3fd2ca7882abf6ed29bd5` |
| `v8-javac8/NL$Box.class` | 375 | `8bbcd8aa7f206da498b6b2349c7d7ee6898566b6f43c280814ce026b33c75ead` |
| `v8-javac8/SR.class` | 1,399 | `b60da829df682eec128ee6b0d106f4260ed6a6c3e7e720162074a4ed92c7b280` |

## 判别事实（javap 实测，两腿 BCI 相同）

`NL.probe`：`synchronized (LOCK) { synchronized (NL.class) { return "n" + o; } }`

```text
 5: monitorenter            (outer)
10: monitorenter            (inner)
11: new StringBuilder       ┐
…                           │ "n" + o 的求值：BCI 11–27
27: StringBuilder.toString  ┘
30: aload_2
31: monitorexit             (inner)  ← 求值必须发生在这之前
32: aload_1
33: monitorexit             (outer)
34: areturn
```

`NL$Box.toString()` 返回 `Thread.holdsLock(NL.class) ? "Y" : "N"`，即"求值时是否持内层锁"：
原 class 运行 `nY`；把内层体写成空块、`return` 外移的呈现运行 `nN`（该负面读数记录在
`openspec/changes/preserve-monitor-exit-evaluation-order/verification.md`）。

`SR`：`retInside`（return 在锁内）、`localAcross`（局部在锁内写、锁外读）是健康单层对照，
`voidBody`（void + throw）由 soundness 片安全拒（`jarde_refused_body();` 保留标记）。
`nestedLock` 与 `NL.probe` 同形，是本片的第二个内层 return 锚。

## 引用它们的测试

`tests/preserve_monitor_exit_evaluation_order.rs`：

- 非 ignored：双腿各呈现一次，逐字钉 `NL.probe`（return 在内层 braces 内）、`SR.retInside` /
  `localAcross` / `voidBody` / `nestedLock` 文本，并钉 `probe` 的来源 BCI 锚（5/10/11/31/33/34）；
- ignored（`cargo test -- --ignored`，需 `javac`/`java`）：剥离注释行后用 `javac --release 8`
  编译呈现文本、`java -Xverify:all` 运行得 `nY`，并与原 class 的同一次运行逐字比较；
  `SR` 的剥离文本必须在 `jarde_refused_body` 上编译失败（fail-closed）。
