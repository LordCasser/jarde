# 类型片预算草稿 root 审查

root全文读private-budget-luna-v1的README及internal-budget-test.patch。它用真实MethodIr而非手造SSA，测试目标正确，但引入4种test-only实体、thread-local observer、生产收费路径cfg(test)分支与另一套大型真实IR组装助手，只为了测2个固定输入的收费点。本项目要求如非必要勿增实体；当前pop片已用临时trace保存真实prefix、去掉trace后以冻结真实类和独立恢复Budget断言准确Stop，未添加永久观察机制。

因此不应用该草稿，不把私有AST或静态描述算实测。类型片重新应用后沿同一方法：临时诊断实际char写@29与null-leading@1的prefix/charge amount，保存原始raw/diff，然后移除诊断，复用已有永久测试的same-analysis trusted subject组装，精确预算Stop/0输出；公开取消只声称真实IR之后原子拒绝，不冒称token恰在内部gate设置。若确有无法外部观察的不变量，再单独论证必要测试接口。草稿保留不覆盖。
