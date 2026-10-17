# 两编译器封存输入的实际 Facts / SSA / 首拒

root在0cf8同源4c6852上实际编译并运行单个cfg(test)探针，1通过/0失败。两907B输入均Complete，两块0..10/10..19、一条catch-all [0,11)→10、一条canonical异常边0→10，没有正常边或自边。可抛点2/6/9仅受row0保护，cleanup调用13与重抛18无handler；store10不可抛且只是范围内的绑定。捕获phi合并三个真实Caught，Local7与load17新Stack10分开核对，不能把二者直接比较。全部11物理BCI保留在raw。

真实generic copy首拒是save索引：before_handler=[0,2,5,6,9]、handler_cleanup=[11,13,16]，cleanup_start=0后checked_sub(1)=None；不是有第二份normal cleanup未识别。resources/examine均拒绝jre_guard_finally_copy@10，普通exception-only cleanup grammar也实际false。

capture含实际raw/argv/计量、两原字节、原始及临时instrumented guard。临时诊断已恢复到原SHA65c845697752e07e2857583e332fac88fa84397bc0986cbdd7f27bc5c30937c2；default Rust stack、资源守卫和最终exact clean/target absent通过。此步是分析证据，不是功能恢复。私有patch曾因同文件重复操作/逆序hunk被apply_patch拒绝，修正顺序后才实际应用；没有把失败应用当工具链运行。
