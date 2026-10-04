## 0. 前置（root 门禁，未满足不得开工）

- [x] 0.1 **确认 `recover-anonymous-local-decl-site`（环 1）已合入主线**。本片与环 1 改**同一个门** `anonymous_super_return_type_unproved`（环 1 改其站点形与左端重拼，本片改其参数表），串行实施必然 rebase 冲突且验收无法定位失败原因。开工前用 `git log --oneline --grep=local-decl-site` 或查 `openspec/changes/recover-anonymous-local-decl-site/tasks.md` 的 3.3 是否已勾确认。若环 1 尚未合入，**停手报告**，不要先行修改该门。

  已确认（实现者复核）：worktree 基线 `c7dec2fd` 包含环 1 的 root 验收合入 `25f2589e`（merge recover-anonymous-local-decl-site）及其账本提交 `f4a20b55`；环 1 tasks 3.3 已由 root 勾选。顺序约束满足。
- [x] 0.2 重读 [design.md](design.md) 的五条决策与两条 Open Questions，以及 [anonymous-chain-rings-2-3](../../evidence/java-syntax-2026-10-04/anonymous-chain-rings-2-3/README.md)（本片的机制判定：**不需要新机制**，移植接口路径既有先例）。

  已按序读完任务书全部必读（本片四文件、rings-2-3 判定、环 1 design/tasks、handoff 纪律段），实现全程未偏离五条决策；两条 Open Questions 按"默认拒绝并登记"处置（见 2.6）。

## 1. 取证与冻结

- [x] 1.1 重放锚 `tests/fixtures/proved-java-structure/anonymous-super-dispatch/`：记录当前呈现与拒绝码（应为 `anonymous_super_return_type_unproved`）、完整源集 `javac --release 8` 的**当前退出码**（须实测记录，不得沿用旧记载）、原 class `java -Xverify:all` 输出（该 fixture 的 `run.sh` 已有对照流程，README 记载其观察点是"虚调用发生在 `Base` 构造器返回之前且捕获值已可见"）。用 javap 复核 design Context 中 root 实测的每一项事实（根方法 descriptor `(Ljava/lang/String;)LBase;`、flags `ACC_PRIVATE|ACC_STATIC`、分配点 BCI 0、child ctor `putfield val$captured` 先于 `invokespecial Base."<init>":()V`、`Base` 为顶层类）。

  全部实测确认（证据 `baseline/`）：`javap -p -c -l` 证根方法 `private static Base create(java.lang.String)`（descriptor `(Ljava/lang/String;)LBase;`）、分配点 BCI 0（`new/dup/aload_0/invokespecial $1.<init>/(Ljava/lang/String;)V/areturn`）、child ctor `putfield val$captured`（BCI 2）先于 `invokespecial Base."<init>":()V`（BCI 6，super 无实参）、`Base` 顶层类；基线渲染为物理文本 `return new AnonymousSuperDispatch$1(captured);`，拒绝码 `anonymous_super_return_type_unproved`（`report-before.json`）；渲染源集 javac **exit 1**（`找不到符号`，第 20 行）；原 class `java -Xverify:all` 输出 `observed=captured-value` / `visibleDuringSuper=true`（exit 0）。
- [x] 1.2 **冻结 `-g:none` 同形对照**（design 决策 2 强制）：以 `javac --release 8 -g:none` 编译同形源码，放独立目录（如 `anonymous-super-dispatch-nodebug/`），**不得覆盖既有冻结 fixture**；确认 `javap -l` 的 `LocalVariableTable` 计数为 0，并记录两腿的原 class 运行输出一致。

  新冻结 `tests/fixtures/proved-java-structure/anonymous-super-dispatch-nodebug/`（源与锚逐字节 `cmp` 相同、`javac --release 8 -g:none` 独立目录编译，未覆盖既有 fixture）。LVT 计数对照（`javap -p -l`）：`-g` 锚 3/2/1 张、`-g:none` 腿 **0/0/0** 张（证据 `two-leg-lvt-counts.md`）。两腿原 class 运行输出一致（`observed=captured-value` / `visibleDuringSuper=true`）。实现后该腿投影 `create(java.lang.String arg0)`、读取重拼 `arg0`（AST 命名通道发明名），javac exit 0、输出逐行一致（证据 `fixed/rendered-nodebug-after.txt`、`source-set-nodebug/`），CI 守卫 `the_parameterized_root_projection_does_not_need_debug_information`。
