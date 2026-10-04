## Why

[TWR 巡查](../../evidence/java-syntax-2026-10-04/twr-javac8-codegen-patrol/README.md)实测（同一 `TR.java` 双腿对照）：jarde 对 **javac 9+** `--release 8` 的 TWR 完整恢复（`one()` 22 指令/2 异常表行 → `try (TR local1 = new TR(arg0)) { … }`，0 引注），但对**真 javac 8**（Corretto 1.8.0_432）的同一语句**整方法拒绝**（`one()` 48 指令/5 行 → `jre_region_exception_edge` ×6 + `jre_region_uncovered_blocks` ×4，整类 6 引注）。`try (Resource r = …)` 是 Java 8 目标层级（jarde 唯一输出层级）的**极常见形**，且真 javac 8 产物与 `--release 8` 交叉编译产物**都是 major 52 的 Java 8 class**——真实世界 JDK-8 编译产物全部命中该缺口。

**几何根因（巡查三之三节，指令级实录）**：两腿**主路径几何本身不同**——javac 8 主路径为 `aconst_null; astore_2`（primary-exception 槽）+ **守卫式 close**（`ifnull` ×2：资源空则跳过、有 primary 异常则走无抑制 close）；javac 9+ 主路径 close **无条件**。且 javac 8 的 primary 异常经"存槽 → `athrow` 重抛 → 被 `any` 行再捕获"进入抑制链，其 `any` 行（`one()` 2 条、`two()` 5 条）指向 javac 9+ 表里**不存在**的抑制链 handler 块。**行变体路线已被判别实验排除**——无论实现走哪条路都必须新增对这些几何事实的证明。这是**机制层扩展**（巡查第三节：guard 证明 CFG/异常表驱动，3700-3860 段 opcode 字面量 = 0），不是 idiom 拼写补丁。

**本片是队列中最后一个已定根因、尚无 change 目录的大颗粒项**（DT-03 残留在飞、EM-15/DT-26/合并头片均已 spec 就绪）。

## What Changes

- 让 TWR 的 region/guard 证明接受真 javac 8 的关闭序列几何：**primary-exception 槽引导、守卫式 close（`ifnull` 双守卫）、重抛-再捕获回路、抑制链 handler 块与其 `any` 行**。
- **MVP：单资源先行**（`one()` 形）。双资源（`two()`，5× `any`）按巡查三之二节的实测边界独立扩验（本片 Non-Goal，可后续片）。
- 实现路线二选一（**task 1.1 插桩定夺，不得预设**）：(A) 扩展既有 `Shape::Resources`（guard.rs `fn guarded` 证明链）的几何判据以容纳 javac 8 的抑制链块与守卫式 close；或 (B) 为 javac 8 形新增 `Shape` 变体家族（按"每资源一条 init-区 any + 一条 close-区 any，目标为抑制链头"的结构事实）。判别标准：现有 `Resources` 判据（`fn guarded` 到 `Shape::Resources` 构造的六项 CFG 机制）能否在**不稀释**（不把 javac 9+ 形的判据放宽到不可证伪）的前提下容纳——巡查三之三节已把两分支的分界写清。
- 恢复后呈现与 javac 9+ 腿同构：`try (TR local1 = new TR(arg0)) { … }`。

## Impact

- **代码**：`crates/jarde-java/src/guard.rs`（`fn guarded` 证明链与 `Shape::Resources` 构造）为主；可能触及 `crates/jarde-java/src/region.rs`（若 `jre_region_exception_edge`/`uncovered_blocks` 先于 guard 拒绝，region 层须先放行抑制链块——**这正是 task 1.1 插桩要定夺的第一道门**）。**不触碰** `emit.rs`（呈现形不变）、`accessor.rs`、`lambda.rs`。
- **测试**：冻结 `TR` 族（巡查 fixture 已有真 javac 8 与 javac 23 双腿）+ javac 9+ 零回退锚（现恢复的 13 个 finally shape 变体与既有 `Shape::Resources` 测试逐字节不变）；负例 = 削弱几何的合成探针（少一条 `any` 行、守卫顺序错乱、抑制链断头）。
- **账本**：CF-17（control-flow.md）+ `summary.md` 的版本耦合行更新；`dual-javac-sweep` 的 P08 行从"待修"改为指向本片。
- **验收锚纪律**：主锚 = `TR.one()` 真javac8 腿（0 引注、整类可编译、行为一致）；对照正例 = `TwrAudit`（巡查第二节的 138 指令/1 `aconst_null` 形，同机制不同资源数）。

## Non-Goals

- **不**做双资源（`two()`）——独立扩验（巡查三之二：MVP 边界的实测依据）。
- **不**改 javac 9+ 形的任何判据（零回退锚钉死；判据稀释即停手条件）。
- **不**处理 `try…catch…finally` 与 TWR 的叠加形（`inner_finally`/`trailing_finally` 域，已有各自覆盖）。
- **不**引入 `java_release`/`major_version` 判据（两种产物 major 均 52；认的是几何事实不是编译器版本）。
