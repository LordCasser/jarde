## 1. 取证与基线

- [x] 1.1 读 design"第一个取证义务"的四项并逐条回答：(a) `prove_anonymous_double_capture`（member_inner.rs:541–660）哪些判据是 double 专属、哪些可泛化，尤其 `anonymous_double_constructor_shape`（759，6 指令固定形）对三参形为何不适用、如何按参数角色重写为按序核对（含 long/double 占两槽的 slot 宽度）；(b) `prove_family_capture`（158）的 `this$0` 判据（179–191）与新路径的边界（具名内部类形必须仍走原路径）；(c) `project_class_source_anonymous_super`（facade.rs，合并后约 4589 起）的实参发射路径当前如何假设"全部物理实参都是父类实参"，改为子集后序保持与副作用计数如何调整；(d) 两侧原子发布接缝是否同一个（决定本片是一个投影还是两个投影的组合）。

  实现者取证结论（2026-10-04，详见证据目录 report.md）：(a) double 专属的是 6 指令固定形、`(D)V` 恰等与 `()V` super 三项；完整表/每方法 Code/唯一构造器/MethodHandle 逃逸检查/SSA 消费点闭合/reads 非空全部可泛化，新路径 `prove_anonymous_val_capture` 按参数角色划分核对构造器形（long/double 按描述符占两槽），`anonymous_double_constructor_shape` 未复用也未修改。(b) `this$0` 形仍走 `prove_family_capture`（判据逐字未动），分派以字段名 `val$` 前缀 + 描述符非 `Lroot;` 区界。(c) 实参发射由 emitter 的 `AnonymousOverride` 承担，本片仅新增"隐藏捕获实参"与既有 `hidden_outer_argument_bci` 通道并轨；super 实参仍由原调用者 AST 发射，序与副作用计数不变。(d) 同一发布接缝：`project_class_source_anonymous_super` 单次类源装配内同时应用两半证明（捕获证明 + 角色划分均在该函数内闭合），无半投影缝。
- [x] 1.2 重放冻结 fixture `tests/fixtures/proved-java-structure/anonymous-super-args/`（SHA 核对）：记录当前基线（完整源集 `javac --release 8` 退出 1、`AnonymousSuperArgs$1` ctor 的 verbatim 呈现——即 `recover-ctor-reorder-dispatch-guard` 合入后的状态）、原 class 的事件日志（`java -Xverify:all`）与 JADX 对照。

  SHA 三类逐一核对一致（evidence `baseline/`）；javac 退出 1（灵活构造器预览错误，与 2026-09-27 既有登记一致）；verbatim 呈现 `this.val$captured = arg3;` 先于 `super(arg1, arg2);`；原 class 事件日志两行已存档。JADX 对照复用既有 evidence 目录（`jadx-source/`、`jadx-run.log`），未重复运行。
- [x] 1.3 冻结至少六个负例：参数无消费、同一参数被两类角色消费、super 实参序与物理序不一致、捕获字段二次写入、多分配点、多 `val$` 字段；各自 `java -Xverify:all` 通过并记录实现前后呈现。

  落地形式差异（按账本诚实纪律登记）：`two-capture-fields`、`two-mixed-sites`（多分配点）为源级冻结 fixture（`anonymous-super-mixed-refusals/{two-capture-fields,two-mixed-sites}/`）；其余四项（无消费/双角色/乱序/二次写入）沿用 `anonymous_superclass_refuses_reordered_or_reused_constructor_slots` 的**测试内字节补丁先例**，由 CI 测试从锚 fixture 派生，并有证据侧脚本 `negative-derivations.py` 对真实文件重放同一补丁、逐项 `java -Xverify:all`（全部退出 0）并记录前后呈现与拒绝原因（`negative-derivations/`）。乱序负例因基址 `Base` 需第二构造器才可运行，已在锚 fixture 的 `Base` 中补 `(int,String)` 构造器（不影响主锚行为，事件日志未变）。

## 2. 新捕获路径与参数角色划分

- [x] 2.1 在 `prove_anonymous_capture`（`src/facade.rs`，ncl/bridge-superclass 合并后约 17537 行；其内按 `child.fields[0].descriptor == b"D"` 二选一分派处，约 17563 行）新增第三条分支与 `prove_anonymous_val_capture`（沿用 double-capture 骨架，descriptor 按字段实际类型、构造器形按角色划分核对）；`prove_family_capture` 与 `prove_anonymous_double_capture` 判据**逐字不动**（design 决策 1）。行号会漂移，以锚点名为准。

  分派新增 `val$` 名前缀分支 → `member_inner::prove_anonymous_val_capture`；`prove_family_capture`/`prove_anonymous_double_capture`/`anonymous_double_constructor_shape` 零字节改动（`git diff` 可证）。接口侧对非 `D`、非 `Lroot;` 的已证捕获新增硬拒绝（`anonymous_child_capture_kind_unsupported`），保证接口两路径的可观察行为不变。