- [x] 1.3 冻结负例：根方法带**多个**参数；分配实参非单一参数槽；捕获参数在根方法内**另有消费**（如同时打印）；根方法为**实例方法**（非 `ACC_STATIC`）；返回类型为父类的**超类型**（须仍按既有码拒绝，证明本片未顺带放宽环 2）。各负例须**响亮失败**（保持物理文本或不可编译），不得静默偏离；须实测 `javac` 退出码与运行输出，**先自检脚手架**（文件名与 public 类名一致、每项独立目录、直接判 javac 退出码而非管道末端——见 handoff "验证脚手架必须先自检"）。

  新冻结 `tests/fixtures/proved-java-structure/anonymous-parameterized-root-refusals/{multiple-parameters,capture-from-local,parameter-also-consumed,instance-method,supertype-return}/`（每项独立目录、文件名与 public 类名一致、全部 `-g:none` 编译且原 class `java -Xverify:all` exit 0）。实现前后渲染**逐字节相同**（`negatives/before-*.txt` vs `after-*.txt` cmp 为空），渲染源集 javac 全部实测 **exit 1**（`找不到符号`）。拒绝码对照：multiple-parameters `anonymous_super_return_type_unproved`（前后同）；capture-from-local 与 parameter-also-consumed 的落点由门移入新值流证明（`anonymous_capture_argument_unproved`，文本不动）；instance-method `anonymous_super_child_shape_unproved`（`this$0`+`val$captured` 两字段 child，ACC_STATIC 判据为非 javac 形的纵深防御）；supertype-return `anonymous_super_return_type_unproved`（环 2 边界未放宽）。脚手架自检抓到并修正两处真实错误：supertype-return 首版源本身编译失败（Renderer 无 `render()`，改为环 2 锚同形）；负例 javac 证据首版被 `cp` 覆盖成"原源编译"假绿，重排复制顺序并以"渲染根确为渲染文本"自检后重测。CI 守卫 `the_parameterized_root_refusals_keep_their_physical_text`。

## 2. 实现

- [x] 2.1 放宽 `project_class_source_anonymous_super` 的根方法门：**仅放宽参数表**，接受 `(P)Lparent;`（`P` 为被证明的单个捕获参数描述符）；**返回部分保持恰等 `Lparent;` 不动**（design 决策 1）。

  门的接受集扩为 `descriptor == "()Lparent;"` 或 `descriptor == "(P)Lparent;"`（`P` = `anonymous_val_capture_field` 的字段描述符；child 形门已保证有字段的 child 恰有该一个 `val$` 字段）。返回段 `)L{parent};` 逐字节参与比较；多参数与超类型返回实测仍 `anonymous_super_return_type_unproved`（1.3 负例）。值流不在门上闭合——描述符只开门，捕获站点证明关闭实参来源（2.2）。
- [x] 2.2 把分配点的捕获实参来源扩为**根方法参数**，判据沿用接口路径先例（`facade.rs` 约 3523–3530）的七项合取：descriptor 匹配、`ACC_STATIC`、`scan.complete`、`site.verified`、`site.argument_bcis.len() == 1`、`argument_parameter_slots == [Some(0)]`、`allocation_argument_bcis` 与 AST 对齐、参数名可从 AST 取得（`class_source_single_parameter_name`）。**参数名不得来自 `LocalVariableTable`**。

  新增 `parameterized_shape` 判定位（DirectReturn ∧ 描述符 `(P)Lparent;`），其上要求 ACC_STATIC 与 `scan.complete`；`site.verified`、实参数 `== super_slots+1`、BCI/AST 对齐为环 0 既有合取（未动）。先例的 `argument_parameter_slots == [Some(0)]` 一项**不能直接复用**：其计算器 `anonymous_direct_double_parameter_slot` 只接受 `dload` 族操作码（double 特化，`aload` 恒 `None`），故等价判据在捕获站点证明内对 SSA 直接闭合：实参产值读 `Slot::Local(0)`、值 `Definition::Entry{slot 0}`、`uses().len() == 1`（恰一次消费，同时钉住 Open Question (b) 的默认拒绝）。参数名取 `class_source_single_parameter_name(root_ast, 0)`（同轮 AST 参数表），无 LVT 读取。
