# 循环内局部更新 finally

NoCallFinally.noCall 的method源码原样来自FY.noCall，独立外层类不改变该方法；两份class的62字节Code及异常表与对应FY一致。v8为javac23 --release8，v8-javac8为真实javac8，均336B。未修改驱动与84字节oracle来自真实原类Java8/23四次运行。十个输入包含空、命中位置、重复命中、无命中与null。driver不能直接观察待return后的局部清理次数；该责任另用物理/SSA证明验收。

规划基线：两份完整生成Java仍因missing return重编失败，没有rebuilt运行成功。实际12命令控制、Facts/Region探针、失败preflight及源码闭包保存在OpenSpec本片results/planning-baseline-v1便携包。新增永久fixture仅本目录5项非md文件，不删除或改旧样例。
