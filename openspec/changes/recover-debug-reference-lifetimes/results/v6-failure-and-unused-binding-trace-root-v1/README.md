# v6失败与unused身份同次诊断

真实guarded库测试395成功/8失败、finally clean成功。保存精确失败来源、receipt及六份raw；测试预算默认值全部为零的错误已与实际LVT拒绝分开，未改失败断言。

root在同一真实rawAliasEscape IR上临时instrument matcher，观察rows=[(32,41,value),(74,82,value)]、reads=[33]、by_bci={30:0,33:0,72:1}。原测试仍失败，说明第二String身份只有store72、没有局部读取，v6的每行至少一次read门误拒绝。诊断不是产品恢复通过。工具、原/临时reuse来源、命令、receipt、四份raw及实际fixture逐字保存；全部4856来源在运行内恒同，outer finally恢复reuse字节和mode，root逐来源核恢复、target absent。

据此修订既有OpenSpec：无读取身份另核唯一已证明Store与row起点的decoded邻接及身份范围，仍保留alias/CFG/owner等旧门。私有新候选尚待实际验收，任务保持1/6。完整Upstream fixture已存在根tests/fixtures，失败时多余副本随后删除；实际编译产物未重编译或修补。
