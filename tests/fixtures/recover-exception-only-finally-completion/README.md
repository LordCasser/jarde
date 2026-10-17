# 仅异常完成finally的完整输入

source/ExceptionOnlyFinally.java是从零定义的完整独立类，含constructor、mark、tryThrowsFinallyRuns与clinit。目标方法与原例同体：mark(7)后throw TRY_FAILURE，finally mark(8)；目标Code19B/11指令/单catch-all [0,11)→10。mark记录trace并经PrintStream让独立driver在7/8注入none/RTE/Error，核九组合的78次序/一次执行及精确对象identity。没有从原FinallyCompletion删除方法。

v8-javac8由真实Corretto8 -source8 -target8 -g:none生成，v8由真实OpenJDK23 --release8 -g:none生成，各682B。sealed/source保留原完整FinallyCompletion七方法源及runner，sealed/javac8、sealed/javac23分别是907B冻结输入；root已从保留的原source双编译并逐字重现这两份原字节。两显式finally非局部完成仍为范围外红项，不能删去或改生成文本凑编译。

实际编译、javap、工具/源码/字节pins见对应OpenSpec results/frozen-inputs-root-v1。本README只声明输入取证，不声明当前反编译已经恢复或runtime已通过。
