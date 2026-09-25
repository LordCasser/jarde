# 嵌套生产者之间的两段副作用

CLI ecab，输入 SHA `f77378eebaa6230967194214ca0ef2ffd3e1a3f9235e3ff92e567b9c98f8677f`。源码 `return take(value());` 精确Code patch为 `value@0; mark@3; take@6; mark@9; ireturn@12`；max_stack/max_locals不变，JVM验证成功。5项覆盖正常、value抛错、第一次mark抛错、take抛错、第二次mark抛错。

原class和JADX完整重编译执行一致。jarde零quote、完整javac成功，但5项全部不同：正常结果6/trace1232变为10/trace2132；异常相同对象的出现顺序同样变化。完整输出与patch在同目录。

根因：prepare_deferred_bindings的candidate_producers过滤把inner直接被outer消费等同于可以内联。外层保存位置之前还有独立mark，必须保留inner保存。现有有界区间事实和Declare/Local足够处理，无需新机制。该反例纳入同一任务的必须恢复正例。
