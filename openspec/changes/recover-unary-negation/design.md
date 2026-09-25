## Context

见 proposal.md。`decode.rs` 只将 `0x60..=0x73` 映射为二元算术；`build.rs::render_value` 的算术臂固定要求两个操作数。Frame/SSA 已按真实 opcode 处理四条取负。AST 已有 `Not` 和 `Cast`，发射器已统一处理优先级、预算与来源。

现有大 change 的 2c.11 建议向 `ArithmeticOp` 增加 `Neg`，但该枚举和 `arithmetic_op()` 是二元算术的全映射，把一元运算塞进去会要求不可达分支或可选返回值。因此本独立设计替代该实现建议，保持二元接口含义。

## Goals / Non-Goals

**Goals:** 补齐一个值操作；沿用已有值消费、类型和来源契约。

**Non-Goals:** 不建通用 unary 框架，不改 CFG/SSA，不做代数化简，不扩大纯循环条件或异常区域的识别范围。外围已有规则确实接纳纯值生产者的位置可同步接纳取负。

## Decisions

1. 在现有 `Operation` 添加一个 `Negate` 事实，在现有 `ExprKind` 添加一个 `Neg { value }` 节点。四个 opcode 共用它们，不增加 pass/规则注册/依赖。`Not` 是布尔非，`Binary::Subtract` 的 `0-x` 在浮点零等情况下不等于取负，不能借用。
2. `render_value` 读取恰好一个逻辑栈值（long/double 的 category-2 仍是一项 SSA 值），递归仍传最终消费位置 `at`，不能退回生产点从而绕过旧值存活检查。构造器、concat 等既有值列表的适用分支只增加这个纯数值操作，不放开存储或未知 opcode。
3. 呈现类型按操作数的一元数值提升：byte/short/char/int → int，long/float/double 保持；boolean/reference/未知不能被猜作数值。沿用 `Expr::presented`，不能依靠 Java 返回上下文掩盖未知类型。
4. emitter 沿用 UNARY 优先级。嵌套负号和负整数字面量必须防止词法合并成 `--`；可给嵌套操作数加括号。来源遍历必须进入 Neg 的孩子，取负 BCI 直接锚定本节点；深度和输出预算保持已有边界。
5. 检查 `renders_the_value_it_reads`、拒绝路径的生产者闭包以及表达式来源遍历，保证调用被消费时只写一次、拒绝时仍可见。对照旧值/拒绝转换的既有回归，不放宽断言。
6. 无可复用库缺口：这是现有 AST 的一种局部运算，外部 parser/decompiler 库会引入不必要的模型与维护/许可负担。jadx 只作可读性对照；javac/java 仅运行自写受控 fixture。解析事实、运行时配置、恢复产物和测试中的执行证据分开陈述。

语义依据：[JVMS 取负指令](https://docs.oracle.com/javase/specs/jvms/se23/html/jvms-6.html#jvms-6.5.ineg) 与 [JLS 15.15.4](https://docs.oracle.com/javase/specs/jls/se23/html/jls-15.html#jls-15.15.4)。浮点有限值及零比较 raw bits；NaN 比较分类，不把规范未保证的 payload 当成契约。

## Risks / Trade-offs

- 嵌套取负印成递减 → emitter 文本断言与 javac 执行双门禁。
- 新值消费者误吞调用 → 一次调用、抛异常、操作数拒绝三组测试。
- 大量共享未提交改动 → 只触碰取负路径；新增独立 fixture/test，已有语料 census 与 fingerprint 由主代理集中更新。
- 编译产物膨胀 → 共享 target，`CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0`，不创建第二个大型 target。
