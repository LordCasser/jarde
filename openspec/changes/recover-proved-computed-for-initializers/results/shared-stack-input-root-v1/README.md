# 共享算术栈值：准确拒绝边界

root v4 实际20命令，两个JDK原class通过-Xverify:all及四行精确oracle；fresh JADX 1.5.6 的完整源在双JDK重编运行同原。Javap固定limit@0/isub@4/dup@5/putstatic@6/istore@9，原class不是手造IR。

Jarde default/all共四完整类原样重编均exit1（缺少返回语句），没有Jarde运行成功声明。四报告find均Fallback/ExplanationOnly，copy@5没有已证明局部赋值，未发布新for；所有现有origin精确方法身份/物理BCI/UTF8正确，全部方法正文/map/default-all一致。root-refusal verifier v2实际exit0只接受可靠拒绝，不把general checker的failed_or_incomplete改写成运行通过。本class的20个物理BCI全部覆盖；其他旧controls的quoted fallback映射债务不混入当前片。

源级同一个结果有两个消费不能冒称SSA同一个ValueId有两个uses；SSA copy模型还需永久真实IR核验，Builder的准确拒绝是已观察事实。

preserved-invalidated-capture-v3是真实20命令之后runner被交付agent改写导致self-pin失败，不能接受；root独占0555 v4保存后fresh重跑，完整前后pins通过。refusal verifier v1只因javac中文诊断与错误英文期待失败，v2准确支持原raw中文而不修改目标源或错误退出。archive-inventory绑定复制字节，receipt原路径保留。
