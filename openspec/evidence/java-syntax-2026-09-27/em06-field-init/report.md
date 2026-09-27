# EM-06：字段初始化位置与顺序

固定 JADX `2fb1b16386941660fda07e9017285aec40fcb37f` 的五项测试及 `ExtractFieldInit`、`ClassGen` 哈希由 [replay.py](replay.py) 核验。[首片输入](input/em06/FieldOrder.java)把 `TestFieldInitOrderStatic` 的连续静态依赖和 `TestFieldInitNegative` 的实例赋值禁提前合在一个顶级类；其余测试的继承字段、异常区及数组初始化留在各自边界。本次无 debug 信息编译，避免局部名影响初始化证明。

[baseline/summary.json](baseline/summary.json) 记录原 class、JADX、Jarde 三份**完整类源码**均以 `javac --release 8` 重编，`java -Xverify:all` 运行逐字相同：`a:ab:abc:abc`、`sb2`。两种反编译输出都把依赖 `state` 的 `field = initField()` 留在构造器中，没有错误提前。源码质量差异集中在静态链：JADX 将 `trace` 与 `a/b/c/result` 按依赖顺序写成字段声明初值；Jarde 保留了相同顺序的完整 `static` 块，运行语义正确，但尚未追平该源级形式。

JADX 的 `ExtractFieldInit.collectFieldsInit` 在单路径上收集字段写入，`filterFieldsInit` 排除重复写和不可移动 RHS，`fixFieldsOrder` 处理字段读取依赖，然后才删除原指令；异常区和多个构造器另有检查。Jarde 在同轮 `<clinit>` 恢复中已产生有序 `ClassInitializerStep::FieldWrite`、字段读身份和 RHS AST；`src/facade.rs` 已有接口字段初值的结构证明、依赖阶段检查及原子投影，`src/class_source.rs` 的 writer 也能按证明顺序写声明并省去 `<clinit>`。当前入口只对普通接口启用，普通类直接给出 `NotApplicable`。因此首片宜复用现有同轮候选、字段/BCI 身份、来源和最终 writer，扩展到**一组完整的普通类静态运行时字段**；无需新增 JVM IR 或第二套全局初始化提取器。实例构造器初值、try/catch 和多构造器差异不得随此次扩展混入。

首片需尤其证明整组写入覆盖所有目标静态字段、没有穿插额外顶层效果、每个 RHS 精确消费字段读且表达式求值顺序不变。只要有缺失/重复字段、ConstantValue 阶段冲突、前向读或不完整方法、异常边、预算停止，就保留原 `static` 块。JADX 的 `canReorder` 是参考算法，不作为 Jarde 的证明证书；独立完整源码重编和运行是准入门槛。
