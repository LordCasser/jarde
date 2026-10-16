# 规格同步与归档

复用本次唯一specs instructions快照，只把本delta1需求/7场景追加到java8-recovery；原66674B全文前缀逐字保持，18/118成为19/125，analysis-contracts12/46不变。全部所选delta重核已无剩余操作，主spec严格18/0。

任务6/6、apply all_done；实际CLI于2026-10-16正常archive --yes --skip-specs --json。skip-specs只避免重做已验证的agent sync，不跳过同步。归档前两版strict365/0，归档后1.11和1.14各364/0。4907产品输入与8261 Git blob、mode及新冻结CLI全部匹配，exact clean后target缺失。新static-prefix计划仍0/6，未被同步或归档。最终提交/main交付按另存实际Git记录核实；此目录不把文档head当产品CI head。
