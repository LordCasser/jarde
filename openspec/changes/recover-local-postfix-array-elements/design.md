## Context

见 [proposal.md](proposal.md) 和[冻结三方证据](../../evidence/java-syntax-2026-09-25/array-postfix-element/analysis.md)。现有 `ArrayInitializers::prove` 逐层证明同块分配、连续 0..N-1 store、单一 SSA 身份及完整元素依赖；BCI 9 `iload_0` 的旧值在 BCI 10 `iinc 0,1` 后才被 BCI 13 `iastore` 消费，普通表达式依赖扫描无法把独立局部更新纳入第二元素。`ExprKind::PostIncrement` 和 emitter 已有，但当前旧值规则限于字段/数组元素。`slot_name_denotes_the_same_value` 正确拒绝在 iinc 后把旧 SSA 值直接写成局部名。

JADX 本例把第二元素写成旧形参、第三元素写成 `(i + 1) * 2`，五行运行正确，却不恢复后置自增；其 `TestArrayFill2.test2` 标记未完成。直接复用 Jarde 现有 AST/SSA 与证明比引入外部库或照搬 JADX 的局部代数替换更精确。reader/decoder 已识别 `iload`、`iinc` 和 `iastore`，缺口只在源码恢复；Java 8 编译与 JVM 验证运行是离线测试，不改变引擎运行或目标代码执行策略。

## Goals / Non-Goals

**Goals:** 在完整、物理相邻、同槽 `iload; iinc +1; iastore` 的数组元素位置写出一次局部 `++`，后续元素仍读新局部值；保留一维及多维数组初始化已有顺序/来源门槛。

**Non-Goals:** 不尝试通用局部 postfix 表达式恢复、不处理 `iinc -1/+2`、跨块或跨元素更新，不把 JADX 对旧/新值的代数替换扩成局部 SSA 重写 pass，也不放宽数组身份、索引和组件准入。

## Decisions

1. **在现有元素证明中增加精确局部更新证书。** 只对 `int` 局部、真实 `iload slot` 的唯一旧栈值、紧随其后的 `iinc same-slot,+1`、由同一元素 store 唯一消费的形状启用。检查 `iinc` 读的是 `iload` 所读的同一 SSA 局部版本，写出新版本，元素区间无额外入口、使用或 effect；下一元素仍沿普通 SSA 读取新版本。把 load 与 iinc 作为该元素一个不可分的证书，沿用数组分配与 store 的既有预算、handler 集合和物理顺序核验。仅凭邻接 opcode 或文本 `a++` 猜测不足。
2. **复用 `PostIncrement(Local)` 而非新语法节点。** 元素渲染时证书先决定已声明局部名及其精确 `int` 类型，创建带旧 load 与更新 BCI 来源的 `PostIncrement`；不调用会在 iinc 后把旧值误读成当前局部名的普通 `render_value`。`array_initializer_element` 仍对返回的 `int` 作现有组件位置检查。提交后抑制独立的 `iinc` 语句；候选任一步失败则不抑制、不发布部分数组表达式。现有 field/array postfix 的类型与语法保持独立证明。
3. **把物理顺序作为提交条件。** 对正例比较 `new int[]{1,a++,a*2}` 的 Java 8 重编执行，包含 `Integer.MAX_VALUE` 溢出；对同字节数的 verifier-valid `iinc +2` 控制检查继续拒绝。后置自增发生在第三元素之前，也发生在前面两次元素写入之后，来源必须涵盖每个真实 BCI。JADX 的本例五行相等只提供交叉对照，不能代替负例的边界证明。

## Risks / Trade-offs

- **旧栈值与局部现值混淆** → 只由 load 的 SSA 输出与 iinc 的同槽输入配对，构建 `PostIncrement` 时用真实更新节点锚定，不以消费点的当前局部名直接替代旧值。
- **更新被发射两次或完全丢失** → 证书与数组链原子提交，`iinc` 的普通语句抑制及来源归属同步进行，预算/取消路径不得写半个计划。
- **与嵌套数组实施共享 `ArrayInitializers` 接缝** → 串行实施，先复验已验收的多维/一维对照，再增加局部后置自增这一独立形状；不把两项的负例混成一个宽松准入。
- **不同原始源码可能生成相近字节码** → 不承诺识别“原作者写了什么”，只在 Java 后置自增与原物理执行可证明等价时输出该语法。
