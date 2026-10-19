# DoWhileBreak2 / BreakInLoop3 同轮真实IR

在原产品region.rs SHA-256 `86a20041442a196dfaa83da539d3f301b9b4974907620b3b50da1eb46f037be5` 上，仅临时增加cfg(test)模块和日志。root串行运行1个精确测试、4个真实class profile，1通过/0失败；日志206780 bytes。真实输入逐SHA绑定完整基线，不用合成CFG代替原件。默认Rust stack、5GiB余量/1GiB target/每秒进程组守卫、离线Cargo与finally精确clean，源码逐字恢复，临时模块和target不存在；独立脚本重开原流及恢复pin已接受。

Do2：debug/no-debug均自然header0、members0/19、latch19，normal边0→17和19→31为出口，ipdom None。17是直接return leaf，31是普通退出返回；既有goto-prefix门在既有terminal_returns证明之前拒绝17，之后尝试pure header又在checkcast9效果处拒绝。应复用已有终端叶证明，不创造新循环类型。

Loop3：两个profile的normal-flow均无自然循环；物理row[17,28)→31/type32，canonical block11跨越lead(11,17)和受保护suffix，不能捏造block17。guard识别Try但retry facts None。prefinal树中顶层Try body拥有11，而catch then臂由37重开相同Try/handler31，重复11/31最终被whole-method OwnershipOverlap拒绝。retry37的物理末指令45跳回11；另一臂48分到61/70，汇合76，正常join82亦是受保护路径的完成出口。现有精确named retry证书是扩展入口，最终owner检查必须保留。

此probe使用同轮Reader/Code/CP/CanonicalCFG/SSA/Operations，但Sites::empty，不代表公开allocation-site/Builder或完整源码验收；只能据其定位结构。外部class、receipt、工具及原源码前后SHA相等，不能将临时日志当产品实现。

新fixture仅复制完整source、debug/no-debug outer/target class、全classJAR和Apache LICENSE/NOTICE，共18payload，私有准备/安装收据均入档；各repo fixture还保存manifest/README。没有把私有准备目录中的全局tests/fixtures/README覆盖到项目。

[归档](evidence.tar.gz)共43 payload、4,820,867未压缩bytes，压缩866,080 bytes，SHA-256 `ef1d508e1ec2c4f8d79e523421c8c3670a69c3032234e08dc5695ef42a47686c`；[inventory](inventory.json)/[capture](capture.json)逐成员复核。内含原始与候选临时source、launcher/guard收据/全部流、独立audit、fixture准备与安装及归档工具源码。参见[完整三方基线](../cf09-last-four-original-root-v1/README.md)及两个实现change的design/tasks，当前实现仍待完成。