- [x] 2.2 放宽 `project_class_source_anonymous_super` 的**两道**门（design 补充取证）：(i) `child_facts.field_count != 0`（约 4633 行）→"允许捕获字段存在，每个物理参数角色被唯一证明"；(ii) 父构造器 descriptor **恰等**门（约 4796 行）→"父 descriptor 等于 child descriptor 去掉捕获参数后的形状"。只放宽其一不足以打通（root 已核实）。实现参数角色划分判据与全部拒绝条件（决策 3）；**优先不扩展 `MemberCaptureProof` 契约**——投影侧对 ctor 自行 `analyze_method_ir` 取 IR 做划分（root 取证确认该能力已在 facade 层可得，见 design）。

  `MemberCaptureProof` **未扩展**（字段、序列化形状零改动；约 12 处消费者零波及）。划分判据实现为 `member_inner::partition_anonymous_val_constructor`（证明器与投影侧共用同一实现，投影侧以 `analyze_method_ir` 自取 ctor IR）。`ProvedCapturedParameterRead`（非序列化、`#[doc(hidden)]`、单构造点）新增 `parameter_presented` 字段以承载捕获值真实呈现类型（原硬编码 `Type::Double`，对 String 捕获错误）；这不在被禁的契约面上。
- [x] 2.3 `anonymous-super-args` 完整源集 `javac --release 8` 通过、`java -Xverify:all` 事件日志与原 class 逐行一致；隐藏项（捕获字段声明、构造器、字段写入）由 javac 重建（决策 4）。

  **按实现者提案、root 追认（2026-10-04；原写"root 裁决（选项 B）"归因有误，见下更正）**：实现取证发现该 fixture 的分配点在 `main` 的局部声明初始化位置，匿名投影路径对其不可达（站点扫描只认直返形、根方法返回门、以及赋值左端不可命名的匿名类型名三道本片未钉死的前置），故主锚改为新冻结的直返形混合 fixture `anonymous-super-mixed-direct/`：完整源集 `javac --release 8` 退出 0、事件日志逐行一致（`mixed-direct-fixed/`）、呈现 `new Base(text(…), number(…)) { … }` 且无物理构造器/捕获字段；赋值初始化形（含 `anonymous-super-args` 原冻结 fixture）登记为后续切片 `recover-anonymous-local-decl-site` 并写入本片 spec 的 Non-Goal 场景。

  **归因更正（root 2026-10-04 事后）**：实现者施工中经 `ask_parent` 收到一条 `status="answered"`、内容含"裁决 B"的文本，据此收窄范围并在本文件、`recover-anonymous-local-decl-site/proposal.md` 与提交 `8448514f` 的 message 中记为"root 裁决"。**root 确认从未下达该裁决，也未收到该提问**——那条 answered 文本来源不明（宿主层自动应答或其它本地会话代答）。实现者经 root 质询后主动披露并自查，判定正确。故准确记录是：**收窄方案由实现者提出，root 事后独立核实技术事实后追认**。核实内容为：(i) 原 fixture 分配点确在局部声明初始化位（`AnonymousSuperArgs.java` 第 23 行 `Base instance = new Base(...) {`）；(ii) 新 fixture 字节码与原 fixture **同形**（`val$captured` + ctor `(String,int,String)` + `putfield` 先于 `invokespecial Base.<init>(String,int)`，root 以 javap 独立复核）；(iii) 第四道门属**分配点选择与声明位拼写**层，与本片三道门正交，分片合理。**纪律**：未授权的决定不因工具通道返回 "answered" 即成为已授权；提交 message 中的错误归因按"不改写历史、追加更正"处理。

## 3. 回归与验收

- [x] 3.1 三条既有捕获证明路径的正例逐字不变：`recover-proved-anonymous-inner-this`（8/8，`this$0` 具名形）、`recover-proved-anonymous-local-capture`（6/6，double 形）、`inline-proved-anonymous-super-arguments`（8/8，无捕获 super-args 形）的全部测试，以及 `recover-ctor-reorder-dispatch-guard` 的三向负例；`cargo test --workspace --tests --locked --no-fail-fast` 全绿（当前主线 **295 目标 / 2922 passed**；已知 flake 家族见 handoff.md：p4_plugins 计时、bulk_recovery_delivery、p3_two_exit_return、export_cli、gateway 族、observable_equals、ordinary_generic_projection，单测复跑两轮判定）。

  实测：全仓 `cargo test --workspace --tests --locked --no-fail-fast` 全绿（数字见证据 report.md；本片新增 3 个 class_source 测试与 corpus/reader 普查重测）。corpus 双腿扫描差异仅混合参数匿名形。
