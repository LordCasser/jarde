## Context

见 `proposal.md` 和 `../../evidence/java-syntax-2026-09-22/compound-assignments/README.md`。现有 `FieldAssign`/`IndexAssign` 将左值与普通 `=` 右值分开渲染；`field_write`/`array_write` 从最终 store 逐个渲染 SSA 操作数，遇到 `dup`/`dup2` 及读旧值时保守引用。`field_increments` 仅认领本地变量接收者的八指令 `return x.n++/++x.n`；它写成一个预格式化 `ExprKind::Local`，不支持调用接收者或数组，也不应承担本 change。已有调用延期消费、`render_value`、来源和预算提供复用基础。

## Goals / Non-Goals

**Goals:** 对 `int` 实例字段及 `int[]` 元素 `+=` 的一块内可证明语句建立完整的单次左值认领。恢复正文需与原始 class 在值、效果、异常顺序上相同，来源可审计。

**Non-Goals:** 后置/前置递增结果、表达式位置复合赋值、一般 `dup` 别名、静态/窄类型/长整数更新、其它算符、跨块写入或源码风格复原。已有局部 `+=` 的 `arg = arg + rhs` 等价正文保持原路径。

## Decisions

1. **给现有字段/数组赋值语句增加真实的复合赋值语义。** 一个小的赋值运算模式表示 `=` 与 `+=`，而不是把 `receiver().value`/`data[index()]` 重复放进 Binary 右侧，也不是借 `ExprKind::Local` 承载整句文本。现有 `FieldAssign`/`IndexAssign` 的接收者、下标和值节点仍供统一 emitter/source map 使用；只有已证明形状选 `+=`，其余保持 `=`。这一必要语义区分比新建通用左值 IR 或追加另一个输出 pass 更小，并能防止副作用接收者重复求值。具体 enum 或等价小表示由实现保持最小闭环。
2. **从最终写入反向证明一个局部更新链。** 字段要求同一基本块的 `dup; getfield; RHS; iadd; putfield`，读取/写入的 owner、name、descriptor 均相同且为 `I`，`getfield` 的接收者是原接收者复制值，`putfield` 消费另一复制值与加法结果。数组要求 `dup2; iaload; RHS; iadd; iastore`，数组和索引两份复制分别同源，数组为已证明 `[I`，读/写操作确属 `iaload`/`iastore`。RHS 使用已有递归值构造/调用消费，且必须是链内单次消费；不以邻近 opcode 代替 SSA 身份，不把任意 `dup`/`dup2` 解释成复合赋值。判据及深度受现有预算/取消轮询控制。
3. **把语句的求值位置放在最终 store，子表达式保留其顺序。** 先在赋值左侧渲染接收者、数组和索引，再渲染右侧。`+=` 的 Java 语义在 RHS 前执行左值读取及 null/界限检查，因此无需额外显式检查语句。旧值不能从 RHS 完成后的同名字段/数组重新读取；RHS 若改写左值仍按先前读出的旧值运算。对需要通过局部读取的值，沿用当前最终消费位置检查；若无法证明延期后仍指向同一值，则拒绝，不以移动效果换覆盖率。
   还须证明左值依赖从其最早指令到复制之间没有独立效果，复制与旧值读取之间、求和与最终写入之间无可观察间隙；RHS 的旧值读取到求和区间也只能由它的完整依赖链占据。否则把整句移到 store 会把间隙中的效果重排，哪怕 SSA 消费唯一也必须拒绝。
4. **认领整条链，失败则保持原引用范围。** 只在所有证明完成后，使接收者/索引调用、复制、读取、RHS、加法和写入由同一更新语句拥有，避免生产者先作为独立语句发射，又嵌入更新执行两次。所有组件的真实 BCI、Fieldref 身份加入原有 origin/source map；若形状失败，既有 fallback/deferred 路径仍负责引用生产者及消费链。新形状不能吞掉后置 `dup_x1`/`dup_x2` 的返回值，也不能把该类不可编译结果假称为本项完成。
5. **不引入依赖或执行目标。** 现有 SSA/field/array facts 已足以证明限定场景；引入解析/重编译库不能代替这份数据流证明。仅自写 Java fixture 在受控测试中由 javac/java/JADX 执行；产品仍只解析目标 class，不自动运行它或下载依赖，parse/方言/verification/源码编译报告语义不变。

依据：[JLS 8 §15.26.2](https://docs.oracle.com/javase/specs/jls/se8/html/jls-15.html#jls-15.26.2) 对单次求值、旧值快照与数组检查先后的规定，以及 [JVMS 8 `dup2`](https://docs.oracle.com/javase/specs/jvms/se8/html/jvms-6.html#jvms-6.5.dup2) 和字段/数组指令语义。

## Risks / Trade-offs

- 只看 opcode 邻接可能把普通赋值或其它复制链误认作 `+=` → 检查 SSA 每一份复制与最终写入身份，并配普通 `=`、多消费者、异成员及异索引负例。
- RHS 调用先独立发射或被跳过 → 以原类的计数、同左值改写、抛错路径及 source map 验证调用恰好一次；拒绝时引用全部生产者。
- 数组普通 `=` 与复合 `+=` 的异常时机不同 → null、越界与 RHS 副作用成对测试，不能用简单赋值替代。
- 宽泛声明把 `byte`/`char`/`short` 的隐式窄化或 `long` 双槽复制误处理 → 本次只允许字段 `I`、已证明 `[I`，其它保持拒绝或既有路径。
- bitwise 与本项共享 build/AST/emit 文件 → fixture 与规格先独立落地，生产按 root 的 Cargo/文件窗口串行移交；未合入的目标工作不作为本项修复前提。
