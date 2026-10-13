# 普通 try/List 的45行完整基线补充

产品597010、冻结CLI fe00fa7f的实际观察。root实际执行独立auditor-v4为audited/errors空，双JDK/debug-none/default-all未手修完整17方法类：8条Jarde整类编译运行各45行同原，8条JADX整类编译失败/no-runtime。五个目标方法仍raw List，unknowncall仍拒绝，不把本基线当新实现通过。

原44行逐字保留为每种mode/JDK输出前缀；新增可变output的handled正常完成行为 `try-catch-handler-mutable-output|exception=none|left=[one, two]|right=[one, two]|effects=2`。旧17行oracle不变。实际exact cargo clean为0，target前后不存在。

`capture.tar.gz` 的251成员由root独立逐成员复开，对嵌入manifest、原文件与sidecar逐字/bytes/SHA核对。118 raw绑定包括本次108份、旧原类oracle8份和最终clean2份。四个输入jar仅有一份原class，逐mode/JDK与自身原类恒同；大型CLI/JDK/JADX安装二进制不打包。旧44基线及4860编译来源闭包见[前一基线](../baseline-product-597010-root-v1/README.md)。纯goto已知映射缺口保持原记录，不声称全指令覆盖。

root在执行前审读236行归档器，只纠正旧none receipt SHA的文字转置，未改变证明或产品。auditor-v3内存bytes不能JSON序列化的问题在执行前发现，v4只将输出元数据投影为bytes/SHA，比较仍使用raw字节；没有弱化验收门。工具review、真实接受结果和逐成员review保留在本目录。
