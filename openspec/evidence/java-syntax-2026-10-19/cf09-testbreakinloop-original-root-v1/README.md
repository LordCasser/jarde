# TestBreakInLoop 完整原始对照

固定上游完整源码未裁剪，debug/no-debug、class/plainJAR、default/all共八API。产品 `050cedbd255ffaddf9e51acf8d2ad52fe573519b` 的冻结CLI SHA `6c83ec82cd4c6caee86c08e7fefddc2fea675952f936c0e4aa3f295ed9be7fc6` 实际采集94命令：Jarde 16整类编译、32隔离VM共160观察全部同原件，目标来源26/26。现有机制足够，没有新实现任务。

原件四VM/20观察、可编译JADX四VM/20观察相同；两次JADX在JDK8下因上游测试框架依赖编译失败，如实保留。18生成类错根与一个原件错根控制均在调用前拒绝。五有限case覆盖空数组、遍历、提前break、可达int环绕、null异常及字段副作用。

归档 `capture.tar.gz` 为1,012,471B，SHA `0bbcad9c7c53b0f40528451428165cc85e0fd76177b5799dc4d2b8c39358e164`；276 regular payload、347目录及inventory，root逐成员复开bytes/SHA/mode后复制到本目录再复开。完整准备、源/driver、命令原始输出、独立审查及root接受均保留。独立审查路径因工具接收目录而形成 `review.json/review.json`，原始路径诊断保留，未重跑工具链。见 `root-review.json` 与 `copy-and-reopen-review.json`。

这是一个原件的有限对照，不关闭CF09或71单元/612文件账本。
