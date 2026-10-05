## Why

[sync-return-timing 巡查](../../evidence/java-syntax-2026-10-05/sync-return-timing-patrol/README.md)：嵌套 `synchronized` 的内层 `return expr;` 被渲染为内层空块 + return 外移——求值越过内层 `monitorexit`。判别探针 `NL`（`Thread.holdsLock(NL.class)` 在 `toString` 内）剥离编译 exit 0、运行 `nN` vs 原 `nY`：**可编译但行为不同**（第一不变量违反）。javap 证实原字节码求值（BCI 11–27）先于内层 `monitorexit`（BCI 31）。jadx 以锁内 temp 赋值+锁外 return 保真——有解，呈现层归因。

## Root 预审计（2026-10-06，读码）

- 字节码事实：`NL.probe` 求值 BCI 11–27 在内层 `monitorexit` BCI 31 之前（javap 已档）；
- 归属错置：guard.rs `Shape::Monitor { returns }` 的文档要求 return 写在**该层** braces 内（guard.rs:320–327），但 `InnerMonitor`（guard.rs:160）**没有自己的 returns 字段**——嵌套形下 return 只属于外层 shape；build.rs synchronized return 分支（~14525 注释/`body.push(statement)`）因此把 return 语句追加到**外层 body 的嵌套块之后**，内层块渲染为空；
- 最小修法二选一（实现插桩定夺）：给 `InnerMonitor` 增加自身 returns（return 语句入内层 body）；或外层渲染时为内层体生成 temp 赋值（求值留内层、return 留外层——jadx 形）。两者都在既有 Monitor/InnerMonitor 证明结构内，无需新机制。

## What Changes

- synchronized 形状的**求值次序不变量**：被 return 消费的表达式，其呈现位置必须使求值发生在配对 `monitorexit` 之前。两种等价呈现：return 语句留在该层 synchronized 体内；或引入合成局部在体内赋值、return 移到体外（jadx 先例）。单层现状已满足（`retInside` 原样），本片只修嵌套内层 return 归属错置。
- 落点：build.rs synchronized return 形（嵌套 pair 的 `returns` 归属）；无新机制，不放宽任何判据。

## 硬不变量

1. 单层 synchronized（return 内/局部跨锁）与既有 monitor 巡查渲染**逐字节不变**；
2. 不得引入无证据的求值重排：temp 形仅在原字节码求值完整位于配对 exit 前时使用；
3. 诊断文本零变化（本例本就全恢复，无引注）。

## 验收

- `NL` fixture 剥离编译 exit 0、运行 **`nY`**（与原一致）；
- `SR` fixture 往返 `42/42/n5`+`caught:neg` 一致；`voidBody` 保持现状安全拒（归 soundness 片）；
- 既有 synchronized corpus（#75 monitor 域）渲染零回退；全门禁。
