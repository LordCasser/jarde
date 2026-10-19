# TestDoWhileBreak 完整原始缺口

完整上游源码563B，SHA `7c8db1024a14626aaafdd977a722665e077f8e17f5751f2249bc7be6e9eba577`，outer与child未裁剪。产品050及冻结CLI SHA `6c83ec82cd4c6caee86c08e7fefddc2fea675952f936c0e4aa3f295ed9be7fc6` 实际46命令：八API目标均Fallback/ArmsDoNotMeet block13，16次Jarde整类编译均失败，没有Jarde目标VM运行。目标14条物理BCI在注释中的map有效，不能当成可执行恢复。

原件四VM/八观察和可编译JADX四VM/八观察相同：k=9返回5、k=10返回0，异常/stdout一致。两次JADX JDK8测试框架编译失败、两个生成类错根及一个原件错根控制完整保留。有限case不能区分等价while，不要求特定do拼写。

归档 `capture.tar.gz` 为939,364B，SHA `d6e22ae912670e760d00df7051da3fa124fdf712a2918159a622c1446de0d43a`；161 regular payload、201目录及inventory，root逐成员复开bytes/SHA/mode后复制到本目录再复开。原始命令与全部失败、源/driver、独立审查及root接受均保留，见 `root-review.json` 与 `copy-and-reopen-review.json`。

后续[单块出口前缀OpenSpec](../../../changes/recover-proved-post-test-loop-break-prefixes/tasks.md)将原件基线与[同轮真实IR](../../../changes/recover-proved-post-test-loop-break-prefixes/results/same-run-ir-root-v1/README.md)分开。此处仍是实际缺口证据，没有实现成功声明。
