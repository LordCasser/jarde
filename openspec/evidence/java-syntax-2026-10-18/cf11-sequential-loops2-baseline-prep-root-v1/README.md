# TestSequentialLoops2：待执行基线

这是下一固定 CF11 用例的准备包，尚未执行，不代表已发现 Jarde 缺陷或完成语法单元。root已完整读Luna collector并修正类access读取和工具after/finally重新pin；8-case driver已审，AST和driver/parser固定hash核验通过。8成员归档已逐字复开，SHA256 `70f2e660a06cb040fb9d51d0b956ead404679b667adf0ee82173b7d077d77bf5`。

原source保留 `lowercases={'a'}`，default真实异常与configured128运行时控制分开。双JVM原始stdout/异常消息各自保存，稳定语义projection比较返回UTF16/对象同一性或异常类型，消息不作跨JDK等价门。完整outer/child输入、<clinit>物理Code、真实javac8框架边界不裁剪；本准备只取baseline，生成源码重编矩阵尚未准备/执行。

本机可执行草稿位于 `/private/tmp/jarde-seq2-baseline-prep-root-v4-20261018/collect_seq2.py`。须在当前Seq1两seed和自身CI交付后，确认target absent与原source/CLI/JADX/JDK/依赖引用pins仍可用，再以新out目录执行。若换机器，应先恢复真实依赖和工具并重新核验；不能把该包当作运行收据或直接删改pins来略过验证。
