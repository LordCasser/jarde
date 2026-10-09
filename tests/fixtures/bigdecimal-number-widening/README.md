# BigDecimal→Number准确事实控制

Main.class分别逐字复制冻结的javac8/23 Main-only baseline，不添加平台类；原source与stdout/stderr也保留。准确输入/hash核对见OpenSpec本片results/fixture-input-closure-v1.json。

NumberArgument.java由root使用Corretto8和OpenJDK23以Java8目标、空classpath/sourcepath、-g:none独立编译。原输入各只有NumberArgument.class，identity descriptor准确为(Number)Number；main的new/dup/ctor/identity调用BCI为3/6/9/12。实际原编译、javap、-Xverify:all运行及工具/hash见results/number-argument-original-v1/manifest.json。原stdout为2.50\n，stderr为空。

两个普通Rust测试只做完整生产API结构与来源检查；两个ignored测试需要JDK，以空CP/SP编译未修改的全类来源并仅运行新classes，原双流逐字对照。root显式分别使用JDK8/23执行，CI在已安装Temurin25上显式执行。命令移除JAVA_TOOL_OPTIONS、_JAVA_OPTIONS、JDK_JAVA_OPTIONS、CLASSPATH。优化concat warning允许保留，完整方法正文不得引用或丢弃System.out。

这两类控制不扩大24腿历史矩阵分母；NumberArgument是额外的准确调用控制。窄片不声称整个EM18或完整JADX语法追平。
