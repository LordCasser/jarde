# 主代理最小实现独立对照

2026-09-23，debug CLI由当前source cargo build -p jarde-cli --locked构建。逐个以javac --release8 -g:none编译sources中的自写类与runners，再将CLI的完整实际stdout保存并原样javac，使用相同runner与java -Xverify:all对照。

StaticStraight=7，StaticConditional两条新进程路径=1/2（第二次设置-Djarde.static.missing=present），StaticLoop=3，StaticTryCatch=7，StaticHelper=11，VoidControl=1；6类7次全部一致。每个子目录保存实际生成源码、报告、javac日志和两份运行输出；class只在/tmp/jarde-static-root-after。

20项emitter unit测试已过，包括新尾返回闭合映射、empty0语句、普通声明、嵌套return和输出预算。source证据预算、空实际clinit、合法提前返回以及初始化失败的完整独立审计还待收尾，不能把本记录读作所有clinit均可恢复。
