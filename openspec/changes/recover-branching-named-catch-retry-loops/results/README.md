# 条件重抛与异常重试：验收入口

产品 `618b5e20f27f94ba28ea2f9e4310648718475f82` 已接受，任务6/6。复用既有异常重试与结构组装，没有新增恢复机制。

- [同轮IR根因定位](same-run-ir-root-v1/README.md)：实际首次拒绝、重复owner路径及临时源码恢复。
- [定向门与完整对照](focused-and-full-comparison-root-v1/README.md)：冻结CLI、完整原件、可达控制例、失败历史和独立复核；32个原件Jarde VM/64观察、32个控制例Jarde VM/160观察均同原件。
- [完整工作区与产品自身CI](native-and-own-ci-root-v1/README.md)：393目标/3823通过/0失败/98既有忽略、官方原始步骤日志/双seed结果、Git来源绑定和root最终验收。

原件catch动态不可达，运行证据由独立控制例补充；物理负例只做双JVM load-only检查及Reader/CFG/SSA拒绝验证。保留JADX的JDK8框架边界，不宣称任意异常循环或全部JVM属性等价。固定13原件7项完整接受、6项待基线；CF09/71/612保持partial。
