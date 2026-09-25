## Context

三方输入、13行执行结果和额外合法栈变体在`../../evidence/java-syntax-2026-09-22/throws/`。
Operation::Throw、终止CFG和异常行已存在；build把未被guard认领的Throw与Monitor一起引用，
StmtKind无Throw。init::renders_its_reads不接受Throw，构造异常因此不能被唯一消费者认领。
原有throws声明已经正确呈现。guard拒绝finally/synchronized样例是另一层问题。

frame的AThrow只验证初始化reference形状并清栈，不证明Throwable继承层级；SSA先记录真实
SlotTouch，再丢弃stack_after上方定义，清栈本身不是读取。不能将帧结果称作完整JVM验证。

## Goals / Non-Goals

**Goals:** 在已接受区域内补齐一条真实语句的表达和消费，保留异常身份与生产者效果；用既有来源、类型、预算和失败规则闭合。

**Non-Goals:** 不解析外部Throwable层级，不修复无效class，不添加异常调度器、栈清理pass或guard新模式；不承诺本项覆盖finally/synchronized全部路径。

## Decisions

1. 增加`StmtKind::Throw { value: Expr }`，作为已有Throw事实的忠实语句。不能借用Expr(Call)或Return拼字符串，它们具有不同终止和计数语义。复用emitter的表达式、node来源、语句计数及commit/replay；对AST既有遍历做穷尽适配，不增加独立遍历体系。
2. 在当前instruction dispatch中，guard已经持有的指令继续由guard负责；普通Throw读取SSA实际唯一异常栈操作数，以athrow BCI为最终求值at递归render_value。使用原表达式，不通过统一Throwable cast掩盖静态类型缺失，不依据类名后缀猜继承关系。这里输出的是原指令表达，不新建verifier；编译/验证平面仍按原契约记录。
3. 只在现有new、invoke、checkcast消费判据中接通Throw。new仍需原new/dup/constructor及唯一消费证明；不把Throw加入所有concat、guard、region白名单。延期是否允许与最后是否能真实消费保持一致，失败经现有quoted_bcis追溯生产者；避免独立调用加throw表达式导致两次执行。
4. 栈中较低值即使被athrow丢弃，也不是throw的读取。直接使用SSA的真实reads/uses即可保持其独立效果；不得把“栈最终为空”转换成额外消费边。`ThrowStack`的nop变体只验证此边界，nop自身是否可恢复不属于本项。
5. 复用类型、名称、声明与region。已恢复的条件和try/catch自然承载Throw；不另算出口，不为通过整个混合审计而修改guard拒绝。执行用单独的可覆盖正面类和原样负面类，保留所有实际输出，明确哪类能够整类编译。
6. 无新增库。现有Rust枚举和发射器缺一个忠实语句分支，外部反编译器/求解器不能补齐这一内部表达缺口；增加维护、依赖和许可负担没有收益。也不引入新缓存、索引或pass。

依据：[JVMS athrow](https://docs.oracle.com/javase/specs/jvms/se23/html/jvms-6.html#jvms-6.5.athrow)、[JLS throw](https://docs.oracle.com/javase/specs/jls/se23/html/jls-14.html#jls-14.18)。自写class的javac/java对照是验收程序，产品不执行目标代码或自动下载其依赖。

## Risks / Trade-offs

- producer延期后被拒绝消费者吞掉 → 固定嵌套调用/cast/new拒绝来源回归，校验throw与每个必要生产者BCI。
- 求值位置缩回producer绕过旧局部值检测 → 始终递归传最终throw at，构造跨写入负例。
- null或cast异常顺序改变 → 同时执行参数null、兼容、不兼容对象和生产者自身抛错。
- clear-stack误作消费 → 合法额外栈值变体检查独立调用仍执行一次。
- synthetic rethrow重复、guard范围扩张 → 现有TWR/monitor/catch正反例回归，不因本项修改它们的准入。
- 新语句来源和预算不同步 → 复用统一发射路径，真实Engine来源/默认无来源/预算停止回归。
- 与numeric共享build/facts等文件 → fixture可独立准备，生产仅在root移交后实施；不并行抢写。
