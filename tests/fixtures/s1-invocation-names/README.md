# 调用名称边界的固定输入

2026-10-23。四种javac8 target8/no-debug真实调用形式：static void、virtual返回值、private invokespecial返回值、interface返回值。每种四合法名称ASCII/BMP/supplementary/`_`，另dash/孤立surrogate仅改CP名称，合计24 JAR。生成器、原源码、编译标记、原始JVM双版本oracle和逐项SHA在OpenSpec结果s1-invocation-name-boundary-v1/permanent-inputs.json及其引用目录。

合法原始16组已编译与双JVM非空输出核验；变异8组只解析恢复，不编译不执行。该组为同源开发输入，不是holdout。旧harness空源码错误原流已保留，不计有效恢复；corrected-baseline-v1从真实CLI文本重新验证，30/32生成编译与60 JVM一致，private `_`声明/调用不一致为已复现失败。
