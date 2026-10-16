# 调用返回循环finally原始样例

CallReturnFinally.java逐字复制原FX.finReturn与c0源码切片，仅外围类名从FX改为CallReturnFinally；self-call owner随真实编译绑定新物理类，不能把它当作原FX方法身份。原Java、CallReturnFinallyProbe.java与217B probe.stdout不改。v8/为真实javac23 --release8 -g:none，v8-javac8/为真实javac8 -source8 -target8 -g:none；class均从已记录实际控制运行原样复制。

基线产品02de294f7af2d0798e33d52ce5474e60efe17b03/冻结CLI fd7b33ce0411854e480186c7ade82ecc21d4401b79d1a0b1bd0d5c7774bbff25：12命令，四次原类Java8/23运行等于217B oracle；两份完整生成Java在line19 missing return，尚无rebuilt运行。actual Facts/Region另2命令/1test观测，临时cfg(test)打印不是产品恢复。

finReturn为127 Code字节/60指令/max_stack3/max_locals7，两catch-all rows [17,30)->88与[88,90)->88。两编译器Code操作结构相同但常量池index不同；c0为6字节Code，不声称整个Code attribute相同。真实CFG七块八边，protected c0@25可能抛异常，cleanup副本30/59/90起始的多调用链均在保护区外。c0纯函数的次数/时点及受保护异常路径不能靠driver输出直接观察，需SSA/effects/来源证明。driver观察返回值及清理顺序、次数和抛出；不改原调用或oracle。

本目录5个非Markdown输入已登记corpus-fingerprint.json；reader census预期delta为+2class/+6Code/+4handlers/+8targets/0jsr，必须由实际reader测试确认。原完整FX剩余finContinue/finBreak失败继续另存，不删方法。
