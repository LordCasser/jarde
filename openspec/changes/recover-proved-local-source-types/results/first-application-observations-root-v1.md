# 类型片首次真实应用：尚未验收

root 应用已审查v3，constructor owner spelling复用现有spell_reference，安装v4 trusted-subject测试并实跑守卫。focused-root-v1/v2两轮均真实2 passed/1 failed；无guard stop，峰值204,220,969 bytes。NoDefault全物理来源与原子预算/取消通过，char正文已恢复char c/append/switch且无quote，但完整来源测试实际缺[82,92,105]，它们都是append(C)后pop。

第二轮仅将全物理断言改成一次列出全部缺口，未削弱要求。两轮失败raw、实际typed生产diff applied-product-root-v2.patch和永久测试applied-permanent-tests-root-v2.rs均保留。不能据2个通过测试标记2.1/2.2/2.3完成，也尚未完整类重编运行或冻结CLI。

root 已读DiscardedEvaluations::discards/call_result_is_discarded/Builder::call_statement/quoted_bcis；同block相邻、相同真实SSA值、唯一pop消费且没有local write已由既有计划证明。拒绝call的quote保留pop，而成功call_statement仅direct(call)，遗漏被accounted跳过的pop。此来源问题另立preserve-proved-discarded-call-origins；不把它混入局部类型决策。root完整保存后恢复build.rs到当前HEAD并移出临时永久测试；类型补丁须在独立来源片完成后重新应用并复测。
