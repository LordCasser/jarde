## Context

见 [proposal.md](proposal.md)。独立三方证据在 `openspec/evidence/java-syntax-2026-09-22/try-with-resources/`；root 的原样复跑在其 `root-replay/`。普通 nullable factory、初始化失败和三资源抑制链均已由现有 TWR 规则完整恢复；最小 670B null class 的原始/JADX整类可编译且运行相同，jarde 在 BCI 0 得到 `jre_region_exception_edge`，生成 `Object local0` 与合成 `catch(Throwable)`，整类 `javac` 因 `local0.close()` 失败。其字节码为 BCI 0–1 `aconst_null; astore_0`，2–10 正文，10/22 两路 `ifnull`，15/27 `NullResourceCore.close()V`，36 `Throwable.addSuppressed`，40 `athrow`。

`guard.rs::initialises_resource` 只在初始化语句内看到分配或调用时才把候选交给 `twr()`；`initialisation`、正常/异常 close、范围、抑制和重抛证明已存在。`build.rs::resource_declaration` 从存储的 SSA 值取类型，`null` 无命名类型，故即使放行 guard，仍需资源声明的有界类型证据。`jarde-jvm` frame pass 明确不读取 StackMapTable；不能凭外部 `javap` 的局部类型设计实现。`BuildInputs` 已有当前类内部名、直接接口及本次指令/操作事实。

## Goals / Non-Goals

**Goals:**

- 用现有资源规则认领唯一的 `aconst_null; astore` 头，仍由完整协议决定能否发射 TWR。
- 在当前类显式实现 `java/lang/AutoCloseable`、正常与异常 close 调用池属主都是当前类且类型可无歧义拼写时，发射当前类类型的 `null` 声明。
- 保留已有的异常身份、关闭顺序、来源、预算和缺口合同，尤其普通 `try/catch` 的分类。

**Non-Goals:**

- 从 StackMapTable、LVT、外部类层级或任意 `close` 属主推断资源源码类型；不增加 resolver、跨类加载或 AST/IR 节点。
- 处理 Java 9 的 `try (r)`、TWR 外层用户 `catch`、return 尾部、普通 `null` 局部声明类型改进；这些有独立边界/任务。

## Decisions

1. **头的准入与完整证明分开。** 在 `resources()` 的候选过滤中，对同一直线语句内由 `Operation::Push(ConstantValue::Null)` 唯一供给的 `Store` 增加窄准入；要求该异常行具有可辨的正常 close 与自保护异常 close 轮廓，再交给原 `twr()` 检查。这样 `x=null; try { … } catch(E …)` 仍由 `catches()` 处理；近似资源但抑制/重抛坏掉时保留 TWR 拒绝。不能把所有 `null` 存储都视作 header，也不能在区域 fallback 后删除合成 `catch` 文本来假装修复。模式检验必须复用已有 `normal_close`、`close_handler`/`receiver_is` 的事实，而非第二套 CFG 规则；如果某一阶段无法在当前规则上无歧义区分，就只认领可完整证明的正例，拒绝的字节码范围必须仍能点名，不扩大一般 catch 语义。

2. **类型从本次关闭协议与本类声明共同得出。** `resource_declaration` 对非 null 值保持原 frame 类型路径。对已证 `aconst_null` 的资源，从其正常和异常 close 指令取真实调用池属主，二者必须相同且等于 `declaring_class`；本类直接接口包含 `java/lang/AutoCloseable`，当前类名须能作为本类源码中的类型无歧义拼写。若已有 `Resource` 只携带正常 close BCI，就把同一层 guard 已读的异常 close BCI 随这条 Resource 一并传给 builder；这是现有证明的一个锚点，不建立新规则或搜索。`null` 可赋给该类型，同一类内调用自己的 `close` 可访问，因此 `try (CurrentClass local0 = null)` 是可编译的保真声明。若任何一项缺失，回到现有 `Guard` 整区引用，不能输出 `Object`。不采用 `Object` 加强制转换：`Object` 不能作为资源类型，凭空转换也会掩盖无法证明的声明。第一阶段不采用外部属主推断：`close()` 的属主未必实现 `AutoCloseable`，也不能凭无解析的名称证明可访问性。

3. **来源与求值位置不变。** `Resource.init`/`close_bci`、`Plan.facts` 和既有 `ResourceDecl` 足够承载证据；`null` 值在原初始化 BCI 呈现一次，close 与抑制 BCI 为 TWR 语句的派生来源。默认/all 正文相同；仅类型决策消耗本次 class header 与已解码调用事实，不启动新分析或无界搜索。多资源顺序与已有 return/catch 限制不变。

## Risks / Trade-offs

- [普通赋值误判成资源] → 先验正常/异常关闭轮廓，再用完整 TWR 证明；固定 `null` 后普通 catch 和破坏抑制链两种负例。
- [类型表面可写却绑定到错误的类] → 首阶段仅本类直接 `AutoCloseable`、同属主 close 与无歧义的本类类型名；其它保守引用并单列后续类型解析债务。
- [失去引用来源或副作用] → source map 逐 BCI 核验，原 class/恢复文本整类重编译与 `-Xverify:all` 执行对照；不修改输出文本作为测试输入。
