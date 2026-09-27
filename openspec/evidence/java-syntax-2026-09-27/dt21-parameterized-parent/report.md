# DT-21 首片：直接参数化父类声明

[Child.java](Child.java) 只包含 `Parent<T>` 与 `Child extends Parent<String>`；[Runner.java](Runner.java) 只观察 `Child.class.getGenericSuperclass()`。固定 [replay.py](replay.py) 以 Java 8 `-g:none` 编译原始物理类，交给固定 JADX `2fb1b16386941660fda07e9017285aec40fcb37f` 与 Jarde，再让三套完整源码各自以 Java 8 重编并在 `-Xverify:all` 下运行。三者均逐字输出 `dt21parent.Parent<java.lang.String>`。Jarde 输出的完整 `Child` 头是 `Child extends dt21parent.Parent<java.lang.String>`；`Parent<T>` 的既有类头投影保持不变。原始 `Signature`、物理父名、解析到的父定义身份及父类自身 Signature 都参与证明，来源仍由物理类报告承载。

replay 还生成并检查拒绝控制：父类 Signature 缺失（按非泛型父处理）、父类有两个类型参数、child Signature 擦除到另一个父名、物理父类缺失、同名父定义重复、child Signature 畸形、实现额外接口，以及 child 本身没有 Signature。所有控制均保留 raw `extends Parent`；有签名的拒绝案例还报告 Signature 未投影，无签名案例不制造拒绝。它们是 classfile/projection 控制，不作为三方源码等价的正向证据；完整原/JADX/Jarde 重编与反射结果才是正向证据。结果记录在 [results.json](outputs/results.json)，固定类文件供 façade 原子/物理身份测试使用。

预算与取消由 façade 单元测试覆盖：父定义读取超过 `class_headers` 限额时，完整物理 child 声明仍可查，类头保持 raw，执行报告标记未完成；在 resolver seam 注入取消时，`project_generic_signature` 原子返回停止且不替换 raw 头。缺失父类控制也保留物理 Child 的 class-bytes 来源。无自有类型参数的无参 raw 类仍不进入该投影分支。多层变量替换、继承成员和 bridge 仍属于相邻独立切片。
