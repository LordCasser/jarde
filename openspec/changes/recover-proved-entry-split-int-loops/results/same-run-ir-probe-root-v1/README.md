# 同次真实 IR 探针：root 已接受

完整输入 `cf08/MultiEntryCounter.class` 为271B，SHA-256 `fe1a87ea523c10649ea3141d89b9ce04a2c2cfb6e8c31e70479193794311e2fb`；生成来源与原/JADX/Jarde行为基线见 [固定反例](../../../../evidence/java-syntax-2026-10-19/cf08-multientry-prefix-baseline-root-v1/README.md)。本次只分析同一次真实IR，不是生产实现或循环行为验收。

实际执行 exact Cargo probe：1 passed、0 failed、506 filtered。完整执行收据SHA `998950251afc9f8bb4b758720da7ce6b0a9743437d92cd8758295abe255105f4`；构建收据SHA `1cff4069bbac826560f921455e68b9467b92c832ce3afab72f841662550f5e0b`。默认栈、可用磁盘5GiB门、target1GiB门/每秒守卫通过，峰值168305572B，实际cargo clean成功；3215来源在构建期间不变，临时module/fixture事后删除，region.rs逐字恢复，target absent。root另逐字复开3212仍存活来源及3份临时快照，不把临时测试当永久门。

真实图为五块0/4/7/11/17、六条正常边，SCC为4/7/11且外部入口4/7。无异常、throw或不可达块。SSA有6 phi、14 value、9 effect。prefix入口值v3有输入v0/v6，header值v6有输入v0/v10，prefix写v10；计数器prefix值v5有输入v2/v12，header值v8有输入v2/v5，latch写v12；return消费header的实际值。不能要求不同位置ValueId相等，必须逐前驱证明phi和槽绑定。frames全Int只证明JVM类别，descriptor `(IZI)I` 仍含Boolean，Builder必须核源码严格Int。

[capture.tar.gz](capture.tar.gz) SHA-256 `17c73978213971950c5739b6b795a92f132d460310898d3dbcd33694502238ef`。33 payload和1内嵌库存均由root重新打开，与实际结果、runner、原prep和归档prep逐字及mode核对；见 [inventory](inventory.json) 和 [独立复核](archive-root-review.json)。归档保留完整raw、两份region快照、实际class、探针源/patch、架构裁决及OpenSpec快照。原prep README是执行前历史说明；归档prep README的“6个pin文件”笔误以实际7文件+pins.json库存为准。

任务1.2的事实探针部分接受；永久正反fixture和生产实现仍待集成验证，不提前勾完成，不增加CF08或71/612整项计数。
