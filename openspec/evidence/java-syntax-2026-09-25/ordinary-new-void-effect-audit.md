# 普通 `new` 中独立 void 效果的已证边界

这项代码审计现已由 [verifier-valid 三方执行反例](ordinary-new-void-effect/analysis.md)证实为正确性缺陷。`init::verify` 在普通 `new/dup/…/<init>` 的区间内允许 `Invoke`，条件是 `produces_a_read_value`；该辅助函数对没有栈写入的调用直接返回 true，对有栈结果的调用也只要求同块任意后续读取。`new_expr` 则只渲染物理构造器实参，站点自身只认领分配、复制及构造器调用。区间中不属于任何实参的独立调用会被写到 `new` 表达式之前，改变类初始化顺序。

受控 Java 8 class 的 BCI 为 `0 new Target; 3 dup; 4 invokestatic Side.effect()V; 7 iconst_1; 8 invokespecial Target.<init>(I)V; 11 areturn`。原 class 经 `-Xverify:all` 输出 `CST`；JADX 1.5.6 与当前 Jarde 都输出 `Side.effect(); return new Target(1);`，各自重编运行均输出 `SCT`。Jarde 仍把 BCI 0 报告为 `presented=true`、`quality=structured`。普通 javac 源不会自然生成这条独立中间调用；用 `new Target(sideReturningInt())` 替代不是负例，因为调用结果确属构造实参。

普通 `new@1` 已由[独立 OpenSpec](../../changes/refuse-unconsumed-construction-invokes/design.md)收紧计划：从构造实参沿 SSA 证明调用归属，缺证即拒绝，不能只改零栈结果分支。成员内部类的[另一份 OpenSpec](../../changes/recover-proved-member-inner-construction/design.md)保留自己的关系和时序证明，两者不混为一个机制。
