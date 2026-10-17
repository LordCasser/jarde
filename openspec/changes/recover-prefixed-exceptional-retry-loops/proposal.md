## Why

固定 JADX 账本 CF-11 的原始 `TestNestedLoops3` 在 Jarde 中仍为 fallback，完整源码的 16 次原样重编均因目标方法缺少返回语句失败。JADX 同输入的六次成功重编、12 次隔离运行共 84 case 与原类一致；另两次 javac8 外层类失败属于测试框架 major55 依赖边界。

## What Changes

- 恢复可完整证明的“外层无条件循环 + 带前缀的具名 catch 重试内环”：try 正常完成回到外层入口，catch 回到内层入口，条件出口离开外层循环。
- 复用已有 Loop、Try、Frame 和私有 retry/handler 组合入口，保持四个边界的独立身份、效果顺序、完整异常归属和物理来源。
- try 正常汇合点可直接为外层入口，或通过一个已证无效果、无额外入口的纯 goto 桥回到外层入口；两位置不强行合并，两个真实转移来源均保留。
- 增加改名及效果变体、所有权/表行/转移负例、真实停止测试和完整类原样重编运行对照。
- 前提是 root 已接受原始输入、JVM oracle、公开报告与实际首拒；候选必须先完成局部证明再认领块。不放宽 Builder 声明门，不把异常边加入 NormalFlowView，不新增公共 AST、solver 或 pass。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：增加可证明的带前缀异常重试嵌套循环恢复，保留非局部出口、预算和来源契约。

## Impact

主要涉及 `crates/jarde-java/src/region.rs` 的循环分类、具名 catch 与既有 retry seam，以及永久 Rust/Java 回归夹具。无新依赖或外部 API。finThrow/nestedFin 已交付，不重复实现；其他 CF-11 用例、finally 显式完成与 jsr 自然环债务另列。本片通过不增加 71/612 整体完成数。
