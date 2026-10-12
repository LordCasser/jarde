# 原始 JVM 字段拼接控制

此包证明输入的真实行为，不能关闭子项实现或Jarde整类门。完整raw、工具、命令、源码、class与SHA在manifest；归档已逐项重读校验。

- original-root-v1：第一版混合fixture，18命令/38raw，六lane原运行；随后拆分正向/反向整类，不能用它替代v3输入。
- original-root-v2：v3分离fixture，36命令/74raw，六lane正向与负向原始源码全部运行。前四lane source8/target8（Java8/23×debug/none），后两lane javac23默认target（debug/none）。
- null-original-root-v1：12命令均成功，但共享Runner在执行期间被修改，source闭包不一致。root-rejection保留，拒绝same-source验收；不改写其execution。
- null-original-root-v2：fresh12命令/26raw，4853来源一致，root独立验全部流、六个NPE与此前其余10个oracle逐字相同；target absent。仅原始行为接受。

实际本地Java23 default的plainPlus已在后续getfield之前调用String.valueOf(Object)，因此本控制没有复现历史indy转换顺序缺陷，不能声称Java23 plainPlus错误。候选对新许可链显式Object valueOf是锁定原overload与位置，仍需新产品真实运行。字段negative local extra use不等同直接getfield SSA多reader；证明层多reader反例须另测。

四组永久class与root补充Runner的来源在fixture manifest；default-target class仅保存在证据包，不误计为Java8 fixture。此归档不会增加71/612或父项完成数。
