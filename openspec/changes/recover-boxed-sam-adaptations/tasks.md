## 1. 冻结形状与三方基线

- [x] 1.1 自写 Java 8 fixture：`Supplier<Integer>`/`Function<String,Integer>`/`Function<Integer,int[]>` 站点各一、串联两站点的方法、`int[]::new` 使用点；javac --release 8 编译冻结 SHA；记录擦除 SAM/instantiated/impl 三段类型、合成分配方法的完整 Code、原 class 执行输出、jadx 1.5.6 的数组 lambda 展开与 jarde 修前整方法引用。[三方证据与 Root 重放](../../evidence/java-syntax-2026-09-26/boxed-sam-adaptations/README.md)

## 2. 配对证明与呈现

- [ ] 2.1 将提前的栈形状门限于精确捕获，`lambda.rs` 逐槽证明 SAM→instantiated→impl 的参数与反向返回转换（含装箱/拆箱）；arity/错误类型对/静态性负例保持拒绝，现有 Object 与捕获路径不退化。
- [ ] 2.2 从现有同类 `ClassMembers` 精确选定合成 impl，证明完整无 handler 的参数 load→目标数组分配→return、数组类型与长度参数；名称/flag-only、额外效果、错误类型、缺成员事实控制拒绝。
- [ ] 2.3 Builder 对已证数组 impl 发 `T[]::new`；其它装箱 SAM 使用点恢复含箭头调用的语句，来源 BCI 锚点与合成方法物理报告保留；重编译检查同名合成方法无冲突。

## 3. 对照与门禁

- [ ] 3.1 恢复文本 Java 8 重编译执行：边界 Integer 值、null 拆箱、负数组长度、正常路径与原 class 一致；jadx 的数组 lambda 展开与 Jarde 的已证数组构造引用对照记录。
- [ ] 3.2 复跑 lambda、functional-receiver、execution_comparison 回归；`cargo fmt`、`cargo clippy -p jarde-java -p jarde --all-targets -- -D warnings`、`openspec validate recover-boxed-sam-adaptations --strict`；golden/语料计数若变重录并说明。