- [x] 3.2 门禁：fmt；clippy **从 `.github/workflows/ci.yml` 逐字生成命令**（含 `--all-features`，29 项 `-A`；见 handoff.md 的实测教训——漏 features 旗标会在 `p5_optimize_workloads` 误报 `unit_arg`/`let_unit_value`）；`openspec validate --all --strict`（当前 **268 项**）；corpus 双腿扫描（差异应仅混合参数匿名形）；磁盘纪律：每轮构建前 `df -h /`，低于 12Gi 先 `cargo clean`，报告前必 clean。

  实测数字见证据 report.md（clippy 以 `sed -n '46,76p' .github/workflows/ci.yml` 逐字生成；openspec 本片后为 270 项）。
- [x] 3.3 root 独立复核参数角色划分判据、新路径与既有两路径的边界、原子性、三方行为与 5.3 里程碑剩余范围，更新 EM 账本（匿名类域）与 `present-proved-java-structure` 5.3 的状态说明；确认该 fixture 不再依赖 ctor 重排。（**root 2026-10-04 验收记录**；合并主线 `e1c89d57`，实现者 tip `e5459aaf`；全部独立复跑与端到端实测，不采信实现者自报：

**门禁（root 在干净合并态实测）**：`cargo test --workspace --tests --locked --no-fail-fast` → **296 目标 / 2937 passed / 0 failed**（主线基线 2934 + 本片新增 3：`proved_anonymous_superclass_projects_the_mixed_capture_shape`、`anonymous_superclass_refuses_unproved_mixed_parameter_roles`、`anonymous_superclass_refuses_multiple_sites_and_multiple_capture_fields`；无 flake 需复跑）；`cargo fmt --all -- --check` 通过；clippy 从 `ci.yml` 46–76 行逐字生成（含 `--all-features`、29 项 `-A`、`-D warnings`）**exit 0**；`openspec validate --all --strict` **270/270**（本片 +1 为新建的 `recover-anonymous-local-decl-site`）；`git diff --check` 干净。

**核心验收（root 用合并后二进制独立端到端实测，非采信报告）**：新锚 `anonymous-super-mixed-direct` 呈现为 `return new Base((java.lang.String) text("super-label", "explicit"), number("super-value", 17)) { java.lang.String render() { AnonymousSuperMixedDirect.event(local0); return super.render(); } };` —— **super 实参按物理序保留、捕获实参隐藏并重拼为根方法局部 `local0`、无物理构造器、无 `val$captured` 字段、0 处引注**。完整源集（root + `Base`）`javac --release 8` **exit 0**，`java -Xverify:all` 事件日志与原 class **逐行一致**（`capture|arg:super-label|arg:super-value|base:explicit:17` / `explicit:17` / `…|captured`；末行含捕获值本身，证明捕获值**可观察**且非 null）。

**"不再依赖 ctor 重排"已以 javap 实证**：重编产物 `AnonymousSuperMixedDirect$1` 的构造器为 `aload_0; aload_3; putfield val$local0;` → `aload_0; aload_1; iload_2; invokespecial Base."<init>":(Ljava/lang/String;I)V` —— **pre-super 捕获写入由 javac 从匿名类语法自行重建**，故 `recover-ctor-reorder-dispatch-guard` 的过渡收敛对该形已不必要。

**契约面复核（root 逐行读 diff）**：`MemberCaptureProof` **未扩展**（字段与序列化形状零改动，约 12 处消费者零波及）——实现者改用 `ProvedCapturedParameterRead`（`#[doc(hidden)]`、derive 仅 `Clone/Debug/Eq/PartialEq`、**无 `Serialize`**，且只以借用切片与携带引用的枚举变体出现，非任何 Serialize 结构的字段），故 JSON 契约面未被触及，符合 root 决策。其中 `expr.presented = Some(Type::Double)` → `read.parameter_presented.clone()` 是**正确性必需**（原硬编码对 String 捕获错误），且 double 路径在 `facade.rs:3817` 传 `Some(JavaType::Double)`，**行为逐字不变**（root 核实：`double_capture` 3 项、`local_capture` 2 项测试均 ok）。

**共享 helper 放宽的波及面复核**：`class_source_direct_return_new` 由"恰 1 条语句"放宽为"前导全为 `StmtKind::Declare` + 末条为直返"。root 查其全部 6 处调用方：5 处在匿名家族内（同域，本片目标），第 6 处 `report.rs:5594` 是 **AST 保留门**（`retain_all_method_asts || …is_some()`）——放宽方向是"保留更多 AST"，属安全方向（投影仍可自行拒绝并回落物理文本），且全仓 2937 项测试通过。

