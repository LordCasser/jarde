# 根实现与验收记录

## 2026-10-15 Public old-production comparison

基线产品d3e2b2457338b61660b842f5705b225bba08a73a已整合main7d0a169ca21b714efab31d7a23d661fb90cd5e40。根只新增公开测试，production与基线逐项恒同。library first abstract-code的Name先通过歧义断言，首listed-ID继而错误执行（0/1）；不声称旧九组27请求全部到达。CLI-v1测试读错usage路径先发生Null/0，作为测试错误保留并排除产品反例；按实际execution.usage修正新测试后，fresh CLI-v2真实list→首listed-ID实际exit0、期望exit3（0/1），第二ID/Name尚未到达。原失败断言未删除、未跳过或改期待。两个receipt各4900来源前后相同，guard与finally clean通过，独立review接受。

## 2026-10-15 Scoped fix and focused gates

Method在原foreign-owner校验后与Name复用同次完整成员表候选路径；删除首取declared_method，不新增身份字段/公开carrier/跨crate选择器，不改body_result或reader/JVM选择。原index/flags/等值IDs均保留。

focused-v1根4952checks接受：76通过/0失败/2忽略，5suites、8命令/16raw；公共矩阵九有序对×Name+两个listed-ID共27次请求全部到达；abstract-code的三次请求均在method_bodies=0下返回歧义且零prepare/body/Code/IR。唯一abstract/native/Code、不同descriptor、同签名foreign-owner、missing/partial/stopped-body，以及完整task_operations、d2_prepared_handoff、d3_artifact_binding、d0_demand_counts和task CLI均通过。CLI真实列举后两ID和Name三请求均exit3，全部预算断言取实际execution.usage。

fmt、当前CI完整Clippy、OpenSpec1.11.0/1.14.1 strict366/0、CLI build、finally exact cargo clean通过。冻结CLI59599048B/SHA c0bc6b64e1c763ee400afdb60d328a83151091fecbd72ed8ead1e906bd170f45；源码4900从开始就包括全部corpus遍历输入，前后及当前相同。default stack，5GiB/1GiB/1s守卫无触发。

[本地完整归档](results/root-local-v1/README.md)62 payload/2327642B/SHA e41c6614a4e4c508fb91ddbc77ca8856673ce7621dc88a24c03e798cdc850c80，63成员由根逐字复开。独立focused review已接受（SHA d5eec32610768addc8641e8b668a687b681072802d223c723580f7aef882409f），2.1关闭；2.2修复产品自身CI/完整workspace及最终交付仍开放，不以本地76/0/2代替。两项proof-stop计划仍0/23，不增加CF11/CF18/71/612总体计数。