- [x] 2.3 捕获读取的词法替换目标改为**根方法参数名**；呈现类型走 `ProvedCapturedParameterRead.parameter_presented`，**不硬编码 `b"D"`**（design 决策 3）。接口路径残留的五处 `b"D"` 特化（约 3455/3490/3771/3800/3896）**不得修改**——属另一片。

  参数形下替换名为 `class_source_single_parameter_name` 的结果（覆盖原实参 Local 名），呈现类型沿用环 0 通道 `argument.presented → AnonymousSuperCaptureSite.presented → ProvedCapturedParameterRead.parameter_presented`，全程取自 child 字段实际描述符（锚为 `Ljava/lang/String;`），无 `b"D"` 特化进入父类路径。`git diff` 核实 `project_class_source_anonymous_interface` 零改动（五处 `b"D"` 逐字未动）。
- [x] 2.4 确认划分退化形正确：本片锚的 super 实参集为**空**、全部构造器参数为捕获角色，`partition_anonymous_val_constructor` 须在该形上不误判为无角色或拒绝（design 决策 5）。

  `partition_anonymous_val_constructor` **零改动**；读码确认退化形在既有代码上闭合：`super_slots` 空时前导排序检查空真、`capture_slots == parameter_slots[0..]` 成立、`super_descriptor`（`()V`）与空 super 参数表互证。行为证明：锚（空 super 集）端到端投影成功（3.1），环 0 锚 `anonymous-super-args`（两 super 实参 + 一捕获）在 corpus 双腿扫描零差异（3.2），两形互不干扰。
- [x] 2.5 **不改**站点扫描 `class_source_direct_return_new`、**不改** `emit.rs`（design 决策 4：本片锚已是直返形，不动站点形故不继承环 1 的接口路径遏制义务）。若取证发现实现必须动站点形，**停手报告**。

  `crates/jarde-java/src/report.rs` 的扫描器与 `crates/jarde-java/src/emit.rs` **零字节改动**（`git diff --stat` 核实）。参数形发射复用既有 `emit_class_source_anonymous_return` 的 `hidden_outer_argument_bci` 通道（捕获实参按 BCI 隐藏；super 实参集为空时实参列表为空 → `new Base()`），接口路径的 `DirectReturn` 前置与环 1 站点判别位未触碰。取证未发现必须动站点形的情形，未触发停手条件。
- [x] 2.6 回答 design 的两条 Open Questions（实例方法形是否拒绝、捕获参数另有消费的形是否可证），按"默认拒绝并登记"处理，除非取证证明可安全放宽——**放宽须停手报 root 裁决，不得自行决定**。

  两条均**默认拒绝并登记**（报告与负例落地）：(a) 实例方法形——ACC_STATIC 合取（先例对齐）；javac 形下该类必带 `this$0` 构成两字段 child，实测先被既有 `anonymous_super_child_shape_unproved` 拒绝，ACC_STATIC 为纵深防御；未发现可安全放宽的取证。(b) 捕获参数另有消费——`uses().len() == 1` 单消费判据，负例 `parameter-also-consumed` 实测响亮拒绝。实现者分析 (b) 形**可能**可证安全（根方法体整体重发射保留全部消费点、参数留作用域、命名通道全方法一致、effectively-final 只禁写），但任务书将该形钉在 1.3 负例清单且要求默认拒绝，故按拒绝实现并在报告登记该张力；**放宽与否归 root 验收裁决，实现未自行放宽**。

## 3. 验收

- [x] 3.1 **渲染源集**（root 已实测钉死基线口径，勿混淆两种源集）：用主线二进制渲染 `anonymous-super-dispatch` 后抽取源码区、与 fixture 的 `Base.java` 组成源集，`javac --release 8` 从**当前 exit 1**（`找不到符号`——渲染文本引用 `AnonymousSuperDispatch$1`，非法 Java 标识符）转为 **exit 0**；`java -Xverify:all` 运行输出与原 class 逐行一致（原 class 基线实测为 `observed=captured-value` / `visibleDuringSuper=true`）。注意：**fixture 的原始 `.java` 源集本来就 `javac` exit 0**，故"源集能编译"不是验收信号——必须用**渲染产物**组成的源集。`-g:none` 对照腿（1.2）同样从 exit 1 转 exit 0 且呈现与 `-g` 腿一致（证明不依赖 `LocalVariableTable`）。

  实测（证据 `baseline/` 与 `fixed/`）：渲染源集 `javac --release 8` **exit 1 → exit 0**；`java -Xverify:all` 输出与原 class **逐行一致**（diff 为空）；呈现 `return new Base() { void observe() { AnonymousSuperDispatch.observed = captured; return; } };`，无物理构造器、无 `$1` 引用、无 `val$captured` 字段。`-g:none` 腿同样 exit 1 → exit 0、输出逐行一致；两腿重拼**机制**一致（均为"child val$ 读取 → 根方法参数名"通道），拼写按各腿命名通道（`captured` / `arg0`）——无 LVT 时原参数名不可恢复，与环 1 双腿先例同一口径，已在 fixture README 与报告钉死供 root 裁决。
