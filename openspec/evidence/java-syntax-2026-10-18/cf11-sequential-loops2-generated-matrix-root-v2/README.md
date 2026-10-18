# Seq2 生成源码完整对照验收

root 实际运行111命令/222 raw，并阅读、修正和执行独立 checker root-v5 接受。receipt SHA256 `56cdd258f72bd021c2db37068e0f292cc3f551e4410a322926b1e03f02292a71`。8份 Jarde 与2份 JADX 源码逐字复制，不手修、不删方法，分别用冻结 javac8/javac23 编译，共20格。

| 输出 | 编译 | rebuilt JVM / case | 实际结果 |
| --- | --- | --- | --- |
| Jarde | 16成功 | 64 / 512 | 全部匹配原 profile/JVM/mode oracle |
| JADX | 2成功、2框架边界失败 | 8 / 64 | 成功产物全部匹配原 oracle；失败格无运行 |

两份 JADX javac8 错误均为完整外层 JUnit 类依赖 IntegrationTest major55/expected52，原始诊断保留。Jarde 每格编译的是完整选定子类源码，合法文件名 `TestSequentialLoops2$TestCls.java`，不声称其编译了上游 outer。JADX 每格保留完整 outer+child。18个成功格共20个 class/60个 Code，独立双 parser 重读完整方法/字段/access/Code/物理 BCI，身份库存及 access 同原。所有实际运行都使用 `-Xverify:all`、driver-only classpath 和 fresh parentless loader；实际/预期 CodeSource 均是本格 rebuilt 目录。18格各一次错误预期 CodeSource 真实 exit1。

总72运行/576case按 UTF16 返回值、对象同一性、异常类型比较；异常消息只保留 raw。原始基线64case另计。driver、生成源、class 树、依赖、JDK、来源和 CLI 字节闭合不变；3298来源同基线，target不存在。构造器未被 runtime 调用，不扩大行为结论。

首轮[失败采集](../cf11-sequential-loops2-generated-matrix-failed-root-v1/README.md)保持失败，不计成功矩阵。collector v1 未执行；v2修正诊断匹配后实际遇 driver MUTF8 工具解析失败；v3只缩小独立 driver 的解析范围，目标类完整双解析不变。checker 的 Path/表示/参数变量遮蔽错误及 root 草稿失败在[工具历史](../cf11-sequential-loops2-review-tools-root-v1/README.md)保留；最终实际接受脚本 root-v5 SHA256 `fd5a966dc5a295baab0875bd258a195a47c7206a770166e1408ec773b1fb1651`。这些是证据工具问题，不是 Java 恢复缺陷，也未改产品代码或 CI。

归档260成员，SHA256 `3769506416378743e70f363827e9e29ab52fc131e60eba50431b1e04a495fea9`。四新包391成员已逐字复开，见[独立归档复核](../cf11-sequential-loops2-archive-review-root-v1.json)。当前范围已有能力覆盖，无需新增 OpenSpec 实现或机制，不增加整个 CF11/71/612 完成数。下一固定 `TestNestedLoops` 先核当前完整输入/输出，再判断 typed foreach/泛型呈现差距；不沿用历史首拒。
