# 原始局部布尔标志与拒绝控制

`original/debug`、`original/no-debug` 的 outer/child class 及 plain JAR 原样复制自已接受的 TestBreakWithLabel 完整原始基线；完整上游 Java、LICENSE/NOTICE 在 upstream。源记录及全部原始/JADX/Jarde 失败见 openspec/evidence/java-syntax-2026-10-19/cf09-testbreakwithlabel-original-root-v1。

`controls-src/BooleanLocalRefusals.java` 是独立控制。使用已 pin JDK8 的 `javac -g -encoding UTF-8 -d <jdk8-output> <source>` 与 JDK23 的 `javac --release 8 -g -encoding UTF-8 -d <jdk23-output> <source>` 编译完整类，再执行 `python3 -B prepare_controls.py --javac8-class <jdk8-output>/cf09/literalboolean/BooleanLocalRefusals.class --javac23-class <jdk23-output>/cf09/literalboolean/BooleanLocalRefusals.class --out <controls-output>`。

改写器从唯一 found:Z 调试槽核完整方法两次直接常量写入，包含 LVT 起点之前的初始化；只把 true 的 iconst_1 改成 iconst_2，长度、分支、异常表和其它方法不变。最终 controls/jdk8、controls/jdk23 各保留完整改写类，原编译产物和全部命令 raw 留在 OpenSpec 验收材料。它们是明确的构造边界，不是原始 JADX 输入或执行等价证据。

两独立物理解码器核完整新增树：6 class、20 Code、0 handler、20 direct branch targets、0 jsr，见 manifest.json。JAR 中重复原件不重复计入 Reader 的 `.class` 文件扫描。Rust Reader、指纹、恢复及运行验收由本片任务记录分别声明，不能从物理库存推断已通过。
