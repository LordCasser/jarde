# 循环内 switch 非局部出口

SN、SM、SL、LD完整源来自2026-10-05已冻结patrol；本目录两个编译器目录的八份完整class逐字复制自2026-10-18 root实际红基线，均为Java8 major52、`-g -source 8 -target 8 -encoding UTF-8 -proc:none`。安装fixture时没有重新编译、删方法或重写源码。manifest记录源/class SHA256、完整Code/BCI与原命令/工具pin，原始流可从其指向的OpenSpec基线归档复开。

SN覆盖case-return有尾/无尾、if控制及break outer；SM五项局部break/continue/if控制；SL混合break/continue/return，循环局部join68与continue71不同；LD包含区别JADX错码的`{9,1,5}→10`。新外部driver的双JVM原始436case另有实际收据，不能把输入冻结当作修复验收。
