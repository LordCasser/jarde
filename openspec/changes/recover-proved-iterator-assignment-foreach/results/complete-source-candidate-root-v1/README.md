# 新候选完整类三方对照

root 实际运行 fixed、controls 各29命令/58 raw streams，双JDK原2/JADX4/Jarde4各十条完整生成类运行腿分别同原八case/十二case。生成源码未手修；固定类四Jarde源码一致，两个Object foreach、原外层String声明及Assign/cast保留。控制类四Jarde源码一致：positive一foreach，noCastAssign/bodyEffectBeforeAssign/extraNextConsumption三while。全部8/20方法来源profile与真实物理指令核准。

独立验收v3由root在2026-10-11T22:22:41Z实际执行exit0，只接受冻结案例观察；acceptance SHA256 `cb3289007202fec4fadc831c000826201f84b518856a7119ddeab33ae52fd55a`。v2实际argv比对失败保留；v3根据被冻结launcher准确选项顺序重建23项参数，未改collector/capture/产品。全工作区、lint、确切产品自身CI另行验收。泛型/typed header/空声明清理/类型债务不属于该片。
