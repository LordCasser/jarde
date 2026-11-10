# S1 enum raw 名称输入

受控源码由 `openspec/changes/restructure-demand-driven-recovery/results/s1-enum-raw-names-v1/tools/run_pilot_r1.py` 生成，javac23 --release8 -g:none。源码、完整命令、原始class、两JVM oracle及旧CLI的原样重编失败在同phase的pilot-r1固定。除int-field.jar同时提供Args Header外，其余JAR只有本目标class；此处仅供Engine恢复测试，不执行目标。14输入为三个普通名称控制和11个非BMP名称变体，属于开发输入，不是holdout。

三项 `_` owner 回归来自 results/s1-enum-raw-names-v1/underscore-probe-r1 的正式原编译输入（javac23 --release 8 -g:none）；同源 X 控制均有投影，旧 Body R4 与 enum R1 对这三项均无投影且生成类不能原样编译。完整 class/oracle/命令与失败原流在该探针目录。新增测试只要求复用共享 Class 拼写门，不放宽 Name 标识符或 enum grammar。

## Constant-body raw owner边界（2026-10-25）

`body-owner.jar`、`body-owner-unspellable.jar`、`body-owner-bad-mutf8.jar`复用冻结的enum-string-field-name完整四class族。只一致替换全部CP owner/descriptor，保持CP索引、Code和CP后结构；标准UTF8 ZIP条目与MUTF classfile名称分域。三组分别是合法𐐀LabeledOps、合法MUTF但Java不可拼写的dash owner、损坏MUTF。来源/变换/pins见body-owner-manifest.json与results/s1-raw-enum-owner-boundary-v1/initial。仅供Engine恢复/拒绝测试，不执行变异目标，不是独立holdout。
