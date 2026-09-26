# 2.16 当前基线：switch 多臂前向汇合

复用已提交的 `tests/fixtures/p3-join-ownership/Join.java` 与 `v8/Join.class`；源码 SHA-256 `848e03648bbf7f929739b0b13b864dffe77d8ebe99f68d17e1437073102bdd2a`，class SHA-256 `458204d72c0298ab4b2e84ce6655d34d2ee131919b8f032e02f888febb988bf1`。Root 以当前 `javac --release 8 -g:none` 重编，class SHA 完全相同。

`mixedExits(I)I` 的真实控制流：`case 1` BCI 28–31 写 10 并 `goto 40`；`case 2` BCI 34–36 直接返回 20；default BCI 37–39 写 30 后落入 BCI 40–41 的 `iload_1; ireturn`。原 class 在输入 0/1/2/3 下输出 `30/10/20/30`。安装的 JADX 1.5.6 把 BCI 40 的返回写在 switch 之后，Java 8 重编并用 `-Xverify:all` 执行，同样输出 `30/10/20/30`。

主分支 `252a0f49` 构建的 Jarde class-source 对 `mixedExits` 整方法给出 explanation-only，诊断为「two switch arms of the block at BCI 0 claim the same block, so one case falls through into another case's code」，引用 BCI 0/1/2/3/28/30/31/34/36/37/39/40/41。当前重叠所有权检查避免了旧批次记录的“default 路径文本缺返回”错义，但仍未恢复这个合法 Java 8 结构。任务 2.16 的终态仍是 `case 1`/default 在汇合 BCI 40 前停止，并由 switch 后续语句唯一认领该返回；不得仅删除 overlap 守卫。

本次只更新基线与实现边界，不把 JADX 的文本当成证明。后续实现须以 decoded `goto`/fall-through、所有 case 的 exit 和 BCI 40 的共同前向入边证明 join；`case 2` 的终止 return 不应被强制经过 join。边界负例应包括 case 臂跳入另一臂、额外入边、回边和有独立效果的汇合前缀。需要同时重验旧 `p3_forward_join` 的共享 if 后继及 `p3_switch_fallthrough`，保证结构块只写一次且无遗漏来源。