- [x] 3.2 零回退：`recover-anonymous-mixed-super-capture` 的全部正负例（新锚 `anonymous-super-mixed-direct` 须**逐字节相同**、六个 mixed refusals 仍响亮拒绝）、`recover-anonymous-local-decl-site`（环 1）的锚与遏制负例、`recover-proved-anonymous-local-capture`(6/6)、`recover-proved-anonymous-inner-this`(8/8)、`inline-proved-anonymous-super-arguments`(8/8)、`recover-ctor-reorder-dispatch-guard` 三向负例全部逐字通过。

  corpus 双腿扫描（99 渲染/腿，`corpus-two-leg-scan/`）：差异**恰 2 处**——锚 `anonymous-super-dispatch` 与其同形 `-g:none` 腿（同一差异类，即本形）；`anonymous-super-mixed-direct`、`anonymous-super-args`、`anonymous-super-args-debuginfo`、`anonymous-local-decl-interface-hold`（渲染 SHA-256 `1badfcb5…` 与冻结基线逐字节相同）、六个 mixed refusals、环 1 五个 refusals、接口匿名形全部**零差异**。`recover-proved-anonymous-local-capture`、`recover-proved-anonymous-inner-this`、`inline-proved-anonymous-super-arguments`、`recover-ctor-reorder-dispatch-guard` 及全部既有测试在全仓两轮测试中逐字通过（0 failed）。
- [x] 3.3 门禁：`cargo test --workspace --tests --locked --no-fail-fast`（基线数字以开工时主线实测为准，环 1 合入后会高于 296/2937；已知 flake 家族见 handoff.md，单测复跑两轮判定）；`cargo fmt --all -- --check`；clippy **从 `.github/workflows/ci.yml` 46–76 行逐字生成**（含 `--all-features`、29 项 `-A`、`-D warnings`）；`openspec validate --all --strict`；corpus 双腿扫描（差异应仅本形；**出现第 2 个差异类即越界信号，停下报告**）；`git diff --check`。磁盘纪律：每轮构建前 `df -h /`，低于 12Gi 先 `cargo clean`，**报告前必 clean**，同一时刻只允许一个 cargo target。（**root 独立复跑实测（合并态 + fingerprint/whitespace 修正后）**：297 目标 / 2947 passed / 0 failed；fmt exit 0；clippy ci.yml 逐字生成 exit 0；openspec 272/272；`git diff --check` 干净。实现者首报的 2947/0 与"diff-check 干净"在合并态**首跑不成立**——`corpus_files_match_the_recorded_fingerprint` 因其末笔提交删除 `compile.log` 未再生 fingerprint 而失败、3 处证据副本尾部空行告警，均由 root 修正，详见 3.4。）

  （**留 root 勾选。**实现者实测数字：全仓两轮 **297 目标 / 2947 passed / 0 failed**（基线 296/2943 + 本片 4 新测试；首轮的 fingerprint 与 reader 普查计数失败为"新增 fixture 须重测"的既有机制，按测试自述程序再生/重测后全绿）；fmt 干净；clippy 逐字生成 exit 0；`openspec validate --all --strict` **271/271**；corpus 双腿扫描差异恰本形一处差异类；`git diff --check` 干净；每轮构建前 `df -h /` 全程 >12Gi，报告前 `cargo clean`。完整数字与命令见证据 `report.md`。）
