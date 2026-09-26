# 验证记录（任务组 2 + 3.1）

本记录覆盖 tasks.md 任务组 2（2.1–2.4）的实现实测与 3.1 的实际恢复对照。工作基线为
工作树上未提交修复之上（HEAD `2ad29cee`），未执行 `git commit`。对照环境：本机
OpenJDK 17.0.20.1（javac/java）、rustc 1.98.1；`cargo test --all-features`。

## 2.1 facts/decode 与 AST/类型

- `crates/jarde-java/src/facts.rs`：`ConstantValue::Float(u32)`/`Double(u64)` 保留原始
  bits；`float_presentation`/`double_presentation` 是唯一 NaN 位模式准入判断，只接受
  `0x7fc00000`/`0x7ff8000000000000`（设计决策 1、4）。任何路径不再经过宿主 `f32`/`f64`。
- `crates/jarde-java/src/decode.rs`：`fconst_0/1/2`（0x0b..0x0d）、`dconst_0/1`（0x0e/0x0f）
  以精确 bits 进 `Push`；池 `Float` 只被 `ldc`/`ldc_w` 接纳、`Double` 只被 `ldc2_w` 接纳
  （JVMS 6.5 类别规则，错配保持 `Other`）。坐标测试：
  - `the_float_and_double_constant_opcodes_state_their_exact_bits`（fconst/dconst 五个
    opcode 的 BCI 与 bits）；
  - `pool_float_and_double_entries_reach_ldc_with_their_raw_bits`（池 1.5f、-0.0f、3.0d
    的 bits 直通，Double-on-ldc 与 Float-on-ldc2_w 拒绝）。
- `crates/jarde-java/src/ast.rs`：`ExprKind::Float(u32)`/`Double(u64)` 有限叶子，
  `presented_of` 由叶子自身声明 `Float`/`Double`（测试
  `floating_leaves_present_their_own_type_from_their_own_bits`）。无新数值抽象层、无求值器。

## 2.2 统一 emitter 与特殊值呈现

- `crates/jarde-java/src/emit.rs`：`spell_float`/`spell_double` 用整数分解符号、指数、
  尾数生成精确十六进制字面量（float 尾数 23 位左移一位写 6 个十六进制位、double 写 13
  位、各自偏置；指数域 0 走零整数部 + 最小正规指数 p-126/p-1022；符号位单独保留，负
  零不失；后缀 f/d 恒写）。测试
  `floating_literals_are_spelled_exactly_from_their_own_bits` 覆盖 ±0、±1、2、0.1、0.25、
  次正规、最小正规、最大有限（float/double 各 8 例，与 536 项拼写实验同一格式）。
- 特殊值由既有 `Binary(Divide)` 表达（build.rs `special_value`）：+∞=`1/0`、-∞=`-1/0`、
  标准正 quiet NaN=`0/0`，三个节点均从常量自身 BCI `derived`，不伪造独立原指令（设计
  决策 3）。
- 取负词法分组识别浮点字面值符号位（尤其 -0），不再产生 `--`（emit 测试补
  float/double 负字面量两例）。
- 次正规、负零、嵌套优先级、重载与比较的类型回归由
  `tests/p3_floating_constants.rs` 全绿覆盖（`recovered_text_keeps_float_double_types_
  grouping_and_consumers` 逐方法断言 `0x` 字面量与 `f;`/`d;` 后缀、特殊值保留 `/`、
  `FloatingSupport.pick` 重载实参、`nested` 两个 `-`、`threshold` 比较极性）。

## 2.3 NaN 准入、闭合浮点树与有名值保存

- 未支持的 NaN（负 NaN、signaling、payload 变体）在常量自身 BCI 明确拒绝；失败生产者
  路径保留常量与消费者：`deferred_producers` 把 `Push(Float/Double)` 与 Class 字面量
  同列为"无自身语句"的必要 quote 来源。build.rs 测试
  `an_unpresentable_nan_bit_pattern_is_refused_at_its_own_bci`：正文为 quote，BCI 列表
  为 `@bytecode 2 0`（失败 freturn 消费者 + 常量），无任何归一化文本，表示非 `Java`。
