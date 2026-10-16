# 循环continue/break finally handler原始控制

LoopTransferFinally.java保留历史FX的finContinue和finBreak完整源切片，仅外类名改为独立类并不包含其它方法。各61B Code/35BCI、三catch-all rows与BCI31 continue53/break59需从真实class和IR核验，不以形状推断SSA。

真实javac23 --release8和javac8 -source8 -target8均-g:none编译；固定未改driver在Java8/23共四次运行逐字匹配168B oracle（11行，含落穿/continue/break/重复/负数/溢出/null）。受保护body没有driver可直接触发的同步异常，运行不证明handler清理/重抛，仍需完整实际Facts证明。

旧冻结CLI464543594/b4a6b2ce…c9c851对两完整3方法类exit0，完整生成源码原样javac0，但两方法仍fallback并保留handler45 bytecode。原始控制没有重建运行，不能把javac成功当恢复成功；12命令原始收据/24流在独立控制证据中保存。