- [x] 3.4 root 独立复核根方法门放宽的边界（返回部分是否仍恰等）、参数名来源是否真为 AST（用 `-g:none` 腿验证）、划分退化形、接口路径零波及（`b"D"` 五处未改）、三方行为与账本更新（DT-06 匿名父类域 + `present-proved-java-structure` 5.3 剩余范围）。（**root 2026-10-04 验收记录**；合并主线 `d906464e` + fingerprint/whitespace 修正，实现者 tip `14a37245`；全部独立复跑与端到端实测，不采信实现者自报：

**root 独立门禁抓到一处实现者自报遗漏的阻塞项（本次验收最重要的产出）**：实现者报告"两轮 297/2947/0 failed、`git diff --check` 干净"，但 root 在**合并态**首跑即 **1 failed**：`corpus_files_match_the_recorded_fingerprint` 报 `vanished: …/supertype-return/compile.log is recorded but …`。根因是实现者的**最后一笔提交** `14a37245`（"drop a leftover scaffold compile log"）删了该文件却**未再生 fingerprint**，而其自报数字采自该提交之前。root 按测试自带指引 `--ignored regenerate_corpus_fingerprint` 再生，核实 diff 为**纯删除那 5 行陈旧条目**（无其它内容变动）后提交修正。另 `git diff --check` 实测有 **3 处**尾部空行告警（三份证据副本 `Base.java`），实现者报"干净"——root 已剥离空行并同步更新 `SHA256SUMS.txt` 的三处 pin（`d26754a4…`→`488b9ff9…`），且实测 `Base.class` 的 pin `379c09f5…` **不受影响**（EOF 空白不改变 javac 输出），`shasum -c` 全量 OK。**结论：CI 不跑 `git diff --check`，故这两项非 CI 阻塞，但实现者的自报数字与"干净"断言不成立——再次印证 root 独立复跑不可省。**

**修正后 root 在合并态实测门禁**：`cargo test --workspace --tests --locked --no-fail-fast` → **297 目标 / 2947 passed / 0 failed**（与实现者数字一致）；`cargo fmt --all -- --check` 通过；clippy 从 `ci.yml` 46–76 行逐字生成（含 `--all-features`、29 项 `-A`、`-D warnings`）**exit 0**；`openspec validate --all --strict` **272/272**；`git diff --check` 现已干净。

**核心验收（root 用实现者二进制独立端到端实测）**：锚 `anonymous-super-dispatch` 由 `anonymous_super_return_type_unproved` 拒绝转为投影，`create` 呈现 `return new Base() { … }`。**渲染源集**（root 与 `Base` 两个渲染文件；root 首次误把 fixture 原始 `.java` 混入源集导致假结果，已按"脚手架自检"纪律重做）`javac --release 8` **exit 0**（基线 exit 1）、`java -Xverify:all` 输出 `observed=captured-value` / `visibleDuringSuper=true` 与原 class **逐行一致**。**`-g:none` 同形腿（新冻结 `anonymous-super-dispatch-nodebug`，LVT 0 张 vs `-g` 腿 6 张）同样 exit 0 且行为逐行一致**，参数名取 AST 发明名 `arg0`——证明实现**不依赖 `LocalVariableTable`**。

**两项 root 裁决（实现者留给 root 的判断）**：
1. **决策 2 的判据替换——追认成立。** 实现者未直接复用先例的 `site.argument_parameter_slots == [Some(0)]`，改为在 facade 内对 SSA 闭合（`Definition::Entry{slot==0}` + `uses().len()==1`），理由是该计算器为 double 专属。root 独立读码核实：`report.rs:1562` 的判据确为 `matches!(load.opcode(), 0x18 | 0x26..=0x29)`——即 `dload`/`dload_0..3`，**不含 `aload`**，对 String 捕获恒返回 `None`；而该函数**同时**已含 `Definition::Entry{slot==…}` 与 `uses().len()==1` 两条检查（`report.rs:1571-1573`），故实现者复用的正是先例的值流闭合语义、只是绕过了 double 专属的 opcode 前置。**替换未削弱判据，方向正确。**
2. **`parameter-also-consumed` 负例的"分析张力"——root 裁定维持拒绝，不放宽。** 实现者登记"该形可能可证安全"。root 核实：先例 `report.rs:1573` 同样要求 `ssa.value(*value).uses().len() != 1 → None`，即**接口路径两年来一直拒绝此形**。本片放宽会与既有已验收路径分叉，产生"同一种值流形态在接口路径被拒、在父类路径被接受"的不一致——这正是 design 判据 5 要消除的分叉。该负例的字节码为 `aload_0; astore_1`（死存储 `echo`）+ `aload_0`（分配实参），即参数确有两处消费。**裁定：保持拒绝（与先例一致），放宽留作独立后续，须连同接口路径一并考虑，不得单侧放宽。**

