# S1 2.9 名称边界 fixture

RawAlias.java 由本任务编写，使用本机固定 JDK 8 `-source 8 -target 8 -g` 编译。RawAlias.class 是该编译的原样产物，未手工修改；摘要与实际命令见 manifest.json。

reader 测试只替换 CONSTANT_Utf8 中 `probe` 的名称，不执行变异类。恢复矩阵另只替换局部调试名 `abcdef`，在受控正例中比较原 class 与生成源码；损坏编码负例不执行。两类替换不混为一次变异。

完整编译/运行原流位于 OpenSpec change 的 results/s1-raw-names-v1。该 fixture 无外部依赖。
