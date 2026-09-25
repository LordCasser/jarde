# 数组初始化器中的局部后置自增

## 三方冻结结果

[源](ArrayPostfixElement.java)使用 Java 8 `new int[] {1, a++, a * 2}`。[class](ArrayPostfixElement.class)以 `javac --release 8 -g:none -Xlint:-options` 编译；[`javap`](javap.txt)中第二元素是 BCI 9 `iload_0`、BCI 10 `iinc 0,1`、BCI 13 `iastore`，第三元素随后在 BCI 16 重新 `iload_0`。这说明旧值必须先进入数组，局部增量必须发生在第三元素求值前。原 class 的[五组验证运行](original-run.txt)覆盖负数、0、正数与 `Integer.MAX_VALUE` 溢出。

JADX 1.5.6 将第二元素写为 `i`、第三元素写为 `(i + 1) * 2`，没有投影源码的 `a++`；[生成源码](jadx.java)在仅删除工具虚构 package 后以 Java 8 重编并[五行结果](jadx-run.txt)与原 class 一致。本地 JADX 的 [`TestArrayFill2.test2`](/Users/lordcasser/workspace/testzone/jadx/jadx-core/src/test/java/jadx/tests/integration/arrays/TestArrayFill2.java)正是 `a++` 元素语法点，标记 `@NotYetImplemented`。因此这里的缺口是源码结构而非该样本的行为错误；不同于[乱序写入反例](../nested-array-initializer/analysis.md)中 JADX 已改变副作用顺序。

当前源码独立构建的 Jarde CLI 对同一 class 的[完整源码](jarde.java)与[报告](jarde-report.json)在 BCI 1 的数组分配、复制和全部 store 留下引用，BCI 10 的 `iinc` 则独立写为 `arg0 = arg0 + 1;`。完整类因缺 return 而[无法编译](jarde-javac.txt)。`ArrayInitializers::prove` 的同块链已经识别基本形状，但第二元素的 `collect_expression_bcis` 只沿旧值 `iload_0`，区间内 BCI 10 `iinc` 是独立效果，`interval_is_expression` 不能把它当成元素表达式；不能靠放宽区间检查或把 `iinc` 丢掉来恢复。

多维嵌套数组实现通过 root 验收后再读同一 class，[完整报告](post-nested-pre-postfix-report.json)的正文与上述基线逐字相同，确认此缺口没有被嵌套改动顺带修复。

## 架构判断

现有 `ExprKind::PostIncrement` / emitter 已能发射 `target++`，但构建规则目前只证明字段和数组元素的旧值返回；`iinc` 的通常路径输出独立赋值。可新增**局部、精确**的数组元素证明：第二元素必须由同槽 `iload` 的旧栈值提供，紧邻该 load 的真实 `iinc slot, +1` 必须只写该槽，后续消费者及 SSA 不能引用错误版本；把旧值与更新作为一个 `PostIncrement(Local)` 在该元素原物理位置求值，BCI 9/10 共同归属，第三元素读取新局部值。父数组的分配、连续索引、唯一身份、异常集合、预算/来源检查仍逐项保持；拒绝其它自增幅度、不同槽、跨元素移动、额外 use/入口与不能证明 Java 局部名类型的形状。

这应是独立于多维嵌套数组的 OpenSpec；两者共享数组证明代码，需串行改动与单独验收。至今仅冻结正例、JADX 的正确但不够简洁输出，以及 Jarde 的明确缺口；尚未构造负例或实现，不能把设想当成已恢复能力。

已补充一个合法 class 负例：[原 class](ArrayPostfixElement.class) 的唯一 `iload_0; iinc 0,1; iastore` 字节片段把增量常量从 1 改为 2，得到 SHA-256 `7ad6f0722c3092f3f2e0d3094ad81d1f3acdcf22860a72566e07d984e4e4c8bf` 的 [`plus2` class](ArrayPostfixElement-plus2.class)。`java -Xverify:all` [五行](plus2-original-run.txt)正常执行，例如输入 0 为 `[1, 0, 4]`，若错误写成 `a++` 将变成 `[1, 0, 2]`。当前 Jarde [仍拒绝](plus2-jarde.java)；未来实现必须对这一变体继续拒绝或另有等价证明，不能以“邻接 `iinc`”一项放行。
