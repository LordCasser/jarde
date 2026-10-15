## 1. Reproduce and repair output admission

- [x] 1.1 在旧 `put` 上实际运行第二个 ledger Checkpoint 内取消的确定性回归，保存真实 `Budget(OutputBytes)` 错误分类；补已有正文后取消的 buffer 原子丢弃断言，并以真实零 elapsed Budget 复现共享 charge 将 deadline 误报为 IR/输出额度；不以预取消或 sleep 替代检查内取消时点。
- [x] 1.2 补齐 `stop::charge` 的真实 ElapsedMillis 到既有 Interrupted 映射，并复用它修复单一输出许可入口；实际跑新增取消/deadline与既有计费、精确额度、部分写入、UTF-8、source-map replay 回归，核真实输出预算的 written/limit/at、正常文本和 OutputBytes 费用保持。

## 2. Integration and delivery

- [x] 2.1 串行运行受影响 Java 库与报告/来源停止回归、fmt、当前 CI Clippy、两个 OpenSpec 版本的 strict 以及必要 workspace 检查；记录命令、原始输出与前后源码 SHA，证明停止未交付正文、声明或部分来源。
- [x] 2.2 独立 review 接受停止分类、预算许可和原子丢弃证据后，以独立切片提交并接受该产品自身 CI；校验最终 HEAD 与本项任务记录，不借其它切片的测试声明完成。

2026-10-15：根旧生产对照实际0/3失败、修后focused 440/0/0，两个strict与精确clean通过，见[verification-root.md](verification-root.md)。1.1/1.2完成；2.1必要workspace与2.2提交自身CI待验，不以局部门禁替代。

2026-10-15 最终联合验收：以上早期“待验”描述保留为历史。产品d3e2b2457338b61660b842f5705b225bba08a73a自身CI38673548915成功，workspace-v3两seed各3587/0/98，focused-v3为462/0/2；独立审计接受，4/4关闭。见verification-root.md新增联合验收节。
