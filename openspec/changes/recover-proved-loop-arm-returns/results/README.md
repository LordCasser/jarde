# 当前证据与任务状态

当前2/8，仅原始基线与真实首拒已接受；尚未集成生产修复。固定71/612与CF-11整体计数不变。

| 归档 | 实际范围 | 成员 / bytes / SHA256 |
| --- | --- | --- |
| first-refusal-root-v1 | 11外层命令含八API+guarded build/clean；完整canonical/normal/SSA/effects、ForHeader/terminal/gateway；非usage JSON恒同、源码完全恢复、target absent，root独立接受；未执行错误草稿单列 | 43 / 1257613 / 564087ec09f53b39ded089fec47ff83c8a1c3cecc1ff46e0a9ba80fd199075b3 |
| original-baseline-matrix-root-v1 | 42命令/84raw，24原样全源码编译；Jarde8javac23缺返回/8javac8框架边界，JADX4javac23成功、8core双VM24case+4check23case同原、4check8边界；root独立review通过 | 112 / 490333 / 7ed8bfb27eb2bfe77342d30f8ed1bb4e172575df49ad7a79040794ef6c79a3e8 |
| original-baseline-root-v3 | 28命令/56raw，完整原始源、outer+child debug/none、8API全3方法/全物理BCI、4core双VM12case同原、2check23通过/2check8 major55边界；另保留早期8命令16raw的collector失败，不能并入成功 | 129 / 868355 / 7700e8fbb3f27bccfaf8be73cd6e03f33c753487059369a7ce36f0044f376a4c |

三归档284成员由root独立逐字重开，见baseline-archive-review-root.json；所有payload bytes/SHA匹配，未执行草稿与失败只按原范围解释。CLI二进制不入包，精确pin保留；JDK/依赖按原路径SHA绑定，未宣称随包提供。

首拒实际到达loop_arm_join_source并返回None。日志已证明child16完整Loop、return29、normal join37、ForHeader15/31/slot3、gateway34；对closed_body首个谓词及后续natural-only/terminal/latch契约的说明是root对同一AST与当前源码的架构推导，不能说成另取了helper内部逐谓词日志。

补充A/B已65命令/130raw终态：8target、16原VM48case各同自身原oracle、16Jarde run fallback/constructor structured、8JADX提取成功；独立审查及完整重编矩阵仍待，不计任务1.3通过。Luna正私有准备实现、公开测试与补充验收，root独占共享编辑及工具链。