- 有界常量树判断（`MAX_VALUE_DEPTH=24`）：`folding_candidate` +
  `closed_floating_constant_tree`（只认浮点常量叶子与浮点 `Negate`/`Arithmetic`，树内
  含任何其它叶子即不闭合）。闭合且含特殊值的真实 `fneg`/算术复用既有
  `prepare_deferred_bindings`→`bind_value`→`Declare` 提交链：操作数成为非 final 局部，
  真实操作留在运行时；不新增浮点专用临时变量表、命名器或调度 pass（设计决策 7）。
  两处与既有排序判定的接缝：folding 候选在"无独立边界"时同样是候选（闭合常量正是
  保存的适用形状）；嵌套时从不被外层声明吞掉（外层初始化式会重新成为闭合常量表达式）。
- build.rs 测试（经 `test_class::single_method_floating` 组装的真实分析+恢复全链路）：
  - `a_closed_nan_negation_saves_its_operand_so_the_real_operation_stays_runtime`：
    float 与 double 两例均产出 `saved0 = 0/0 常量除法;` 与 `return -saved0;`；
  - `finite_and_open_negations_render_inline_without_a_saved_local`：有限闭合取负
    `return -0x1.000000p0f;`（折叠逐位一致，无需保存）与含参开放树都不新增局部；
  - `reader::classfile::test_class` 新增 `single_method_floating`（float 变体占一个池
    槽、wide double 变体按 JVMS 4.4.5 占两个池槽），仅测试支撑模块。
- 有限叶子与已准入常量直接返回、存储、实参、比较的正常支持未被拒绝路径影响
  （p3_floating_constants 全绿；全套 triage 无新回归，见下）。

## 2.4 证据与预算契约

- `essential_and_all_have_the_same_text_but_only_all_has_source_origins` 通过：默认与
  完整请求正文逐字一致；默认请求无来源表（`NotRequested`），完整请求 `Complete` 且每
  段 origin 有真实 BCI 与成员身份（合成特殊值的除法与子表达式 anchored 在常量的真实
  BCI 上）。
- 输出/来源不足沿用既有 put/commit/replay 原子停止与 `MAX_VALUE_DEPTH` 停止边界；本
  change 未绕过任何停止路径（emit/engine 既有预算回归保持通过；全套 triage 无新失败）。

## 3.1 实际恢复对照（证据见 `evidence/task-3-1/`）

以下全部使用**实际恢复输出**重编译执行，未用手写候选拼写替代。

1. **36 行完整类基线**：`cargo test --all-features -p jarde --test p3_floating_constants
   -- --ignored`（`recovered_complete_class_matches_frozen_original_raw_bits`）实跑通过：
   恢复正文经 `javac --release 8` 编译、`java -Xverify:all` 执行，36 行 raw bits 与冻结
   原类逐行一致（含 floatZero..threshold、floatArgument=1/doubleArgument=2 的重载分派、
   nested 的运行时取负、threshold 比较极性）。
2. **536 项有限 bits**（`floating-bits-536/`）：以证据同源生成脚本（seed 20260923，
   12 组固定边界 + 随机 256 组/宽度，排除 exponent 全 1）生成 536 方法类
   （268 float + 268 double），`javac --release 8 -g:none`；jarde 实际恢复全部 536 方法
   为 `Recovered`；恢复正文重编译后 `java -Xverify:all` 执行，536 行 raw bits 与原类
   逐行一致（`diff` 空）。输入/恢复正文/两侧输出与 class SHA 见该目录。
