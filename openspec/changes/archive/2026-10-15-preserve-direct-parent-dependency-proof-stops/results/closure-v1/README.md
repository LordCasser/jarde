# 规范同步与归档

复用本次唯一 specs instructions 快照，将 delta 的1条新增需求/8场景原样追加至 java8-recovery 主规范；原63029B前缀逐字保持，17/110变为18/118。sync-receipt记录对照及无剩余delta。analysis-contracts 未变。

任务6/6且 apply 为 all_done；实际CLI正常 archive --yes --skip-specs --json。此处 skip-specs 仅避免重复应用已经完成并验证的 agent sync，不跳过主规范同步或验证。归档前两版 strict 各364/0，归档后各363/0。archive-execution保留命令和raw；product-source-parity核实4905源与冻结CLI/产品commit一致，target缺失。

本目录收据的路径指向当时活动目录属历史事实；归档保存原字节。本目录不把文档head当成产品CI head。最终提交/主分支绑定另以实际Git交付记录为准。
