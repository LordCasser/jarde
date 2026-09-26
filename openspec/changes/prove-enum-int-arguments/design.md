## Context

参考 `proposal.md` 与冻结对照 `openspec/evidence/java-syntax-2026-09-27/enum-constructor-arguments/README.md`。当前普通 enum 证明在 `src/enum_constants.rs` 中把 source argument 收窄为 `AnyLiteral | Exact(i32) | None`，输出模型是 `Option<i32>`，prefix matcher 将 source int 固定为一个 literal 指令；类文本投影随后直接打印该整数。完整组、构造器、`<clinit>` prefix/suffix、values factory/API 和 use census 已经是同一个证书边界。

## Goals / Non-Goals

**Goals:** 在不削弱组级证明门的前提下，把完整 int 实参的受限表达式原样投影进常量列表；将静态读取的可观察初始化语义留在原位置。

**Non-Goals:** 不实现 String/数组或通用 Java expression recovery；不扩展单方法恢复；不改变公开 JSON/API；不把无法证明的部分常量改为猜测输出。

## Decisions

### 1. 扩展当前证明值，不加平行 enum recognizer

继续由既有 class-source 同次 run 的 `EnumMethodCodeCandidate`、物理表索引、constructor 信息和 initializer facts 证明 ordinary group。用闭合的 int argument proof value 表示 literal、字段读、字段读加 literal；每个字段叶都携带原始 owner/name/`I` descriptor、source spelling 与 initializer BCI，并绑定相应 constructor call。不要保存或解析 decompiler 文本。

对 `getstatic` 的跨类事实，在 facade 的类级装配边界复用 selected-environment dependency read：owner 必须唯一解析，完整字段表中必须唯一匹配 `name:I`，字段必须同包可访问、`static` 且非 `final`。外部 owner 必须是 Java 8 可写的顶级类型；拒绝同 enum owner、跨包 owner、含 `$` 的二进制 owner、私有/受保护、synthetic 或 interface/annotation class flags、带 `EnclosingMethod` 或声明自身为 member 的 class attributes、解析歧义或不完整读取。静态字段用同包短类型名发射，因此还要读取 enum 的完整 `InnerClasses` facts，并拒绝遮蔽该短名的同包 member type。保留原符号引用并发射访问表达式，不读取字段值。限制为同包顶级非 final 字段，避免 Java 编译器将 constant variable 内联并改变类初始化时机。

先支持 fixture 直接命中的 `getstatic I` 与 `getstatic I; iconst/bipush/sipush/ldc-int; iadd`。如果实现同时覆盖 literal 左操作数，必须按真实操作数顺序保留节点；不为交换律重排表达式。除此之外的方法调用、两次字段读取、其他 arithmetic opcode、分支、异常路径和额外栈消费者继续拒绝。

### 2. 用已有表达式拼写能力发射值，静态读不常量折叠

把证明值转成一个仅服务此证明的闭合值树，再由现有 class-source Java 拼写逻辑输出。每次 `getstatic` 具有类初始化及异常语义，必须只在原参数的求值位置输出一次；值可能 mutable，不能查运行环境预求值。

### 3. 保持 atomic projection 与 current gates

`source_argument` 从整数扩展为有类型、可拒绝的已证明表达式值，但 `prove_initializer_prefix` 仍逐项绑定物理常量字段、ordinal、每条前缀指令及其 reference/BCI、构造器 descriptor/call 与 `putstatic`。该完整、无 exception handler 的同次 raw Code 证书是 enum `<clinit>` 前缀的权威边界；它逐字节消费连续指令，并证明常量字段顺序和 `$VALUES` 发布。泛用 AST initializer sidecar 仍用于同次身份与 handler 状态，但其不能分类 `new; dup; getstatic; invokespecial` 时不阻断已证明前缀；物理 `<clinit>` 方法报告和 fallback markers/诊断仍原样保留。

前缀之后仅允许 raw Code 的单条 terminal return，或当前既有证书完整证明的 static assignment suffix。其他 suffix、不完整代码、未解释指令或引用、use census 变化、字段/方法表不完整、预算/取消、构造器非标准或表达式 grammar 失败时，完整 group 不进入 emitter。对 static suffix 的既有 sidecar/Code 检查保持不变。

验证必须有 literal control、静态字段 control、字段加 literal、未支持的 method-call/extra-field-read 拒绝以及初始化/使用顺序对照。用 Java 8 javac 重编 Jarde 输出并运行 runner 检查常量值和副作用；对拒绝变体检查没有部分常量被投影。不要把 JADX 文本当作唯一 oracle。

## Risks / Trade-offs

- [静态字段读可触发类初始化或异常] → AST/闭合 proof value 保持原位置和次数；不计算字段值、不做常量替换。
- [限定类型名拼写可能与源访问能力不一致] → 只允许已经能由当前 source spelling 规则确定的 owner；无法合法拼写或存在额外 source gate 时整组拒绝。
- [拓宽表达式会误收额外栈使用或更复杂算术] → 只接纳明确 opcode 序列并绑定 operands/use census；拒绝任何未声明消费者或指令。
- [AST initializer fallback 无法表示交错 new/dup/getstatic] → 让同次完整 raw Code/BCI 证书拥有 enum prefix；保留 physical method fallback，并继续要求 raw suffix 与剩余组证书通过。
- [字符串 varargs 与 int 证书混合会拉宽模型] → String/array arguments、varargs Signature 和零长度数组规则留在另一项，不加入此 change。