3. **canonical NaN 运行时取负**（`nan-negation-recovery/`）：按 task-1-2 的 fneg/dneg
   Code 补丁修改冻结类（SHA-256 `226eb2c1…` 与 task-1-2 记录逐字节一致）；原类
   `-Xverify:all` 运行 `floatNan=ffc00000`、`doubleNan=fff8000000000000`（JVM 运行时
   取负 bits）。jarde 实际恢复正文为 `float saved0 = 0x0.000000p-126f /
   0x0.000000p-126f;` + `return -saved0;`（double 同构），不是会被折叠的闭合常量表达
   式；重编译执行 raw bits 与原类一致。对照实验（本机同 JDK）：直接写
   `-(0.0f/0.0f)` 被 javac 折叠为 `7fc00000`（正 NaN）——证实保存而非内联是必要边界。
4. **真实 0/0 操作**：task-1-2 的 `runtime-zero-div-zero` 变体在恢复正文中的形状
   （`fconst_0; fconst_0; fdiv;`）即本 change 对 0/0 的标准呈现，其 bits 行为已由第 1
   项的 36 行对照（`floatNan=7fc00000`）与第 3 项的保存形式覆盖。

## 套件状态与遗留

- 验收钉子：`cargo test --all-features -p jarde --test p3_floating_constants` —
  3 passed, 1 ignored（ignored 项实跑亦通过）。
- 回归：`p3_unary_negation` 7 passed。全套（`-p jarde` / `-p jarde-java` /
  `-p jarde-cli`，`--all-features`）与 HEAD `2ad29cee` 基线对比：
  - HEAD 43 项失败 → 工作树 23 项；其中 22 项与 HEAD 逐项同因（合并提交既有债，非本
    change 范围）；
  - 唯一工作树新增失败为 `p3_sync_return` 的
    `independent_effects_and_unproved_nested_monitors_do_not_receive_the_exit_exemption`，
    归因于同一工作树中 build.rs 死区域豁免的未提交改动（该 fixture 类无任何浮点常量，
    浮点路径对其可证惰性）；root 审查时随死区域工作一并处置；
  - jarde-java 5 项与 HEAD 相同；jarde-cli 由 HEAD 4 项降为 0（工作树修复）。
  - `p3_required_conversions` 的 `ir_items=224` 钉在 HEAD 即已失败（HEAD 实测 239），
    工作树 concat 豁免使其实测 165；本 change 对该成员（无浮点）的 delta 为 0，未代为
    重录，留 root 处置。
- 536 项与 NaN 折叠验证使用过的临时 dump 测试已删除；`cargo fmt --all` 已执行；
  clippy（`-p jarde-java -p jarde --all-targets --all-features`，按任务口径过滤）无本
  change 新增警告。

## Root 复核（3.2，2026-09-26）

Root 在工作树（含协调者与其它簇未提交修复）独立复跑：

- `cargo test --all-features -p jarde --test p3_floating_constants`：3 passed + 1 ignored（ignored 实跑通过）；`-p jarde --test p3_unary_negation` 7 passed。全树 triage 由实施者按 HEAD `2ad29cee` 同 worktree 对照为 43→23 失败、零新增回归，root 复核了失败清单归属（其余失败均为已登记簇的合并提交既有债）。
- 拒绝来源抽查：非 canonical NaN 在自身 BCI 拒绝且失败消费者 quote 保留常量与操作坐标（实测 `@bytecode 2 0`）；canonical NaN/±∞ 走除法拼写并从常量 BCI derived。折叠债务的保存边界由 javac 折叠（`-(0.0f/0.0f)` → `7fc00000`）与 JVM `fneg`（`ffc00000`）的位差证实，root 认可其必要性。
- `cargo fmt --all --check` 干净；`cargo clippy -p jarde-java -p jarde --all-targets --all-features` 的全部警告位于 enumswitch.rs/region.rs/facade.rs/class_source.rs 等合并提交既有文件，浮点改动文件（facts/decode/ast/emit/build 浮点区）零新增。census 与 fingerprint 通过，无需重录。
- `openspec validate recover-floating-point-constants --strict`：后补复跑通过。
- 零值拼写裁决：钉子要求返回行含 `0x`，实施者取 hex 形式（javac 探针验证折叠到精确 ±0 bits）；design 未规定零格式，root 接受该裁决，留待后续可读性迭代（最短十进制往返拼写）另行评估。
