## Context

见 `proposal.md` 与 [DT-06a 冻结证据](../../evidence/java-syntax-2026-09-27/anonymous-super-direct/report.md)。DT-05 已把匿名接口单点投影建立在 typed `InnerClasses`/`EnclosingMethod`、同次 `AnonymousAllocationScan`、闭合 owner XRef、同次方法 AST 和原子文本发布上。本例的子类无字段，调用者在 BCI 0 创建 `$1`、按 BCI 4/7 求值 `next()`，BCI 10 调用 `$1.<init>(II)V`；子类构造器把两个参数传给 `Base.<init>(II)V`，`Base` 还声明 `(IJ)V`。当前完整物理源码已能 Java 8 重编运行，因此缺口是源级匿名表达式及正确实参角色，不是通用构造器恢复。

JADX 的 `AnonymousClassVisitor.getArgsToFieldsMapping` 用 SSA 单消费识别合成字段捕获或 `super` 实参；`InsnGen.inlineAnonymousConstructor` 选择父类参数并发射表达式。这两个入口可借鉴，但 `ProcessAnonymous.checkUsage` 按使用方法数而非分配 BCI 决定内联，在双站点反例会改变类身份。本实现继续用 Jarde 的逐 BCI 与完整范围证书。

## Goals / Non-Goals

**Goals:** 在 DT-05 的投影路径上补一个无捕获父类分支。源码实参直接从调用者同次 AST 原顺序发射；子类构造器只证明这些参数被原样转发到准确的父类构造器。物理类报告、成员身份及各自来源保持独立，拒绝时根类全部文本回到原物理表示。

**Non-Goals:** 合成捕获字段/外层实例、实例初始化器、字段、构造器参数变换、可变参数语法、泛型父类、跨包访问推理、非直接返回、嵌套匿名、多分配点和整个 jar 的项目导出。现有含捕获 DT-06 证据以及 DT-08 捕获任务仍单独处理；本切片不能宣布整个 DT-06 已追平。

## Decisions

1. **在现有匿名类族路径上增加分支，不复制扫描器。** 调用者必须有完整类/方法表，直接 `return new` 的 AST 与唯一 `AnonymousAllocationScan` 按 head/constructor BCI、准确子类定义和实参生产 BCI 对齐；所选输入范围的完整 owner XRef 只允许该创建、构造与两份已核验匿名 self row。子类的 typed `EnclosingMethod` 必须准确指向调用者物理方法。DT-05 接口分支继续保持原有零实参门槛；父类分支不得靠二进制 `$1` 命名猜匿名身份。
2. **区分物理参数和源级父类实参。** 选定子类必须零字段、零接口、唯一构造器；其物理构造器参数描述符与准确选中的父类构造器参数描述符相同。用同次构造器 AST/初始化 prologue、SSA effect 与完整原始 Code 共同证明：接收者为 `this`，各参数按描述符 slot 与原次序直接加载，唯一 `invokespecial` 的 owner/name/descriptor 是选定父类 `<init>`，随后仅返回；无分支、异常处理、字段写入、变换或未归属指令。父类类定义和该重载在同一所选环境唯一且为同包源码可访问；任一缺口拒绝。借鉴 JADX 的参数角色映射，但对本切片直接拒绝合成捕获，不新增通用捕获转换机制。复用 DT-05 已留存的子类方法 AST 和现有 raw Code/初始状态读取；若当前平凡构造器候选不能表达实参转发，只扩展同次私有证明入口，不改公开报告 schema。
3. **实参原 AST 发射而不从构造器文本反推。** 扩展 DT-05 发射器的准确 `New` 节点替换：写父类源类型与 `(`，逐个调用原 `Expr` 发射并保持原索引和求值次序，再写 `) {` 与已证子类方法体。不得复制、重排、常量化或以文本搜索重建 `next()`。`super.sum()` 等子类方法仍使用其同次 AST/普通格式器；每个方法需完整 Java 表示、合法 Java 8 声明与可解析的父类成员访问。根类源码仅在所有方法、构造器、身份、访问性与输出预算均成功后一次性替换；预算停止或任何拒绝不得先写 `Base`/匿名语法。不要依赖可选 SourceMap/RuleDetails 选项来决定投影。
4. **验证完整语义而不把源码相似当证明。** 固定 class SHA 和 `javap`；原/JADX/修后 Jarde 的完整源单元用 `javac --release 8 -g:none` 与 `java -Xverify:all` 对照，检查 `13:2`、两次调用次序、`(II)V` 重载和 `super.sum()` 分派。源单元编译根类加真实 `Base`，独立物理 `$1` 报告另查，不与 javac 生成的同名匿名 class 一起编译。负例包括参数位置或目标描述符变动、捕获字段、第二 BCI、跨类引用、混合/不完整子类方法、低预算和取消。
5. **依赖与架构边界。** 当前 reader、SSA、AST、Java emitter 和 `jarde-query` 足以表达这些事实；第三方库无法替代所选定义身份和同次预算证书。核心 crate 不依赖 CLI 或 JADX；JADX 仅作为本地算法与 Java 8 结果参照。

## Risks / Trade-offs

- `[两个 next() 被交换或复制]` → 对照调用点 AST 实参顺序、分配扫描生产 BCI、构造器 slot 转发与运行效果；任一不一致拒绝。
- `[错选 (IJ)V 或不可访问构造器]` → 精确核对常量池目标描述符和所选父类定义、声明、同包访问；不凭参数个数或文本猜重载。
- `[捕获值泄露进父类实参]` → 当前只收无字段、构造器描述符完全同形且所有参数只直达 `super` 的子类；含捕获证据留给后续独立机制。
- `[半个类族或重复匿名身份]` → 沿用 DT-05 BCI/XRef 闭合证明和最后原子发布；预算与取消负例必须看到原物理文本。
- `[现有静态成员分支提前改写源名]` → DT-02 审计债务在其自身 OpenSpec 中修复，不引入本任务或复用其未闭合投影路径。
