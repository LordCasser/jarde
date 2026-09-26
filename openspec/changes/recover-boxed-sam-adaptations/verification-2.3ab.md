# Root 候选接缝与方法级数组引用验收

2026-09-26 审读同次 `InvokeDynamic`→LambdaMetafactory→静态实现 MethodHandle 的候选提取。候选保留 CP 原始 owner/name/descriptor 字节，限定同一物理类、`lambda$` 与 `(I)[array` 形状，追加到已有 accessor callee 序列后稳定去重；`read_prepared_named_callees`/`read_named_callees` 仍负责预算内读取，名称与标志不授予投影。真实 javac helper 用紧凑 `iload_0`（0x1a），2.2 的人工 Code 测试只覆盖通用 `iload`（0x15）；现已在原严格三指令、无 handler 的证明内补这个实际形状。

单方法请求的 `BoxedSamProbe.arrayCtor()Ljava/util/function/Function;` 经实际 CLI `recover --policy single-class --release 8` 生成 `return int[]::new;`，本次 CLI SHA-256 `33216c51745b172a71ae70d53ebc64cd5d9986b35b73141b2d96612b12609463`，输入 class SHA-256 `42dec3fe7f959bf4577417bd497cc14b5a55c59b0e6354e8eb65a9be33e8d32b`。同次 callee 报告选中精确的 `lambda$arrayCtor$0(I)[I` 物理方法，Code 读取计入 2 个 `method_bodies`，没有额外 class 选择。方法恢复测试核对 `int[]::new` 来源为 Indy BCI 0；缺 `ClassMembers` 的直接恢复仍写普通 helper-call lambda。

类源码请求在 2.3c 用途/目标类型证明完成前保守写 `(java.lang.Object p0) -> BoxedSamProbe.lambda$arrayCtor$0((java.lang.Integer) p0)` 并保留物理 helper 声明。Root 重跑 `p3_java_recovery` 33/33、`p3_patterns` 54/54、`p3_immediate_functional_receivers` 3/3；后者先发现“数组引用 + helper 声明”的半投影回归，加入类级安全门后重跑通过。真实 Java 8 捕获反例 `Supplier<int[]> array(int n) { return () -> new int[n]; }` 保留零参数 lambda 与 helper，重编通过；数组 Code 相同但长度来自捕获，Plan 本身不再授予 `T[]::new`。`cargo fmt --all -- --check`、`git diff --check`、`openspec validate recover-boxed-sam-adaptations --strict` 通过。

类级投影尚未验收：保留 helper 与 `int[]::new` 会和 javac 合成的 `lambda$arrayCtor$0(int)` 碰撞；只省 helper 时，冻结大样本的 raw `Function` 返回声明又使 `Object` 无法拆箱为 `int`。临时 `ArrayMaker { int[] make(Integer n); }` 控制类不依赖泛型声明：保留 helper 时冲突，省 helper 后 Java 8 重编执行得到 `3`。2.3c 将以这个可写目标为正例、raw `Function` 为拒绝边界，不在此阶段发表半投影。
