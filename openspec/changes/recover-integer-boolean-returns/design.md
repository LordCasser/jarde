## Context

见 `proposal.md`。已冻结的 `BooleanReturnBoundaries.class` 是 Java 8 输入，SHA-256 `336f8f291987fa9fdc583e62ffb99b522d37d19ffef8db6cfd64e5d77db973a4`：只将 `integerAsBoolean` 的描述符 `(I)I` 改成 `(I)Z`，Code 仍是 `iload_0; ireturn`。`java -Xverify:all` 对 0/1/2/3/-1 返回 false/true/false/true/true，Jarde 当前对其保留可定位拒绝。本地 JADX 1.5.6 的 `InsnGen` RETURN 分支直接打印已有参数，实际输出 `return i;`，完整源码无法编译；同一类中的 `(Z)B/C/S` 又被其三元式折叠，在合法 raw 2 调用上将原 JVM 的 2 错写成 1。JADX 可供定位转换应放在消费端的思路，不能作为此边界的语义 oracle。

`build.rs::return_expr` 已按真实求值 BCI 渲染普通、同步及 switch 臂值；`adapt_return` 再以实际 `ireturn` BCI 和本方法描述符适配 B/C/S。已证明的字段自增快捷路径直接把一次更新表达式送入 `adapt_return`。另一个已完成的 `field_value` 对 Z 字段写入的整数值用现有 `BinaryOp::Remainder`/`NotEqual` 构成 `value % 2 != 0`。这些都是现成的值、类型、来源和预算接缝，不需要新的事实表、AST 类型或通用反编译 pass。

## Goals / Non-Goals

**Goals:** 只在已呈现的整数值遇到真实 `ireturn Z` 时表达 JVM 最低位，覆盖现有四条返回消费入口，并独立证明行为、来源、单次求值和停止契约。

**Non-Goals:** 不改变普通 `Z` 参数调用、`bastore`、字段以外的布尔赋值、条件 stack phi 或局部类型推理；不从某个观察值猜测所有值只会是 0/1；不生成与原 class 文本相同的源码作为目标。

## Decisions

1. **准入在真实返回消费处。** 同一方法的返回描述符精确为 `Z`、终点指令精确为 `ireturn` (`0xac`)、表达式已呈现为 B/C/S/I 时才允许低位转换。保留 `return_expr` 的已有 boolean 证明和 0/1 常量拼写；缺类型、引用、long/float/double 或未呈现值继续拒绝。`return_expr` 将未知值渲染后交给共享 `adapt_return`，使普通/同步/switch 与字段自增路径一致。不能只看 frame 的 int 形状，也不能放宽 `meeting_position`，否则普通赋值和调用会获得字节码未授权的转换。
2. **复用已有最低位表达式。** 从 `field_value` 抽出小型纯 AST 构造 helper，供 Z 字段写入与 Z 返回分别用自己的消费 BCI 调用；表达式 `value % 2 != 0` 对所有 Java int 都与 `value & 1 != 0` 同真值，负奇数余数为 -1 仍为 true，除数 2 不会抛零除异常。选择它而非复制两份构造或新增节点，是为让两处规则共享已验收语义和发射优先级。该 helper 不做准入、producer 渲染或缓存；调用方各自证明实际字段/返回 descriptor。若实现使用现有 `BitwiseAnd` 写 `value & 1 != 0`，也必须用 AST 和完整负值/优先级对照证明等价，不能拼接裸字符串或改写成 `value != 0`。
3. **原值只出现一次。** 新表达式拥有一份原 `Expr` 子树；前缀调用、字段后/前自增和同步监视器都不因转换被提前或重复。switch 下推仍在各臂求值 BCI 渲染，其共同真实 `ireturn` BCI 只决定转换并锚定来源。字段自增先完成原字段的完整整数更新，然后才计算返回布尔值，不把字段本身变成 `Z`。
4. **来源与停止沿用既有契约。** 外层转换标记实际 `ireturn`，保留操作数原来源；合成常量仅从同一转换位置派生，不声称 class 中多了一条常量指令。使用现有 AST 发射深度、输出/IR/来源预算、取消与原子提交；默认、all、replay 应共享同一已决定正文。失败继续走 `quoted_bcis` 与返回 fallback，不能丢被延期的 effectful producer。
5. **单独固定可重编译的 Java 8 正例。** 现有 boolean-boundary 类同时含 `(Z)B/C/S` raw 2 拒绝，不能把它作为本项完整类成功的 oracle。新建最小 source-only 输入与精确等长 descriptor 补丁，冻结 Code/hash、原 JVM runner 和 Jarde/JADX 完整输出阶段；覆盖参数、奇偶/极值、调用一次、字段前后更新和同步 null。已有 major-49 shared-stack-join 可另取一个 `Z` descriptor 变体，验证求值/返回 BCI，不能把原条件 stack phi 债务伪称为已恢复。每个正例只有零引用且完整类 `javac --release 8` 与原 JVM 实际执行一致才算通过；JADX 若编译失败只记阶段，不修补源码后继续比较。
6. **层次与依赖。** 本项是 source recovery：不改 class parse、dialect 校验、runtime selection 或 JVM verification；合法性由永久 class 的 `java -Xverify:all` 单独证实。reader、SSA、AST、类型、来源均已具备；引入外部反编译/求解库无法替代本方法的真实返回消费证明，还增加维护、许可证和适配成本，因此不加依赖。

依据：[JVMS `ireturn`](https://docs.oracle.com/javase/specs/jvms/se23/html/jvms-6.html#jvms-6.5.ireturn)、[JLS 整数余数](https://docs.oracle.com/javase/specs/jls/se23/html/jls-15.html#jls-15.17.3)。

## Risks / Trade-offs

- 非零判断误把 2、-2 当 true → 固定偶数、负数与极值执行 oracle。
- 只修普通 `return_expr` 漏掉字段快捷返回 → 四入口与真实返回 BCI 分别测试，保留字段完整更新后的值。
- 把当前 raw 2 `(Z)B/C/S` 边界混入 `(I)Z` → 原有 boolean-boundary 类保持拒绝回归，并以独立正例类做完整执行。
- 合成表达式抹去调用或异常 → 运行次数、同步 null 与来源/预算一起验收；任何不能证明的 producer 保留引用。
