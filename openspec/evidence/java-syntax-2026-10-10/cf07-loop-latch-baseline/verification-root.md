# CF-07 来源基线：root 独立验收

root 实际执行 collector v2（SHA `87c693a53f31898374d5c963c6fbe14b550db20439e0409486b9d191a00cec8f`），并在全文审查及静态纠正后实际执行 **verify-baseline-root-v4.py**，退出0。接受文件为 **independent-verification-root-v3.json**，该脚本SHA `85b2a43a57b49188f43f52ca24ca4915eed7a7931d7a138672712e87199e2d51`。输出在 baseline inventory 外，未修改原raw。

接受范围是旧冻结乘法 CLI 的完整 CF-07 类基线：118文件闭合、29条命令成功、原2/JADX4/Jarde4共10个完整类重编运行腿；exit/stdout/stderr 与各自同JDK原oracle逐字相同，双JDK原oracle也一致。完整源码不改正文、不删成员，Runner原样；fresh classes只含cf07/LoopCases与Runner，empty CP/SP与-Xverify:all参数逐字核验。目标JAR只有javac23目标class，不含Runner或原class回补。

两份javap独立解析四方法及准确flags/BCI。四Jarde profile逐一绑定原class实际BLAKE3/length/snapshot/base、method身份、全部primary与derived的物理BCI、非空report.text UTF-8跨度；完整生成源等JSON text，default/all整类与逐方法正文/map恒同。没有以整类文本长度来代替method跨度边界。

仍有准确来源缺口：andWhile goto@15→2；counted goto@20→27、@30→6；lastIndexOf goto@25→5。接受的是完整类行为与这些物理观察，**未接受全部BCI覆盖或新候选**。if汇合与非Straight末尾来源债务分开；当前preserve-proved-loop-latch-origins只处理已有证明内的普通while末尾Straight隐式latch，应用后必须用fresh CLI再核完整类及来源精确变化。

历史v1 collector初始化顺序错误只经静态拒绝，未运行；修正版单独保存v2并按原SHA恢复v1。独立verifier v2/v3经root静态拒绝而未执行，v4收紧实际schema/参数、class集合、primary+derived、源码与manifest/input pins后首次实跑成功。具体记录 private-review-root-v1.json，不伪造旧版动态失败。
