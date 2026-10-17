# 首轮实现与真实边界验收

root执行v3：456库、2公共正例、2公共停止、181reader、1指纹再生及5指纹检查，共647 passing appearances；六完整IR拒绝邻变体、五正向输入及真实early/late预算/取消/期限已实际通过。指纹只新增15文件，原2256条及分类不变；独立解析11class增加71Code/39handler/1goto，与完整reader统计一致。最终exact clean，target不存在。

v1引用参数编译失败、v2错误假设所有SSA effect带来源导致3个新增测试失败均保留；原453测试均通过。修正只匹配实际SSA契约：所有指令BCI完整核对，可抛点核物理身份，完整source-map另由公共测试核。新CLI已冻结，完整runtime矩阵和最终回归尚待验收，不以该本地轮次替代产品自身CI。
