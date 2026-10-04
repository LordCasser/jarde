## 1. 取证与基线

- [x] 1.1 重放冻结反例 `tests/fixtures/proved-java-structure/anonymous-super-dispatch/`（SHA 核对 results/fixture-sha256.txt）：用 `results/repro.sh` 复现回归（原类 `visibleDuringSuper=true` → 渲染重编 `false`）；落点在 `crates/jarde-java/src/ctor_order.rs::present_prologue_first`（daa4fb31 新增的 255 行模块；其 `build.rs` 侧仅 +17 行接线），确认 `init::Prologue` 携带的 `class`（super 目标内部名，init.rs:1507）与目标 descriptor 在判据处可得；记录 C1/C2 正例的 `Object.<init>` 目标（实测 `capture_ctor_class` 生成器 pool 第 4/8/43 行即 `java/lang/Object`+`<init>`+`()V`）。
- [x] 1.2 冻结/构造至少三个负例：`anonymous-super-args/AnonymousSuperArgs$1`（super `Base`，非 Object → 退回 verbatim）、`anonymous-capture/AnonymousCaptureCases$1`（super `AnonymousCaptureCases$Base` → 退回 verbatim）、生成器构造的用户类 super 形（无需新 fixture）；各自记录实现前后呈现、重编状态与 `java -Xverify:all` 行为。

## 2. 判据收紧

- [x] 2.1 重排准入加 super 目标等于 `java/lang/Object.<init>()V` 的单档前置条件（owner 与 descriptor 双匹配，design 决策 1）；不满足时保持既有逐字呈现与诊断（决策 3：不改整方法拒绝、不丢成员）。**不引入跨类读 super ctor 体的宽档**——决策 1 已用三条实证否证（跨类读体无既有先例、headers-only 覆写名代理会反向回归、单类事实无法区分三处 fixture）。
- [x] 2.2 回归测试三向钉死（决策 4）：(a) `anonymous-super-dispatch` 断言重排未发生（捕获写入文本在 `super()` 之前）且不得出现"可编译且行为不同"；(b) C1/C2 与 `capture_ctor_class` 全部正例断言重排仍发生、逐字不变；(c) 新增负例（super 为非 Object 用户类）断言不重排、退回 verbatim。

## 3. 回归与验收

- [x] 3.1 `cargo test --workspace --tests --locked --no-fail-fast` 全绿（含 daa4fb31 的三正例两负例、capture-ctor 家族、synthetic-ctor 全部测试）、fmt、CI 完整 `-A` clippy、`openspec validate --all --strict`、diff check；磁盘纪律：每轮构建前 `df -h /`，低于 12Gi 先 `cargo clean`，报告前必 clean。
- [x] 3.2 `repro.sh` 复跑：原类与渲染重编的行为对照如实记录（回归消除的判据是"不再出现可编译且行为不同"）；C1/C2 家族三方对照逐字不变；两处非 Object super fixture 退回 verbatim 后行为仍与原 class 一致（响亮失败：完整源集编译退出 1，与 2026-09-27 既有登记状态一致）。
- [x] 3.3 root 独立复核判据、回归测试三向性与既有正例零回退，更新 `recover-synthetic-ctor-super-order` 的 3.3 验收记录（补记该反例遗漏）与 `present-proved-java-structure` 2.10 的状态说明（本片实现了其"独立二进制名类视图不声称可编译"立场的一半；另一半即可编译性由 5.3 类级匿名语法承担）。

