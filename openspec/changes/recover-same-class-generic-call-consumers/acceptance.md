# GC 同类泛型调用里程碑验收清单

基线 main d158989b，行为CLI `/tmp/jarde-raw-receiver-final-v3-cli` SHA256 `3f75dab495c9753ec2c3f1bd0ac91665ed42f8b877eea65cb7c69a6879448f70`。四腿固定为真 Corretto8u432/OpenJDK23.0.1，各自 debug/no-debug；Jarde source profile 为8，JADX使用参考仓库实际构建的 dev launcher，不用发行版冒充参考源码算法。输入源、class/jar、工具与libs、命令stdout/stderr/exit、实际输出和Probe都保留hash。历史父巡查仍保留。

每个正例都要原/JADX/baseline/candidate完整源码独立重编；JADX失败如实列出，不给候选放宽通过条件。原和候选用独立新classes运行 `-Xverify:all`；显式空classpath/sourcepath，不含原jar。字段/constructor/method/class反射分别比，参数变量以 `getGenericDeclaration()==declaringClass/declaringMethod` 与variable index校验，不能只比较同名字符串或将defpackage包装差异当变量身份。marker、T[]同一数组、多参数位置、Number marker17和异常路径分别执行。外部Probe不属于被反编译目标，不删除目标中失败方法。

| ID | 必备族与控制 | 必须验证的结果 |
| --- | --- | --- |
| GC-01 | EmptySink、既有VoidDirect/FieldSetter、CallRelay、NullCall/void结果消费 | 空体/void/direct call result 的方法API恢复；既有成功不计为新修复，完整重编与marker/调用次数一致 |
| GC-02 | ArrayRelay、NumberBoundRelay、MultiParam、WideRelay(long/double前缀)、TwoClassVariables、ArrayDimensionRelay | 直接T/T[]/维数、bounds、两个不同class formal及真实物理槽位；完整API与行为一致 |
| GC-03 | 至少两层DeepRelay、ReverseDeclarationRelay、NestedCallArgument(`second(first(x))`)、失败incoming旁IndependentLeaf | 声明顺序无关，整链共同恢复；直接调用结果作为下一调用参数有精确AST/SSA消费闭合；不能只恢复首层；失败关联不使独立已证leaf退化 |
| GC-04 | MethodShadow、IndependentCallee、CompatibleIntersectionBinder、合法不兼容SameErasureBinder控制 | method T/class T/callee U归属独立；直接唯一方法formal substitution与bounds正确；同擦除不同binder不得合并 |
| GC-05 | TypedReceiverRelay(本类C<T>参数)、RawOwnReceiver、ReboundOwnReceiver；外部List raw/mutated exploratory作额外拒绝历史 | 实际receiver类型决定成员选择；正例本类参数代换完整恢复；raw/未知result不冒充T；重绑定不借用旧声明 |
| GC-06 | 前片冻结CallHold/ExceptionHold各四输入、CatchCallMarker(正常+实际throw路径) | constructor(T)的class binder、Object()初始化、调用结果字段写顺序与handler保持；不能仅因EH拒绝已可闭合形，字段独立结果照实记录 |
| GC-07 | 已冻结BoundOverload、SameNameOverload(T/String)、PlainUpperBoundOverload | 实际源集合/类型选回原物理目标；必要Number上转型无新增运行时检查；唯一选择不添无用cast；相关generic overload头独立核对 |
| GC-08 | UnknownIncoming(合法源码含未保留unchecked T cast)、CycleRelay有限执行控制、MethodHandleUse、不完整site/多use、varargs/bridge/继承未知 | 关联拒绝/回退可靠、整类不新增编译失败；rawsource保留效果/来源；未知cast不猜；不合法源preflight不计控制输入 |
| GC-09 | 既有字段23族、构造20×4=80、raw16×4=64 | 保持各自baseline已验收完整编译/行为/反射指标；已知SCGB/constructor8失败如实保留，不删例增成功率 |
| GC-10 | collector/type-proof/staging/output预算与取消；fmt/clippy/两固定seed/ignored/strict spec/真JDK25 CI | 停止不变普通拒绝、没有partialheader/body/facts；明列计费命令和logs；只以最新main HEAD CI交付 |

兼容intersection的caller class T确有callee所需全部Number/Runnable bound，可以唯一映射U→T；这不代表变量身份相同。合法不兼容控制应以null或Object+原源码uncheckedcast构造真实class，而不是把原javac已拒源伪装成恢复控制。类型推断cycle可靠拒绝不计API正例。原输入与JADX重编出错属于数据，不靠改源/删方法凑一致；调试harness修正必须留下前轮和说明。

任务1.1冻结时补充每族源/物理调用/预期Probe与四腿索引，root再独立核对。此文件是必须兑现的验收范围，不是已通过证据；不将行号、脚本存在或subagent自述当验收结论。

实施审阅补充：NestedCallArgument 是 design§1 已承诺的直接调用参数消费位，不是一般 alias/phi 传播。补充四腿独立冻结与原/JADX/baseline完整对照，不改已有35族140输入的frozen-inputs-v1和历史统计。accepted-cli-v4的140输入结果已独立核对，但执行脚本只记录了mutable源路径与hash，后续candidate标签修改使历史脚本字节不再可核；保留该缺口，改用从run开始保存实际源码快照的accepted-cli-v5重新固定同一基线，不覆盖v4。
