# 外层 try 的同次循环子调用诊断

产品7e93上的原exact integration test真实通过1/0/0（7 filtered）。固定handled中，原protected region_at(0)返回Straight[0]/next7；在同一次canonical/view/SSA和protected frame下，实验region_at(7)实际返回Fallback(ExceptionEdge BCI7, ordinal0)，next=None，visited无新增。尚未进入loop body frame，不能称该层已被运行验证。

六个Walker可变状态均恢复；预算不退款，Err路径恢复后传播原Stop。原最终uncovered仍[7,16,47]。这是诊断观察，既非产品修复也非完整类验收。root逐raw/hash及全部4854 live source核验，源码字节/mode恢复，finally exact cargo clean成功、target absent；默认Rust栈、5GiB/target1GiB/1秒进程组守卫。

11证据成员加嵌入members.json共12文件，逐成员复开；archive精确大小/SHA见manifest.json。原v1/v2证据保留，v3只增加一次诊断child walk。
