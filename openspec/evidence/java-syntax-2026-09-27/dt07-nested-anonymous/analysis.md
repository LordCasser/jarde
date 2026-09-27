# DT-07：双层匿名接口表达式审计

## 固定 JADX 依据

冻结 inventory 的 DT-07 指向 JADX `inner/TestNestedAnonymousClass.java` 和 `inner/TestAnonymousClass12.java`，生产入口为 `ProcessAnonymous`、`AnonymousClassVisitor`。本地 JADX checkout 在审计时为固定提交 `2fb1b16386941660fda07e9017285aec40fcb37f`，工作树干净；两份测试原文副本为 [TestNestedAnonymousClass.java](reference/TestNestedAnonymousClass.java) 和 [TestAnonymousClass12.java](reference/TestAnonymousClass12.java)，重放脚本核对它们与固定 checkout 逐字一致。

`TestNestedAnonymousClass.test()` 断言 `Callable<Runnable>` 匿名体的 `call()` 返回另一匿名 `Runnable`；它证明 JADX 能写出两层语法，但测试没有完整项目重编/运行断言。`TestAnonymousClass12.test()` 断言一个匿名 `BasicAbstract` 的方法内再创建匿名 `BasicAbstract`，并修改外围 `TestCls` 的 `inner` 字段。它还包含匿名父类、外层实例捕获和跨层字段访问，不能直接当作本次首片的实现范围。两者的测试均不是 Jarde 的语义等价证据。

固定 `ProcessAnonymous.checkUsage` 按构造器 `getUseIn().size()` 判断构造器是否只使用一次；这个数是使用方法数，不是完整分配 BCI 数。`mergeAnonymousDeps` 另外从叶节点向顶层建立 codegen dependency。`AnonymousClassVisitor` 按匿名子类构造器参数将合成字段映射回创建参数，并删除相关构造指令。可借鉴其依赖顺序与隐藏捕获参数的处理，但不能把 `useIn` 计数当成唯一性证明；同方法多处分配和范围外类型身份都应由 Jarde 的物理扫描/XRef 拒绝。

## 冻结隔离夹具与重放

[input/p/](input/p/) 是基于 `Callable<Runnable>` 嵌套返回形状缩小出的 Java 8 输入：根方法直接返回唯一匿名 `Factory`，其 `make()` 直接返回唯一匿名 `Action`；两接口均是顶级类型，避免把成员类型声明混入 DT-07。`Action.run()` 只递增根类静态计数器。共同的 Runner 在 `-Xverify:all` 下打印 `1`。

[replay.py](replay.py) 从原始输入编译原 class 和 jar，调用固定 JADX，再分别抽取 JADX 与 Jarde 的所有完整物理类源码；三个来源都与同一 Runner 组成完整 Java 8 编译单元。基线重放输出保存在 [baseline/](baseline/)，包括完整源码、原始 `javap -p -c -v`、Java/JADX 版本、编译日志、运行日志与 SHA-256。独立第二次重放的 `summary.json` 与冻结摘要逐字一致。

固定基线结果：

| 来源 | `javac --release 8 -g:none` | `java -Xverify:all` | 输出/诊断 |
|---|---:|---:|---|
| 原始源码 | 0 | 0 | `1` |
| 固定 JADX | 0 | 0 | `1` |
| Jarde 完整物理源码 | 1 | 未运行 | 构造器捕获字段赋值排在 `super()` 前 |

JADX 的完整根源码在 [baseline/jadx/source/p/Nested.java](baseline/jadx/source/p/Nested.java) 中嵌入 `new Factory() { ... return new Action() { ... } }`。Jarde `class-source p/Nested` 返回 exit 4，诊断 `anonymous_interface_child_additional_use` 指向物理 `Nested$1$1` 对其外层物理类 `Nested$1` 的字段描述符引用；根仍输出 `return new p.Nested$1();`。Jarde 分别保留两个物理类和完整方法正文。

原 class 的 `Nested$1$1` 具有精确 `EnclosingMethod: p.Nested$1.make`。`javap` 显示它只有一个 `ACC_FINAL, ACC_SYNTHETIC` 的 `this$0:Lp/Nested$1;` 字段，构造器按 `aload_0; aload_1; putfield this$0; aload_0; invokespecial Object.<init>; return` 写入捕获后才调用父构造器，`run()` 只读写 `Nested.trace`，没有读取 `this$0`。Jarde 将同一字节码直写成物理构造器，Java 8 源码编译器不能接受构造器调用前的实例字段写入。这里首先是**类级匿名关系及其构造器投影没有闭合**造成的源码编译差距，不是 classfile 解码或原程序行为差异。只有编译原/JADX 源码成功，Jarde 侧不声称运行等价。

## 可实现的首片与边界

当前 `project_class_source_anonymous_interface` 接受一个根方法的唯一直接返回分配，绑定一个物理 child，并证明其接口合同、方法 AST、构造器和 owner XRef。`prove_anonymous_owner_xrefs` 允许创建/构造及声明关系；本例的孙 child 捕获字段描述符引用外层 `Nested$1`，是一个尚未获证的跨物理匿名节点关系，因此被拒绝。若只放宽该 XRef 而不闭合下一层，就会把物理匿名类名/构造器暴露到生成源码中，并没有获得完整投影。

可复用的局部接缝已经存在：`ClassSourceMethodAst` 保存同次方法 AST，`AnonymousAllocationScan` 保存物理分配 BCI/目标/构造调用，`emit_class_source_anonymous_return` 可在指定 AST 节点写匿名表达式；`NestedClassSourceText` 与 `MemberFamilyMethodText` 提供类级放置和范围翻译。窄扩展应为准确两节点、单向嵌套的匿名接口关系构造一个小证书，并以 AST 发射和结构化范围翻译一次性组装完整 root 源码，而非递归重新恢复、以 `$` 命名推断关系或全局替换文本。

首片仅纳入本夹具所证明的形状：根 `return` 唯一匿名接口 child；其一个已选实现方法中只有一处分配，并直接返回另一个匿名接口 child；typed `InnerClasses`、精确 `EnclosingMethod`、两个接口 descriptor 和完整的同次 AST/Code 一致；全选定输入范围的 XRef 仅包含这条两节点创建链与其结构声明；内层 child 的唯一 synthetic-final immediate-parent capture 字段只在构造器中写一次且没有正文读取。任一身份引用、第二分配、其它捕获/字段/构造效果、方法体不完整、范围扫描不全、共同预算停止或并存源码族投影均原子拒绝并保留物理源码。

`TestAnonymousClass12` 的匿名抽象父类、外围实例字段写入和外层捕获读不满足上述接口/无捕获读取边界；它只作为后续组合验收候选。更深嵌套、多子节点、非直接返回分配、类父类匿名体及捕获局部值同样不由本切片推断。任何负例都不得把“保留物理源码”计作匿名表达式通过。
