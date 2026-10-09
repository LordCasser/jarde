# 数值转换候选只读对抗审查

## 结论

当前候选的生产路径分层正确：`init` 只把属于物理构造实参依赖闭包的 `PrimitiveConversion` 作为纯值操作接受；Java 类型是否合法仍由 Builder 检查。构造站点结构通过不等于正文恢复成功。Boolean 源值会在 Builder 的转换源类别检查失败，随后由消费语句走既有 fallback。当前缺少的主要验收是：没有测试让真实构造站点携带 Boolean 源值走到 Builder 并验证完整正文 fallback。因此不能将已有 `primitive_conversion_source_categories_accept_int_family_but_reject_boolean` 单元测试描述为 constructor-body 负例。

handler 路径只对本轮新增的实参转换标记启用既有闭区间检查；共享 Budget 和 Stop 传播接线正确，Stop 不会返回半份 `Sites`。两组控制没有显示额外 alias 或扩大到成员/statement 构造的路径。建议补一个仅分析、绝不执行的 verifier-valid classfile 负例，直接覆盖 Boolean 构造实参到完整正文拒绝这一 seam；其余候选保持在当前范围。

## 已证实的拒绝闭环与缺口

- `init.rs:1031-1085` 扫描 allocation-copy 与 constructor 间的操作。只有 `argument_dependencies` 中的 `PrimitiveConversion` 设置 `embedded_primitive_conversion`；无关转换落入原 `StatementFree` 拒绝。该层不推断转换源是不是 Java `boolean`，这是正确的分层。
- `build.rs:24968-24999` 渲染转换时检查单一 stack 输入、呈现类型和 `primitive_conversion_source_matches`。`build.rs:29245-29257` 明确接受 `int` opcode 类别的 byte/char/short/int，拒绝 Boolean 及不匹配的 numeric category。`build.rs:33951-33977` 覆盖类别 helper，但只是函数单测。
- 若某个已验证 Site 的参数转换源是 Boolean，`new_expr` 会沿 `render_value` 传播该错误（build.rs:26264-26339）；普通 return 语句捕获错误后调用 `fallback`（build.rs:22704-22711）。这条代码路径应拒绝该处的完整正文投影，不会把 `boolean` 强行转成 Java 数值表达式。不过本候选新增测试没有构造这个调用闭环；现有 fixture 源码也不含 Boolean 来源转换。
- `init.rs:3543-3589` 的双 JDK 正例核对 Byte/Short/Long/Float/Double 与普通 return-new 的真实转换 op、实参 anchor 和 Site；这些是结构证明，不是 Builder 正文/编译证明。源文件家族确含 `longViaFloat`、`longViaDouble` 舍入敏感成员；相关保证应由 root 的完整原始双流验收给出。新增 extra-dup 变体中的 `l2f; f2l` 是负例，不证明成功路径保留转换链。

## handler、dup 与计费/原子停止

- Handler 正控 `sameHandler` 使用真实 javac 8 与 javac 23 class，原 handler 范围为 `(0,19)`，覆盖 allocation、constructor 和唯一 `astore` consumer（init.rs:3727-3761）。负控只在内存把 `start_bci` 改为 8，仍由真实 `sites` verifier 得到 `jre_new_primitive_conversion_exception_boundary`（init.rs:3763-3799）。这确认转换 marker 会触发既有 ordinal 覆盖闭区间门。它验证的是 handler 事实拒绝，不是完整 report fallback；测试 helper `sites_of` 走测试专用 unmetered `sites()`。
- `primitive_pair_extra_dup2_variant` 从真实 `(JJ)V` fixture 替换第二实参片段为 `dup2; l2f; f2l`，保留 constructor/返回描述符并调整 Code 长度（init.rs:4117-4205）。原 `max_stack >= 6`，新栈峰值仍为 6；断言未降低 max stack 合理。变体没有分支、异常表或 Code 子属性，因而无需重写 offset 表。SSA 断言真实 `dup2` 一读两 distinct 输出，候选仍以 `jre_new_interleaved_effect` / `StatementFree` 拒绝（init.rs:3593-3679）。该控制只用 javac 8 类派生；由于变体针对通用 opcode/SSA 规则，单腿可作为 focused 控制，但不应声称双 JDK extra-dup 验收。
- 生产 `sites_after_array_composition` 用共享 Budget 构造 meter 并直接调用 `verify_metered`（init.rs:515-545、576-616）；meter 在每次 charge 前 poll，数组 pending Site 在入口只移动一次。Report 调用点将 Stop 映射到 `stopped` 并立即返回，Region 不会收到局部 Sites（report.rs:9394-9409）。Budget 测试将 `analysis_steps` 限为 1，验证 verifier 内 BCI 4 的实际 Stop；取消测试在 census 首个 charge、BCI 0 即停止（init.rs:3437-3533）。返回类型使 Stop 时不可能暴露部分 Sites。取消例是入口早停，不应描述为覆盖了 handler loop 内取消；生产代码中的逐项 charge 已保证该循环可停止。
- 无关 conversion 负控将 `(I)Byte` 变体中插入非实参依赖的 `i2l`，确认仍落到 `StatementFree`（init.rs:3681-3724）。它验证依赖成员资格门，没有引入额外转换形状。

## 最小后续验收建议

在现有 `wrapperByte` class 的方法描述符上做同长度的内存变体，把 `(I)Ljava/lang/Byte;` 改为 `(Z)Ljava/lang/Byte;`，保留原 `i2b` 与 `Byte.<init>(B)V` 字节码。该类文件对 JVM 的 int 类验证形状仍成立，但 Java source 将 `boolean` 传入 numeric cast 不合法。只分析/渲染，不运行该类：先确认 init 的结构 Site 可保留，再确认 Builder 因 Boolean source-category 拒绝完整方法正文，原语句及 `new/dup/i2b/invokespecial/consumer` 不能被静默折叠为成功正文。测试应单独标明“结构 Site 已验证、Java 正文拒绝”，不可把它测成 init 的结构拒绝。

数值 source-category 的现有 helper 表覆盖若干错误类别，但若需要 constructor 端到端负例，可复用同一 report seam，对一个实际转换 opcode 配置与 source descriptor 不匹配的 classfile 变体作只分析控制。无需扩成新的 Java 类型规则或执行非法源码。

本审查只读当前候选和既有 fixture/test 源，不运行 Cargo、Git、rustfmt 或 Java；未修改产品/测试。此报告不替代 root 的完整双 JDK 原始 stdout/stderr、编译和语义验收。
