## Context

参见 [proposal.md](proposal.md)及 [DT-14 固定回放](../../evidence/java-syntax-2026-09-27/dt14-enum-init/report.md)。当前 `src/enum_constants.rs::prove_group` 只准入 int、无用户参数和 `String...` 数组的物理构造器；`src/class_source.rs` 虽从 Signature 识别 `SingleString`，普通 enum projection 仍不接受它。现有 int ternary 证明检查同类 `invokestatic ()Z`、`ifeq/ifne`、两个 literal arm、唯一 `goto`/构造器汇合；现有 String-varargs 证明处理新建数组及其元素写入，不能作为标量值证据。`StringTernaryInit` 的三个 join BCI 为 20/46/72，构造器准确为 `(Ljava/lang/String;ILjava/lang/String;)V`，正文是 Enum 前缀加 `aload_0; aload_3; putfield own String; return`。

## Goals / Non-Goals

**Goals:** 在现有 class 级完整组证书中证明标量 String literal 或同形条件表达式，按真实分支极性写入每个 enum 常量的唯一源参数；证明构造器仅保存该参数，保留物理成员、BCI、预算和原子拒绝。

**Non-Goals:** 任意 String 表达式、String 数组到标量转换、一般方法条件表达式、非 ASCII MUTF-8 首片、多个用户参数、匿名常量体、复杂 `<clinit>` suffix，或改变既有 int/varargs 证书的准入。

## Decisions

1. **增一个必要的标量 String 证书，不合并不同值种类。** 在 `ProvedEnumSourceArgument` 中只为标量 String 增加有 BCI 的 literal/ternary 值分支，继续让 `StringVarargs` 保有数组分配/元素写入专属证明。复用既有 `ProvedEnumStringLiteral` 的有界 ASCII 解码和 `escape_string` 源码拼写；条件可复用 int 证书的准确同类 `()Z` 成员身份/源码名约束。避免把 String 伪装成 int 或把一个数组参数当成零到多个源参数。
2. **按物理 Code 证明 branch，而非按 JADX 文本反推。** 每项参数只接受同类无参布尔静态调用、紧接的 `ifeq/ifne`、一条 String `ldc` arm、一次直接 `goto`、另一条 String `ldc` arm、唯一 constructor join；检查每条宽度、目标 BCI、栈消费、异常表、完整指令和全部常量字段顺序。极性决定 `trueArm/falseArm`，谓词只在输出的构造实参位置求值一次。可抽取现有 int diamond 的私有共同检查，但只有减少重复且保留类型区分时才抽取；不增加通用 CFG pass。
3. **构造器与输出必须同轮闭合。** 仅在准确 `(String,int,String)` 私有构造器的 Enum 前缀后，唯一 `aload_0; aload_3; putfield` 指向本类唯一 String 字段且紧接 `return` 时允许投影；Signature 必须是同一单 String 源参数。`class_source` 的普通 enum 准入/参数呈现随后消费该证书，写出一个 String 参数声明和每项一个标量表达式；`<clinit>` 及构造器的物理报告继续可查。与现有常量组使用普查、完整性、输出预算和取消合同共用同一原子发布点。
4. **三方运行比文本断言更强。** 固定 `StringTernaryInit` 的 predicate 恒假，只证明 false arm；另建计数/交替 predicate 的可编译 Java 8 正例，使 true、false 两 arm 都执行并检查每常量调用次数、顺序和字段值。对照原 class、固定 JADX 与 Jarde **完整源文件集**，使用 `javac --release 8` 与 `java -Xverify:all`。普通/literal/int-ternary/String-varargs 控制不能回归。

## Risks / Trade-offs

- [错误的 String/数组类型合并导致可编译但不同义] → 参数 descriptor、Signature、构造器字段 store 和完整 `<clinit>` 消费逐层匹配；两个证书分开。
- [反转 `ifeq/ifne` 或复制谓词的副作用] → 记录真实目标 BCI/极性，只生成一个条件表达式，并以交替计数 fixture 验证。
- [枚举组里一项未证仍发布其余常量] → 沿现有 group 原子拒绝，所有失败/停止仍保留物理字段与 initializer origin。
- [共享 helper 抽象超出首片] → 优先私有窄函数和现有数据模型；不为本次引入公共语法或控制流层。