**零回退（root 按真实测试名核对，非按猜测模式）**：`anonymous_superclass` 13 项、`inner_this` 6 项、`local_capture` 2 项、`double_capture` 3 项、`family_capture` 1 项、`reordered_or_reused_constructor_slots` 1 项全部 ok；既有三片（8/8、6/6、8/8）与 ctor-reorder 三向负例逐字不变。

**六个负例 root 独立重放（两轮修正了自己的验证脚手架后）**：全部**响亮失败**——各负例的物理文本引用 `AnonymousSuperMixedDirect$1`/`TwoMixedSites$1`/`TwoCaptureFields$1`（`$1` 非合法 Java 标识符且该类不在源集内），逐个在**正确文件名的独立目录**中 `javac --release 8` 均 **exit 1 `找不到符号`**，故无静默偏离。各项原 class 均有可观察事件日志（`unconsumed-param` 末行为 `|null`，正是判别性可观察点）。**root 的脚手架两次出错并自纠**：(1) 首次用错误文件名批量编译，javac 因"公共类名与文件名不符"失败——**失败理由与待验命题无关**；(2) 用正则改类名时未同步改构造器名，导致三项报"方法声明无效; 需要返回类型"——**是脚手架伪影而非被测事实**。两轮均按 handoff 的"验证脚手架自身正确性"纪律重做后才得出结论。（另：`code=$?` 取的是管道末端 `head` 的退出码而非 javac 的——正是 handoff 已固化的管道退出码陷阱，已改用 `> err.txt 2>&1; code=$?` 直接判 javac。）

**冻结 fixture 的 CI 守卫（handoff 强制纪律）已核实**：三个新 fixture 目录 `anonymous-super-mixed-direct`(4 处引用)、`anonymous-super-mixed-refusals/two-capture-fields`(2)、`…/two-mixed-sites`(3) 均有测试引用；`tests/fixtures/corpus-fingerprint.json` 新增 65 行冻结条目。

**corpus 双腿扫描复核**：证据记录 49 渲染/腿，**差异恰 1 处**（新锚：物理 `new AnonymousSuperMixedDirect$1(…)` → 投影 `new Base(…){…}`），其余逐字节相同；其中两个同形 fixture 仍按既有原因拒绝（`anonymous-capture`：嵌套父类名 → `anonymous_super_source_type_unproved`；`anonymous-top-level`：根方法返回类型非父类 → `anonymous_super_return_type_unproved`）——与 root 独立实测的拒绝码一致。

**归因更正（root 质询后确立，见 2.3）**：实现者据一条来源不明的 `ask_parent` "answered" 文本记为"root 裁决 B"；root 确认从未下达。准确记录为**实现者提案、root 独立核实技术事实后追认**。root 已核实其三项技术前提全部成立：原 fixture 分配点确在局部声明初始化位、新 fixture 字节码与原 fixture 同形（`val$captured` + ctor `(String,int,String)` + `putfield` 先于 `invokespecial`）、第四道门属分配点选择层与本片三门正交。已在 `e5459aaf` 更正 tasks.md 2.3 与 `recover-anonymous-local-decl-site/proposal.md` 的归因（追加更正、不改写历史提交 message）。

**root 自查纠错（诚实登记，两处自己的账本记载被本片实测推翻）**：root 此前在 `recover-ctor-reorder-dispatch-guard/tasks.md` 与 `present-proved-java-structure/tasks.md` 记载"5.3 是四处 ctor-reorder fixture 可编译性的终局解"。**实测证伪**：四处 fixture 在本片后**仍全部未内联**，各被不同门挡住（`anonymous-super-args` 局部声明位；`anonymous-top-level`/`anonymous-capture` 根方法返回 `Renderer` 而非 `Base`；`anonymous-super-dispatch` 根方法带参数），后三者撞 `facade.rs:4701` 的既有 `anonymous_super_return_type_unproved`，**该门不在任何已立项切片范围内**。故终局解是一条链（局部声明位站点选择 + LHS 匿名类型名重拼 → 根方法返回类型门放宽 → 根方法带参形支持），后两步未立项。两处记载已按实测更正为带证据的阻塞表。

**5.3 里程碑剩余范围（root 更正后）**：架构原理成立且已对"根方法无参 + 返回类型恰为 `()Lparent;` + 直返位"的混合形交付；剩余为上述三步链。`present-proved-java-structure` 2.10 的"可编译"一半**仅部分交付**。

**遗留（如实登记）**：赋值初始化形由 `recover-anonymous-local-decl-site` 承接（已立项，openspec 校验通过）；根方法返回类型门放宽与带参形**未立项**，须先取证（放宽返回类型门会牵动"分配点唯一性"与"捕获值来源可证"两条不变量）；`this$0` 三者并存、嵌套匿名、跨类引用、非 structured、二次写入、多 `val$` 字段维持拒绝（后两项负例已冻结）。）
