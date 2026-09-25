# 普通类 final 写入与接口初始化的分界

`run_audit.py` 用 javac 23.0.1 的 `--release 8 -g:none` 生成五个 source-only 类；不增加永久 class。每个原 class 和实际 JADX 1.5.6 完整输出均经 `java -Xverify:all` 执行。JADX 的生成包和正文保持原样，仅为 source-only helper/runner 加相同包；六次输出逐字比较相同。当前 debug jarde 的完整输出直接交给 javac，未删改方法或赋值。

| 类 | 原 class = JADX | 当前 jarde javac |
| --- | --- | --- |
| FinalOrder | `12:2` | 两个 final 限定赋值错误 |
| FinalBranch | `9:0`、`7:0` | 两分支的 final 限定赋值错误 |
| FinalLocalCollision | `22:2` | 两个 final 限定赋值错误；局部已名为 local0 |
| FinalSelfRead | `3:1` | VALUE 限定赋值错误 |
| FinalInterface | `1:1` | 字段缺声明初始化，接口禁止 static 块 |

普通类恢复依赖的字段身份、值、区域和执行顺序已经存在。最小闭环需要 FieldAssign 能表达简单字段名，以及 NameTable 避开必须裸写的当前字段；无需新增恢复 pass、resolver 或搬移初始化器。只去掉 `FinalLocalCollision.` 会让赋值绑定到恢复局部，并使 final 字段未初始化，所以必须处理名称约束。

接口不能用相同修复关闭：它必须在字段声明处初始化；这涉及 class-source 的声明/初始化表达式归属，暂记录为独立债务。本目录最后版已将接口的 result 改为普通调用，去掉原审计里额外的 0/1 汇合干扰；原样 jarde 的 result 可恢复，javac 两条错误仍明确来自声明与 static 块。

实现规划见 [recover-final-static-field-writes](../../../../changes/recover-final-static-field-writes/proposal.md)。原始各类源码、完整输出、javac/运行日志和 javap 一并保留；工作 class 位于 `/tmp/jarde-final-initialization-expanded/`。`summary.json` 记录每个编译退出码，不能把当前失败误报为已修复。