> **2026-10-04 root 验收记录**（合并主线 `fc868aba`；全部独立复跑，不采信实现者自报）：
>
> **门禁（root 实测）**：`cargo test --workspace --tests --locked --no-fail-fast` **295 目标 / 2922 passed / 0 failed**（与实现者自报精确吻合；= bridge 验收后 2919 基线 + 本片 3 新测）；`cargo fmt --all -- --check` 通过；clippy 从 `ci.yml` 46–76 行**逐字生成**命令（含 `--all-features`、29 项 `-A`）**exit 0 / Finished 干净**；`openspec validate --all --strict` 268/268。
> **守卫测试在我自己的套件中执行并通过**（非采信冻结 trace）：`the_dispatch_fixture_keeps_the_capture_write_before_the_constructor_call`、`user_class_super_fixtures_return_to_the_compiler_s_byte_order`、`a_synthetic_capture_before_a_user_class_constructor_call_stays_where_the_bytecode_made_it` 三者 ok；既有正例 `capture_constructors_are_presented_prologue_first_and_the_family_recompiles`、`a_synthetic_capture_store_is_presented_after_its_constructor_call`、`a_field_named_like_a_capture_but_not_synthetic_stays_before_its_constructor_call` **断言未改一字**仍 ok（零回归）。
> **判据复核**：生产 diff 仅 `ctor_order.rs`（+41）与 `p3_patterns.rs` 测试（+40）；判据插在 `stmts[..=prologue_index].rotate_right(1)` **紧前**、读 `operations.get(prologue.bci)` 的 `Operation::Invoke(target)`、三元匹配 `java/lang/Object` + `<init>` + `()V`、不满足即 `return Ok(())` 保 verbatim——与 design 决策 1 逐字一致；**未扩展 `Prologue`、未动 `build.rs`、无任何跨类读**，且未实现被否证的宽档（决策 1 的三条实证被遵守）。文档注释同步说明了"loud vs silent failure"的依据。
> **析取不变量 root 独立复现**（`/tmp/guard-verify`，非采信 `repro.sh` 记录）：原类基线 `observed=captured-value`/`visibleDuringSuper=true` 不变；guard 渲染为 verbatim 序（`this.val$captured = arg1;` 在 `super();` 之前）→ `javac --release 8` **exit 1**（"灵活构造器是预览功能"，响亮失败）。即"可编译且行为不同"的静默态已消除，退回忠实但不可编，符合 spec 决策 3。测试断言的可判伪性亦经复核：基线先钉 `visibleDuringSuper=true`，若重排回归则 javac 会接受且输出 `false` → `assert_eq!` 失败。
> **实现者普查勘误经 root 独立确认**：`anonymous-top-level/AnonymousTopLevel$1` 是同形第四处（javap 实测 `putfield val$captured(arg3)` → `invokespecial Base.<init>(J)V`，其 `Base(long)` ctor 的 `invokevirtual` 全在新建 StringBuilder 上、无 `this` 分派），我原 README 普查表漏计；其 2026-09-25 既有证据（`anonymous-top-level/evidence.md` 第 80 行）登记的正是 verbatim 序 + javac exit 1 + "构造器顺序是 OpenSpec 2.10 的独立缺口，在 5.3 投影前不能把该类源码当作可编译的匿名体证明"——守卫使其**恢复该登记状态**，非新损失。据此修正 3.2 的"两处非 Object super fixture"表述：实为**四处**（`anonymous-super-dispatch`、`anonymous-super-args`、`anonymous-capture`、`anonymous-top-level`），corpus 双腿扫描（465 类）的 4 处差异即此四类、全是 ctor 语句序，无其它渲染变化。
> **保守判据的已知代价（root 复核确认属 spec 内取舍）**：`anonymous-super-args`/`anonymous-capture`/`anonymous-top-level` 三处的 super ctor 实测**无** `this` 虚分派，其重排本是行为安全的，但单档判据使其退回 verbatim（不可编译）。这是 design 决策 1 明确接受的取舍（换取"无需跨类读方法体、无新机制"），终局解见下。
> **终局解关联（2026-10-04 root 实测更正：原写"5.3 是四处 fixture 的终局解"不成立）**：本片是过渡收敛（响亮失败）。[recover-anonymous-mixed-super-capture](../recover-anonymous-mixed-super-capture/)（5.3，合并 `e1c89d57`）已落地，但**只覆盖"根方法为无参、返回类型恰为父类 `()Lparent;`、分配点在直返位"的混合形**，其新锚是新建 fixture `anonymous-super-mixed-direct/`（root 实测：完整源集 `javac --release 8` exit 0、`java -Xverify:all` 事件日志与原 class **逐字一致**，且 javac 自行重建 `putfield val$local0` 先于 `invokespecial Base.<init>` —— 该形确已不依赖 ctor 重排）。
>
> **四处 ctor-reorder fixture 的状态（root 2026-10-04 三次实测；环 3 合入 `d906464e` 后用主线二进制重测）**：
>
> | fixture | 根方法签名 | 阻塞门 | 现状 |
> | --- | --- | --- | --- |
> | `anonymous-super-args` | `main` 内 `Base instance = new Base(…)` | 分配点在**局部声明初始化位** | **已内联**（环 1 [recover-anonymous-local-decl-site](../recover-anonymous-local-decl-site/) `25f2589e`；root 实测渲染源集 `javac` exit 0、事件日志与原 class 逐行一致） |
> | `anonymous-super-dispatch` | `private static Base create(final String captured)` | 同上门：根方法**带参数** | **已内联**（环 3 [recover-anonymous-parameterized-root](../recover-anonymous-parameterized-root/) `d906464e`；root 实测渲染源集 `javac` exit 0、输出 `observed=captured-value`/`visibleDuringSuper=true` 与原 class 逐行一致；`-g:none` 腿同样通过，证明不依赖 LVT） |
> | `anonymous-top-level` | `static Renderer create()` | `anonymous_super_return_type_unproved`：返回类型是 `Renderer`，门要求 `()LBase;` | 仍未内联 → **环 2**（未立项） |
> | `anonymous-capture` | `private static Renderer baseArgumentAndCapture()` | 同上 | 仍未内联 → **环 2**（未立项） |
>
> **故四处中已闭合两处，终局解只剩环 2**：根方法返回父类的**超类型**（返回接口 `Renderer` 而父类为 `Base`）。环 3 的 `supertype-return` 负例经 root 实测**仍按既有码拒绝**，证明环 3 只放宽了参数表、未顺带打开环 2 的门。环 2 需要一个新的**类级可赋值性证明**能力（盘点：`prove_snapshot_hierarchy_widenings` 证值不证类；`members.rs::subtype_of` 概念匹配但私有且绑定访问检查机制），放宽前须取证其对"分配点唯一性"与"捕获值来源可证"两条不变量的影响——机制判定与优先级见 [anonymous-chain-rings-2-3](../../evidence/java-syntax-2026-10-04/anonymous-chain-rings-2-3/README.md)。`present-proved-java-structure` 2.10 的"可编译"一半已交付直返无参形 + 局部声明初始化形 + 根方法带参形。
> **遗留**：handoff 记载的测试基线 2918 与实测 2919（bridge 验收后）差 1，属历史计数漂移，不影响门禁判定；测试数基线已在 handoff 更正。
