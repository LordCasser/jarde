## Why

CF-08 的固定 `TestNotIndexedLoop` 含带效果双出口。隔离的 `EffectfulExits.pick` 在循环头越界时执行 `cost(7)` 并写结果，再跳共同后继；体内命中时跳过 `cost` 直接到同一后继。原 class 与固定 JADX 的完整 Java 8 源码重编验证运行一致，Jarde 因跨引用局部与未覆盖区域安全拒绝。先前双**纯**网关证书不能覆盖这条带效果路径，放宽其白名单会改变执行次数。

## What Changes

- 先证明现有 Region walker 能有界地将头部判断作为循环体首个 `if`，只在首次访问头部时放行、回边处停止；若不能闭合则保留现有拒绝，记录具体架构阻碍，不推进代码投影。
- 对单入口、单回边、两个独占出口及唯一共同后继建立窄证书：头部出口的调用和写入留在自己的 `if` 臂，体内纯转移成为另一条 `break`。必要时仅在 Region 增加无头测试的 endless 循环形态，现有 AST 仍输出 `while (true)`。
- 以实际 SSA 核对循环后局部合流、全部物理来源和正常/异常边；三方完整 Java 8 源码重编和 `java -Xverify:all` 比较效果次数。`TestNotIndexedLoop` 的外层分支与局部汇合仍单列。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：允许受证的单入口带效果双出口循环以 `while (true)` 和体内 `break` 保留执行路径。

## Impact

现有入口为 `crates/jarde-java/src/region.rs` 的 `loop_region`、`Frame.loop_body`、`loop_body_sequence` 与 `Region::Loop`，以及 `build.rs` 的 Loop 装配。只允许为这一形态补一个窄 Region 表示，不新增通用 CFG pass、AST 或依赖。固定证据见 `openspec/evidence/java-syntax-2026-09-27/cf08-endless-loops/effectful-baseline/`，JADX checkout 固定为 `2fb1b16386941660fda07e9017285aec40fcb37f`。
