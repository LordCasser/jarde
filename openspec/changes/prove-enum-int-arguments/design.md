## Context

参考 `proposal.md` 与冻结对照 `openspec/evidence/java-syntax-2026-09-27/enum-constructor-arguments/README.md`。当前普通 enum 证明在 `src/enum_constants.rs` 中把 source argument 收窄为 `AnyLiteral | Exact(i32) | None`，输出模型是 `Option<i32>`，prefix matcher 将 source int 固定为一个 literal 指令；类文本投影随后直接打印该整数。完整组、构造器、`<clinit>` prefix/suffix、values factory/API 和 use census 已经是同一个证书边界。

## Goals / Non-Goals

**Goals:** 在不削弱组级证明门的前提下，把完整 int 实参的受限表达式原样投影进常量列表；将静态读取的可观察初始化语义留在原位置。

**Non-Goals:** 不实现 String/数组或通用 Java expression recovery；不扩展单方法恢复；不改变公开 JSON/API；不把无法证明的部分常量改为猜测输出。

## Decisions

### 1. 扩展当前证明值，不加平行 enum recognizer

继续由既有 class-source 同次 run 的 `EnumMethodCodeCandidate`、物理表索引、constructor 信息和 initializer facts 证明 ordinary group。用闭合的 int argument proof value 表示 literal、字段读、字段读加 literal；每个字段叶都携带 resolved-from-same-run 的 owner/name/`I` descriptor，并绑定 initializer BCI 与相应 constructor call。不要保存或解析 decompiler 文本，也不要重新读取 class 文件。

先支持 fixture 直接命中的 `getstatic I` 与 `getstatic I; iconst/bipush/sipush/ldc-int; iadd`。如果实现同时覆盖 literal 左操作数，必须按真实操作数顺序保留节点；不为交换律重排表达式。除此之外的方法调用、两次字段读取、其他 arithmetic opcode、分支、异常路径和额外栈消费者继续拒绝。

### 2. 用已有表达式拼写能力发射值，静态读不常量折叠

把证明值转成现有 int field/arithmetic 表达式或一个仅服务此证明的闭合值树，再由共同的 Java 拼写逻辑输出。owner 必须经现有 JVM internal-name 到 Java type 拼写路径形成限定表达式；无法形成确定合法拼写则拒绝该组。`getstatic` 具有类初始化及异常语义，必须只在原参数的求值位置输出一次；值可能 mutable，不能查运行环境预求值。

### 3. 保持 atomic projection 与 current gates

`source_argument` 从整数扩展为有类型、可拒绝的已证明表达式值，但 `prove_initializer_prefix` 仍逐项绑定物理常量字段、ordinal、调用 BCI、构造器 descriptor 与 `putstatic`。现有 initializer suffix、`$VALUES` factory/API body 证明与全方法 use census 不变并继续是投影前置条件。字段表或方法表不完整、预算/取消、extra uses、构造器非标准或表达式 grammar 失败时，完整 group 不进入 emitter。

验证必须有 literal control、静态字段 control、字段加 literal、未支持的 method-call/extra-field-read 拒绝以及初始化/使用顺序对照。用 Java 8 javac 重编 Jarde 输出并运行 runner 检查常量值和副作用；对拒绝变体检查没有部分常量被投影。不要把 JADX 文本当作唯一 oracle。

## Risks / Trade-offs

- [静态字段读可触发类初始化或异常] → AST/闭合 proof value 保持原位置和次数；不计算字段值、不做常量替换。
- [限定类型名拼写可能与源访问能力不一致] → 只允许已经能由当前 source spelling 规则确定的 owner；无法合法拼写或存在额外 source gate 时整组拒绝。
- [拓宽表达式会误收额外栈使用或更复杂算术] → 只接纳明确 opcode 序列并绑定 operands/use census；拒绝任何未声明消费者或指令。
- [字符串 varargs 与 int 证书混合会拉宽模型] → String/array arguments、varargs Signature 和零长度数组规则留在另一项，不加入此 change。