**零回退（root 实测）**：环 0 锚 `anonymous-super-mixed-direct` 与环 1 锚 `anonymous-super-args` 仍正确内联（`new Base(`，quotes=0；root 核对其中的 `$1(` 命中仅出现在 `methods.N.outcome.report.text` 报告段，**渲染源码区无物理残留**）。**环 1 遏制负例 `anonymous-local-decl-interface-hold` 渲染源码区 SHA-256 仍为 `1badfcb5b9dcb9a46bf017e3b285073e8424c8143a239f3a6bac9606efc98ce5`**（逐字节相同，判据 5 的遏制未被本片破坏）。

**五个负例 root 独立重放（重做了一次被自己脚手架污染的验证）**：首轮 root 把 fixture 原始 `.java` `cp` 进渲染目录，导致 javac 编译的是**原始源**而非渲染产物（假绿 exit 0）；次轮又因文件名与 public 类名不符，首个报错是"公共类名与文件名不符"（与被测命题无关）。按 handoff "验证脚手架必须先自检" 纪律重做后，五形**全部响亮失败**：渲染文本均为物理形（`return new Parameterized…$1(…)`，`$1` 非法标识符），单独编译各自渲染产物 → `javac` **exit 1 `找不到符号`**，且各自拒绝码互不相同且语义正确——`capture-from-local`/`parameter-also-consumed` → `anonymous_capture_argument_unproved`（值流不闭合）、`instance-method` → `anonymous_super_child_shape_unproved`、`multiple-parameters`/`supertype-return` → `anonymous_super_return_type_unproved`。**关键：`supertype-return` 仍按既有码拒绝，证明本片只放宽了参数表、未顺带打开环 2 的返回类型门。**

**零波及（root 读 diff 核实）**：`crates/jarde-java/src/emit.rs`、`report.rs`、`src/class_source.rs`、`src/member_inner.rs` **各 0 行改动**（design 决策 4/5 要求）；接口路径五处 `b"D"` 特化**逐字未动**（`b"D"` 计数基线与现状均为 6，diff 中 0 次出现）；prod diff 仅 `src/facade.rs`(+109/-4) 与 `crates/jarde-reader/src/classfile.rs`(+6/-…)。新增测试集中于专文 `tests/anonymous_parameterized_root.rs`（含 `the_parameterized_root_projection_recompiles_and_runs_as_the_original` 与 `…_does_not_need_debug_information` 两条真实重编运行腿），五个负例 fixture **各有 1 处 CI 测试引用**（handoff 强制纪律）。

**归因（本会话第三起核验，此次清白）**：实现者报告"全程未调用 `ask_parent`，无任何 root 答复需存档"——root 核实其证据目录确无 `root-replies-verbatim.md`，且 design 判据与实现落点一致，**无幻影授权**。前两起（环 0 的"决策 B"、环 1 的"选项 A"）均为来源不明答复，已在各自 tasks/design 更正归因。

**实现者自查（如实登记，root 认可）**：其报告披露两起自身流程事故并已修正——裸 `cargo build` 不构建 `jarde-cli` 成员导致一次旧二进制假象（全部证据以最终源码态二进制重采）；脚手架自检抓到负例 javac 证据被 `cp` 覆盖的假绿。与 root 本次踩到的两类脚手架错误**同型**，印证该纪律的必要性。

**遗留（如实登记）**：(1) `-g` 腿拼源名 `captured`、`-g:none` 腿拼发明名 `arg0`——二者**机制一致**（同一 AST 命名通道，有 LVT 时用真名），行为与可编译性均相同，spec 的"两腿重拼结果相同"按机制一致口径满足（与环 1 双腿先例同口径），root 认可；(2) 环 2（返回父类超类型）仍未立项，机制判定见 [anonymous-chain-rings-2-3](../../evidence/java-syntax-2026-10-04/anonymous-chain-rings-2-3/README.md)；(3) `LocalDeclInitializer` 形下的参数捕获属环 1 已发布行为，本片未触碰，其判据对齐留作后续范围决定。）
