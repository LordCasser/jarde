# 函数表达式作为构造参数巡查（2026-10-08）

基线主线 `ddfd05f8d8a2e5f3537f5c3b47a4aefea25ddb4c`；root 自编译 jarde-cli。真实 Corretto javac 1.8.0_432 与 OpenJDK23.0.1 --release8，均 -g:none。JADX CLI1.5.6；算法参考本地 `/Users/lordcasser/workspace/testzone/jadx` 的 CustomLambdaCall、InvokeCustomBuilder、InsnGen；所读 TestLambdaConstructor 只覆盖构造器方法引用，不是构造实参 lambda，未将其当作本族覆盖证据。

## 输入与基线

source/FunctionalConstructors.java、IntBox.java、Driver.java。每条编译腿保存原 class/jar、source.stdout、baseline 完整呈现。9 个构造方法均拒绝，init/new@1 在参数区间遇到 InvokeDynamic 时不准入。legacy LG 对照在 baseline-LG.txt/json、javap-LG.txt；历史“未初始化 local”诊断已变为整方法 explanation-only，不能照抄旧诊断。

## root 门控实验

临时只把 `Some(Operation::Invoke(_)) if argument_dependencies.contains(...)` 扩展为 Invoke|InvokeDynamic，其他路径不变。gating-*.text 是完整输出；gating/recompiled 保存剥离注释的整类与 javac.log/run.stdout。两腿均编译并 -Xverify:all 运行，与 source.stdout 逐字相同（8 行，创建=1，调用=13）。实验改动已撤销，不是正式实现；正式片须补 handler 覆盖与拒绝测试。

JADX 整类在 jadx/sources/defpackage，未改函数 body。JADX 会把 default-package 类改到 defpackage，外部 Driver 仅添加相同 package 后进行完整编译/执行，两腿同样通过。第一次未适配 harness package 的 javac 报错是比较脚本问题，已修正，不登记为 JADX 缺陷。

## 结果口径

门控仅证明已存在的函数恢复与构造类型适配可以组合。PriorityQueue/部分返回 generic Signature 仍有保守拒绝记录；输出可编译并不意味着泛型语法已覆盖。不可空绑定接收者以及未知 bootstrap 必须单独保留拒绝验证。不复制 JADX 代码、不增加 owner 白名单/新 IR/planner。

正式任务：`openspec/changes/recover-functional-constructor-arguments/`。基线 json 保留原始 evidence；recompiled 类仅为测试产物，不入库。
