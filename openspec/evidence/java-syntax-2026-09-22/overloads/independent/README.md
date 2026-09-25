# 主代理独立调用参数审计

`InvocationAudit` 整类经 javac 23.0.1 `--release 8 -g:none` 编译，debug jarde CLI 实际输出整类未经正文替换直接再编译。原类和恢复类使用相同的 PrologueParent、AuditEffects、InvocationAuditRunner helper。全部 class 在 /tmp/jarde-invocation-independent，永久 fixture 未增加。

修前 jarde 可以编译，但12行中3行错误：super 构造器1→2，this构造器1→2，左右效果均一次且顺序12的 ordered 调用41→42。同型Object/String、已有显式String cast/null/CCE保持。原始、生成源码及执行输出均在本目录。

jadx 1.5.6 输出带自动 `package defpackage;`。为保持生成的完整 Probe 源码逐字不变，只在外部 helper/runner 副本上加相同package，再编译并以 defpackage.InvocationAuditRunner 执行；最初缺失此包装的编译失败也保留。最终jadx在super构造器仍为2，this与ordered正确。

with-bare-throw保留首次试验的裸athrow独立缺口。主输入改为调用外部 effects helper 后重新编译原class，不通过删掉生成成员来绕开失败。这里验证参数目标、左右求值顺序、异常短路和new/this/super入口。2026-09-23主代理重建debug CLI后，after-jarde.java.txt原样整类编译，12行运行结果全部与原class相同，修前3个错值全部关闭。随后补充self-argument中的this→Object场景，发现隐式this缺presented类型仍被拒绝，已纳入同一事实入口修正；因此尚未宣称全部边界验收完成。
