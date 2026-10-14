# 最终整类与前向条件时序的 root 验收

2026-10-14，frozen CLI v11 SHA5fe98150a52b055b8bbb9bdb840c0642fb8de35f9128a7f50d42282e113165e7。focused-v11实测447/0/1，root独立校验全部raw/pins/命名门，4876源码/fixture闭包不变，峰值888367574<1GiB，5GiB/default stack/1秒守卫及exact finally clean实际通过，target absent。一次ignored是明确诊断测试，不计通过。

最终matrix-v2四份完整捕获：baseline debug/none各27命令，admission debug/none各31命令，共116命令/232raw。双JDK8/23、debug/none、default/all，16个Jarde完整类全部compile/runtime成功；完整17方法baseline的45行与13方法admission的33行逐字同各自原类。root实际主审计accepted/errors空，核完整输入/源码/report/class/Signature/物理来源与controls。8个baseline JADX编译失败无runtime，8个admission JADX成功同33行，真实分类保持。先前matrix-v1失败仍在独立失败归档，不能用新成功覆盖。

同最终CLI的forward-order-v2：双JDK/default-all四个完整16方法源码均未手修，16命令/32raw及12行原oracle一致（SHA115862c1b9ac2e857cf2b26f1a7f09836c9735c6857077df7d7c0d55374a279a），root独立核64方法maps/身份/UTF8/语义BCI、完整编译class库存、实际raw与source==render text。字段10→RHS写100→最终11，producer/test/arm顺序、null length在branch前抛、Runtime/Error RHS未执行最终写均同原。

BI控制：首先错误强加2026-10-05历史“3 consumers拒绝”，4个成功render后的断言失败，无完整execution receipt，原8raw保留。当前frozen v6已恢复BI；v11与其两个编译器class及原历史bi.jar输入在default/all完整类正文、方法map、物理身份均逐字不变。新私有certificate对实际回边/循环仍None；不撤销其他既有证明。规划已按这些确切输入纠正，不改产品迎合旧快照。

保留focused-v7执行器旧计数417与实际418不符、v8最后计费后取消/错误dup读次数断言、v9 dup仍未构成真实二次读的失败raw与finally clean。最终v11用两臂各自dup的真实2reader，3-input/mixed/live/BI/handler及晚预算/取消门均实际通过。未删除或改预期规避失败。

归档排除安装JDK/JADX/CLI大二进制；包括实际工具、冻结CLI构建收据、相关生产/测试源码逐文件匹配v11 source pins、完整输入和全部实验raw。archive-review逐一复开所有成员核长度/SHA。纯goto已有映射缺口继续单列，不宣称全指令映射。

fresh双seed workspace、额外CI runtime门、自身产品CI和clean-main交付仍待接受，本记录只关闭整类/时序验收，不增加整CF11/71单元/612测试计数。
