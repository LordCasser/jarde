## 1. 冻结形状与三方基线

- [x] 1.1 自写 Java 8 fixture：`Supplier<Integer>`/`Function<String,Integer>`/`Function<Integer,int[]>` 站点各一、串联两站点的方法、`int[]::new` 使用点；javac --release 8 编译冻结 SHA；记录擦除 SAM/instantiated/impl 三段类型、合成分配方法的完整 Code、原 class 执行输出、jadx 1.5.6 的数组 lambda 展开与 jarde 修前整方法引用。[三方证据与 Root 重放](../../evidence/java-syntax-2026-09-26/boxed-sam-adaptations/README.md)

## 2. 配对证明与呈现

- [x] 2.1 将提前的栈形状门限于精确捕获，`lambda.rs` 逐槽证明 SAM→instantiated→impl 的参数与反向返回转换（含装箱/拆箱）；arity/错误类型对/静态性负例保持拒绝，现有 Object 与捕获路径不退化。[Root 验收与既存预算测试债务](verification-2.1.md)
- [x] 2.2 从现有同类 `ClassMembers` 精确选定合成 impl，证明完整无 handler 的参数 load→目标数组分配→return、数组类型与长度参数；名称/flag-only、额外效果、错误类型、缺成员事实控制拒绝。[Root 验收](verification-2.2.md)
- [x] 2.3a 从同次 IR 的 LambdaMetafactory BSM 精确提取同类合成 impl MethodHandle 候选，追加到既有按需 callee 读取并复用已准备的类、预算和物理身份；实际 class-source/method-only 站点能把 helper Code 交给 2.2 证明，名称/flag-only 与缺失事实仍不能触发数组投影。[Root 集成验收](verification-2.3ab.md)
- [x] 2.3b Builder 仅对已证数组 impl 且零捕获、唯一 SAM 长度参数的站点发 `T[]::new`；捕获长度的 `Supplier<int[]>` 不得误投影，其它装箱 SAM 使用点恢复含箭头调用的语句，来源 BCI 锚点与合成方法物理报告保留。真实 fixture 的方法级文本与类级保守回退均验证，不以单元构造的 ClassMembers 代替集成证明。[Root 集成验收](verification-2.3ab.md)
- [ ] 2.3c 对同名 helper 做类级原子投影：从同次 IR/BSM/方法表普查全部同物理用途，以私有 typed AST sidecar 重发每个已证站点，预付费后同步省略 helper；JSON/方法级报告不变。其它直接调用/句柄、raw Function 目标、未分析方法、缺覆盖、预算/取消时保留普通合成调用与声明。用真正同名的 `arrayCtor`/`lambda$arrayCtor$0`、非泛型 `ArrayMaker.make(Integer)` 最小正例 Java 8 重编，并保留大 BoxedSamProbe 的 raw 目标负例。JADX 的精确句柄解析可借，立即 `DONT_GENERATE` 的无全用途省略不能照搬。

## 3. 对照与门禁

- [ ] 3.1 恢复文本 Java 8 重编译执行：边界 Integer 值、null 拆箱、负数组长度、正常路径与原 class 一致；jadx 的数组 lambda 展开与 Jarde 的已证数组构造引用对照记录。
- [ ] 3.2 复跑 lambda、functional-receiver、execution_comparison 回归；`cargo fmt`、`cargo clippy -p jarde-java -p jarde --all-targets -- -D warnings`、`openspec validate recover-boxed-sam-adaptations --strict`；golden/语料计数若变重录并说明。
