# 外层try十场景完整类基线

当前产品7e93/v12冻结CLI上真实运行24命令/48raw/20次完整类编译。原始UpstreamNestedLoops整类与独立Proxy/lambda harness在Java8/23、debug/none四腿均编译和运行成功，每腿物理class set恰为target和harness两类、10行输出逐字相同。RuntimeException在iterator/hasNext/next/output.add四边界被catch且effects=1；对应AssertionError逃逸、精确type/message/events、effects=0。success输出[value]/effects1；empty输出[]/effects0。以上来自真实原始运行，不是预填oracle。

JADX两个rename profiles和Jarde default/all的16次未手修完整类编译均exit1/no-runtime（JADX原raw alias类型错误，Jarde还拒绝handled）；这是失败基线，不能关闭新实现的运行任务。原17场景Runner另在已接受基线保留，没有用此harness替换。root复核全部raw pin、四原输出和classset、无generated runtime、source/tool前后一致；5GiB/target1GiB/1秒守卫。

完整receipt/raw/编译输入及实际class产物和collector/harness pins在capture.tar.gz，root逐成员复开；精确数量/大小/SHA见manifest.json。未嵌入CLI/JDK/JADX工具二进制。
