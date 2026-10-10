# CF-07：完整类循环回跳来源基线

复用2026-09-27 CF-07 的完整 `cf07.LoopCases` 与 Runner，不增 fixture。原样输入包含短路 while、计数循环及循环内提前返回；JADX 测试关联见原 [report](../../java-syntax-2026-09-27/cf07-basic-loops/report.md)。

root 全文审查后实际执行 `prepare-baseline-luna-v2.py`：固定 JDK8/23、JADX1.5.6 default/rename-none、旧冻结乘法 Jarde CLI default/all，共29命令、118闭合文件、10个完整类编译运行腿。实际所有编译/运行exit0，各重建程序的 exit/stdout/stderr 与同JDK原始oracle逐字相同。基线在 `baseline-root-v2`，独立接受以 `verification-root.md` 所列 root 实跑结果为准。

每腿使用 fresh classes 和 empty classpath/sourcepath，运行 `-Xverify:all`；目标生成源码不改正文、不删成员，Runner 包不需修改。JADX 输入jar只有javac23目标class，不含Runner或原class回补。来源检查绑定BLAKE3物理class、准确四method及flags、javap BCI和UTF-8源码跨度。

四Jarde profile一致的真实来源缺口：

- `andWhile(Z)I` 的 goto@15→2。
- `counted(II)I` 的 goto@20→27（if汇合）与goto@30→6（循环回跳）。
- `lastIndexOf([IIII)I` 的 goto@25→5。

整类运行成功不代表来源完整；这些是旧CLI基线，不是新patch接受。当前来源候选只处理已证普通while body末尾Straight的单个隐式latch。if汇合和非Straight末尾需要各自沿现有区域来源接缝分析，不混入当前实现，也不因此计CF-07整单元追平。候选应用后还须以fresh CLI重放该完整类并核正文/运行及精确来源变化。

脚本通过固定SHA的AST helper-only定义复用已有 one-arm-loop-controls 录制、重编及javap解析，不执行旧脚本main。v1静态审查发现helper初始化顺序错误而未执行；root将修正版单独保存v2并精确恢复v1 SHA，说明在 `private-review-root-v1.json`。所有raw保持原字节，inventory不包含其自身。

重放入口（拒绝覆盖现有证据，不能再次写同一输出）：

```sh
uv run --offline --with blake3 python -B openspec/evidence/java-syntax-2026-10-10/cf07-loop-latch-baseline/prepare-baseline-luna-v2.py
```
