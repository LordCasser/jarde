## Why

[CF-13 固定对照](../../evidence/java-syntax-2026-09-27/cf13-switch-exits/README.md)的 Java 8 循环内 switch 有两个 case 在 BCI 55 汇合后执行共享语句，另一个 case 在 BCI 49 直接跳至外层循环更新 BCI 58。Jarde 当前让 switch selector 的 canonical block 出现多个 Region owner，整段 `walk(I)I` 只能引用字节码，类级源码因缺少返回而不能重编。固定 JADX 在该组合中产生 `continue;` 后的不可达 `break;`，也不能重编；原 class 的 `-Xverify:all` 输出 `38` 是本任务的行为基准。

## What Changes

- 在现有 switch/loop Region 证明中，将已证的当前循环 `continue` 目标与 switch 局部正常汇合点分开；只在各 case 路径与所有权唯一时恢复 switch，跳向循环更新的 case 发出正确的 `continue`。
- 共享语句只出现在 switch 之后，循环更新仍由外层循环拥有一次；每个原指令的 Region/source-map 归属保持唯一，不在 `continue` 后插入 `break`。
- 对不满足证明的嵌套 switch、多入口、不唯一汇合或混合出口维持引用；以 CF-13 完整 Java 8 重编/运行和现有负例验收。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：恢复带 switch 局部汇合与外层循环 `continue` 的已证循环内整数 switch。

## Impact

限定于 `crates/jarde-java/src/region.rs` 的现有 CFG/Region、`Frame` loop target 与 switch case arm 走访，以及必要的 `build.rs` 语句/来源核验。不添加通用 CFG 重写、公共 pass 或对 JADX 的不可达 break 后处理。
