## Why

Java 发射器在输出额度检查或计费失败时丢弃真实错误，统一报告 `Budget(OutputBytes)`。若取消在最初轮询之后、输出 Probe 成功后的 observer 回调生效，实际 `Cancelled` 会被误报为输出预算耗尽；同类错误压缩也遮蔽检查期间的 deadline 停止。

## What Changes

- 输出提交路径保留真实取消与中断类别，只在实际输出额度不足时报告输出预算。
- 复用现有预算、停止映射和 buffer/map 丢弃逻辑；补齐共享计费辅助函数的 deadline 分类，保留真实输出预算的已写字节、限值及位置。
- 添加确定性的检查内取消回归，并运行既有精确额度、部分写入、replay 与报告停止测试。

前置是既有 `Budget`、ledger observer、`stop::charge` 和原子发射协议。当前已正式立项并准备私有源码与回归候选，尚未应用共享源码或运行回归；应用与测试继续由 coding agent 的串行窗口管理。

非目标：不修改全局预算许可与首次停止语义、不新增预算维度、报告平面、计时框架或依赖，不扩展条件值或普通 Try/List 的语法准入。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：输出检查和计费期间的真实取消、中断与输出额度停止保持各自分类；未提交的文本与来源不泄漏。

## Impact

主要影响 `crates/jarde-java/src/emit.rs` 的单一提交入口、`stop.rs` 的必要错误分类及其测试；必要时补现有报告入口回归。公开 API、AST、来源身份、正常文本与 replay 规则保持。独立于进行中的 `recover-forward-conditional-stack-carry`，不以本项证据代替其矩阵验收。
