# Loop4/5 完整工作区与本产品自身 CI

产品 `412245d514896b1d0eb9a8fe24388f6a70668637`，最终CLI SHA-256 `d29b6fee723d8f59fbecd9fd8be9f15a8ec73de3a90f5a2a29b24ff91e4af607`。root和独立Luna审计均接受。文档收尾提交不重复产品CI；本入口的自身CI明确绑定上述产品提交。

最终fresh native一次完整执行：394目标、103批、3841通过/0失败/98既有忽略，3939具名结果及96精确门。5299来源pin在格式化、运行、清理前后相等；默认栈、本机5GiB余量/1GiB target守卫均无触发，target峰值854,234,934 bytes，finally清理成功且target absent。定向最终CF12旧精确预算门9/9、Java核心540、公开API4、fmt和Clippy均通过，详见[完整三方对照](../focused-and-full-comparison-root-v1/README.md)。

[自身CI39251504476](https://github.com/LordCasser/jarde/actions/runs/39251504476)的stable/test and specification、MSRV1.88.0、supply chain、fuzz smoke四jobs全部成功。52个官方API步骤逐一绑定原始ZIP独立步骤成员；ZIP总60成员含目录和聚合日志，没有用聚合日志拼造步骤。两个固定seed5350648285461741569/1570各394目标、3841通过/0失败/98既有忽略，全部3939个(target,test,status)逐项一致，规范JSON行SHA-256 `4d7f47ae3006322df17f7baad9b6addba4abc12f834c775be52e73fee333fed9`。各seed有141精确门一次通过（旧123+新增18：Region11、公开API4、Builder3）；JDK25 oracle、完整Java8/整类/数组/example/依赖及规范步骤均由官方原流核实。

CI收集184个规范命令，384份命令raw，含8次传输尝试；元数据和fuzz日志各一次EOF失败后成功，失败原流不删除。root复开388条唯一stream/ZIP pin路径；独立审计另逐条核对。34源码及144 fixture共178条与产品Git show及实际文件绑定。构建3346/native5299的来源并集5306逐Git blob、bytes、mode核对；实际product-binding收据为准，不把早期准备计划中错误的native参数当执行事实，也不要求不同平台CI二进制与本机CLI逐字相同。

保留失败历史：首轮native与首版a4ccac6b4自身CI39248350582因非候选单出口循环提前支付完整scan费用，破坏旧CF12精确预算门。最终412仅前置既有两出口必要条件，不改旧365@29/130@1测试和真实候选收费；最终native及上述自身CI均重新完整执行。旧失败不能并入最终通过计数。

既有工具债务单列：workspace_runner.py将编译artifact顺序与summary顺序zip，observed_counts误归属316个target项、涉及95批。冻结verifier本来就以stderr实际Running executable头和stdout结果重建目标；root与独立审计所得394目标及3939具名结果逐项一致。原始收据未修改；下游不能直接消费错配metadata。本轮不扩大为runner修复，CI只消费自身原流，不借用native结果。

归档908个payload、46,121,981未压缩bytes；[evidence.tar.gz](evidence.tar.gz) 8,336,857 bytes，SHA-256 `33981500b6bf8eb7316a361220e11360bde95d85a1a075525c4abb3a23e0da34`。所有成员已按[库存](inventory.json)逐bytes/mode/SHA复开，[capture.json](capture.json)为实际回执。成员前缀包括native-first-failed/native、native-review/独立审计、own-ci/官方ZIP、ci-independent-review、product-binding、final-root-acceptance及final-strict-closeout；原绝对路径可由库存反查，工具原码及历史计划一并保留。final-root-acceptance/acceptance.json取代此前CI独立复核待定快照。最终严格OpenSpec为388通过/0失败，生产源码与产品412一致，未重新启动Rust。

本轮两OpenSpec各8/8；固定13原件为9接受、4待完整基线。Loop4来源23/23；Loop5来源59/60缺BCI103作为独立映射债务，CF09/71/612保持partial。
