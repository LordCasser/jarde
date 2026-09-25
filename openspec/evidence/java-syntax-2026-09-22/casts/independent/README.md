# 显式引用转换：主代理独立执行对照

2026-09-23，以 javac23.0.1 --release8 -g:none 编译本目录两个自写源码；原 class 经 java -Xverify:all 执行。CastAudit 包含直接、接收者、数组、多维数组、嵌套、字段、局部、调用结果及无用局部的检查。driver 23 行结果同时记录值、CCE/NPE、调用计数及检查失败阻止后续调用。

root重建CLI后，整个CastAudit真实stdout直接javac编译成功，23行trace与原class逐字一致；没有修改恢复方法体或删去成员。生产build.rs在构建前后SHA-256一致。证据为jarde-after.java.txt、jarde-javac.log、recovered.txt与original.txt。

初版自写输入含static字段初始化，位于with-clinit/：生成static块末尾保留了return，javac报告“返回外部方法”。这是另一个源码结构缺口，已保存真实失败；没有删除恢复文本里的return。主验收输入改为由driver显式赋初值，再重新javac生成原始class，用新class完成上面的整体对照。早期CLI快照已经含候选cast实现，归档为candidate-before-root-build，不能当作修前红证据；本change原有CastProbe修前证据另存上级目录。

jadx1.5.6会删除unusedLocal中的String检查：new Object输入由原程序CCE变成返回2，后续计数也增加。jadx对照仅去掉自动添加的defpackage包名，不修改方法体；其输出不是jarde的期待。

重放：在临时目录以 javac --release 8 -g:none -d classes CastAudit.java CastRunner.java 编译，再用 java -Xverify:all -cp classes CastRunner。反编译读取同一CastAudit.class，使用class-source / single-class / release8 / text，stdout与stderr分开保存。将真实stdout作为类源码，与同一driver编译执行。
