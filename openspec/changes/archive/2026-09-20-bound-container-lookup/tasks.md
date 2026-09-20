## 1. 固定证据与定向访问

- [x] 1.1 按 optimize-demand-workloads 的 W1/W2/W5 与测量门禁固定 baseline/candidate 构建，扩展现有 P5 harness 记录目录解析、nested 物化、阶段/总时长、请求 Budget 与语义 fingerprint；以小/大 flat JAR、STORED/DEFLATED nested WAR 重现重复工作，验证准备成本、CP/Header 产品策略和测量波动可独立归因（A15/A18）。
- [x] 1.2 从 nested replay 抽出 reader 共用的单 container 访问，并将 ZIP/tree 查名改为定向访问；验证目标与 ancestor 可读、未搜索 sibling 物化为零，坏 sibling 不阻止局部结果，显式整树仍报告损坏（A08/A14/A16）。
- [x] 1.3 建立完整目录的多值 raw-name 定位表并贯通同次操作的查名、绑定与 entry 读取；验证跨请求 cache 关闭时仍持有事实不重建、duplicate ordinal/origin 保留、稳定顺序、伪造 origin/metadata 拒绝、目录未完成不得判 Missing（A07/A14）。

## 2. 有界复用

- [x] 2.1 扩展现有 FactsCache 的 container 产品与物理/schema key，保留已验证目录及 backing；验证同 snapshot 的不同方法复用、内容/chain/schema 改变不命中、Partial/Cancelled/损坏项不发布（A15）。
- [x] 2.2 为同一 store 增加 entry 与 retained-byte 双上限及统一权重核算，覆盖 container 和已有 CP/Header payload；验证 shared backing 不漏计/重复计、容量不足不重扫当前请求、drop/clear 释放持有引用、开关默认 off（A14/A15）。
- [x] 2.3 将 selected entry 读取接入权威目录/backing；验证 warm 路径不重新扫描目录、不重复解压保留的父容器，仍校验所选 class entry 完整性及伪造 metadata；覆盖 root、两层嵌套和重复条目（A08/A15）。
- [x] 2.4 分开报告实际工作 usage 和 cache 复用，命中应用当前取消/时间/结构限制与输出预算；验证预取消、到期 elapsed、已终止 Budget、较低 nested depth、output budget 和容量退化不会重置预算或伪造 Complete。对已验证 DEFLATED backing 明确核对 archive_entries/read_bytes/entry_bytes 的节省及实际 class 读取仍计费（A14/A15）。

## 3. 集成验收

- [x] 3.1 对固定原始 B、定向直接 D 及同候选的 cold/warm/容量不足执行同方法重复和多方法消融；跨路径足额完整结果核对定义、顺序、诊断、coverage、质量、text/source map 的语义 fingerprint，相同初态的完整报告只排除 elapsed_millis（含 usage）。紧预算验证真实停止，不强求跨路径 usage 相等；分开记录准备/序列总成本、retained weight 与 RSS，并按总专项给出实测收益或未证实结论（A15/A18）。
- [x] 3.2 运行适用 reader/resolver/recovery、R8/R9、X0/X1 隔离与 CLI 回归及 fmt/clippy/OpenSpec strict；更新 cache API、成本边界和验证记录。主规格同步及归档只在上述验证通过后进行，不修改其它 change 的状态（A13/A16/A17）。
