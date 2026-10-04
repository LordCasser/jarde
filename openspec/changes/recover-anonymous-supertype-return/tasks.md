## 0. 前置（root 门禁，未满足不得开工）

- [x] 0.1 **确认环 0/1/3 均已合入主线**（`e1c89d57` / `25f2589e` / `d906464e`）。本环改的门与环 3 改的是**同一处**（环 3 刚把它条件化为 `plain_return`/`capture_return`），串行实施会 rebase 冲突。用 `git log --oneline --grep=parameterized-root` 或查环 3 的 tasks 3.4 是否已勾确认。若未合入，**停手报告**。

  已确认（实现者复核）：基线 `27beca4b`，三环 merge 均为其祖先（`git merge-base --is-ancestor` 实测），环 3 tasks 3.4 已由 root 勾选。顺序约束满足。
- [x] 0.2 重读 [design.md](design.md) 的六条决策、五条不变量、两条 Open Questions，以及 [anonymous-chain-rings-2-3](../../evidence/java-syntax-2026-10-04/anonymous-chain-rings-2-3/README.md) 的设施盘点（**无现成的类级可赋值性证明件**，故本环 MVP 只做一层、复用 `parent_read`）。

  已按序读完任务书全部必读（本片四文件、rings-2-3 判定、环 3 design/tasks、环 1 design、handoff 全部纪律段），实现全程未偏离六条决策；两条 Open Questions 处置见 2.4 与报告 §八。
- [x] 0.3 **重验行号锚点**：design 引用的 `facade.rs` 约 4735–4757（返回段判据）与约 4758（`resolve_class_source_dependency_read_raw`）在环 3 合入后的位置——本会话 facade.rs 已两次因相邻片漂移 100+ 行，**不得照抄行号**，以锚点名（`anonymous_super_return_type_unproved`、`plain_return`/`capture_return`、`resolve_class_source_dependency_read_raw`）定位。

  实测：门在 4735–4757、`parent_read` 解析在 4758——与 design 记载一致（以锚点名重定位后核行号），未触发停手条件。

## 1. 取证与冻结（**1.1 是阻塞项，须在写任何生产代码前解决**）

- [x] 1.1 **复核 root 已裁定的 Open Question**（design 已实测钉死，勿重复取证，但要独立复现一次）：环 3 已冻结的负例 `anonymous-parameterized-root-refusals/supertype-return` 与本环锚形高度相似……**若你的复现与 root 裁定不符，停手报告**……若相符：**它是范围变更**……你不得自行修改环 3 的 spec 文件，须在报告中提出，由 root 验收时裁决执行。

  **复现相符，未触发停手。** javap 四项事实全部独立核实（证据 `baseline/javap-supertype-return-{child,root}.txt`）：child `super_class`=`Base`（顶层无 `$`）；`Base implements Renderer`（接口数 1，一层直接）；三名均不含 `$`；根方法 `(Ljava/lang/String;)LRenderer;`、BCI 0 直返。fixture 注释自证 "ring 2 boundary … this slice widens only the parameter table, never the return type"，与本环锚的唯一区别确为"根方法带一个捕获参数"。**范围变更已提出**：该 fixture 落地后为 `(P)+LT;` 对照正例，实测端到端投影且不依赖普查允许臂（证据 `fixed/contrast-*`）；环 3 的 spec/tasks/fixture README 行/账本/测试归属留 root 裁决执行（本片对 `tests/anonymous_parameterized_root.rs` 仅做最小事实性更新：该 fixture 移出拒绝数组并在注释指明去向）。
- [x] 1.2 重放锚 `anonymous-top-level`：记录当前拒绝码（应为 `anonymous_super_return_type_unproved`）、当前呈现、渲染源集 javac 当前 exit 1、原 class `java -Xverify:all` 事件日志基线。用 javap 复核 design Context 的每项事实。注意渲染源集须含全部相关类。

  全部实测确认（证据 `baseline/`）：拒绝码与文本同 design 所载（`report-before.json`：state=refused、reason 逐字一致）；呈现 `static Renderer create() { … return new AnonymousTopLevel$1(choose(), captured); }`；渲染源集（渲染后的 `AnonymousTopLevel`+`Base`+`Renderer` 三个文件，`source-set-before/`）javac **exit 1**（`找不到符号 AnonymousTopLevel$1`）；原类运行 `value=23:captured` / `events=capture,choose,base(23),render` / `counts=1,1,1,1`；javap 复核 child `super_class`=`Base`、ctor `(JLjava/lang/String;)V`、根 descriptor `()LRenderer;`、`Base implements Renderer`（`javap-anchor-*.txt`）。
- [x] 1.3 冻结负例（各须响亮失败并实测 javac 退出码）：(a) 无层级关系；(b) 祖父类或间接接口；(c) 数组或基本类型；(d) 含 `$`。先自检脚手架。

  **(b)(d)+遏制探针冻结为 fixture，(a)(c) 为 CI 合成探针（实现形式差异，如实登记）**：新冻 `tests/fixtures/proved-java-structure/anonymous-supertype-return-refusals/{indirect-supertype-return,nested-supertype-return,interface-self-invocation}/`（javac 8 -g:none、独立目录、全部原类可载）。(a) 无层级与 (c) 非引用在合法 Java 里**不可由 javac 产生**（声明返回类型必须可由分配类型赋值——正是本门所证纪律），故以冻结锚根类**等长描述符补丁**在 CI 内合成（`tests/anonymous_supertype_return.rs` 的 `non_javac_return_descriptors_keep_their_refusals`，测试内自检描述符恰出现一次；补丁类 javap 可读，存证 `descriptor-probes/`）；同长度约束下无基本类型描述符，与数组同 match 臂，数组探针即覆盖。实测拒绝码：(a)(b)(c) `anonymous_super_return_type_unproved`、(d) `anonymous_super_source_type_unproved`、遏制探针 `anonymous_interface_child_additional_use`（前后渲染逐字节相同）。脚手架自检抓到并修正两处真实错误：首轮 javac 失败未拦截致部分 class 被复制（已删并改为"先验编译成功再复制"，两处源码漏实现接口抽象方法已修）；对照正例首次采证 stderr 混入渲染文本（分离流重采）。

## 2. 实现

- [x] 2.1 放宽 `anonymous_super_return_type_unproved` 的返回段判据（design 决策 1/2/6）……与环 3 的参数表放宽正交组合……不得写四条平行字符串恰等比较。

  门整体移到 `parent_read` 解析与 `anonymous_super_declaration_unproved` 检查之后（`src/facade.rs` `project_class_source_anonymous_super`）。接受集 = 返回段 ∈ {parent, parent.super_class, parent.interfaces}（internal name 全等、一层、无 walk）∧ 参数表 ∈ {空, 单个已证捕获描述符}——返回段与参数表两个独立事实的合取，非平行字符串比较。描述符解析复用 reader 的 `descriptor_facts`（Method kind）；参数表字节经 `DescriptorComponent::bytes` 切取。`parameterized_shape` 判定位改为 `direct_return_parameters == [捕获描述符]`（与返回段无关——`(P)Lparent;` 与 `(P)LT;` 同走环 3 值流证明，`supertype-return` 转正例即其行为证明）。
- [x] 2.2 返回段取出后，`T` 与 `parent_name` **两者**都过既有 `anonymous_super_source_type_unproved` 判据……复用不另写（design 决策 3）。

  抽出模块级谓词 `spellable_source_type`（同包 + 无 `$` + 每段合法标识符），父类检查（4707 原地）与 `T` 检查共用同一谓词；父类路径拒绝码与文本逐字不变，`T` 不可拼写走同一拒绝码（负例 (d) 实测）。
- [x] 2.3 发射：返回位置按 `T` 拼写、分配点按 `parent_name` 拼写（design 决策 4）……不改 `emit.rs`……若取证发现必须改 emit.rs，停手报告。

  **取证结论：发射零改动**。根方法声明的返回位置拼写来自根方法自己的描述符（物理呈现本为 `static Renderer create()`——基线与实现后该行逐字相同），分配点拼写走既有 `emit_class_source_anonymous_return` 的 `source_type`（父类）。`crates/jarde-java/src/emit.rs`、`report.rs` 与站点扫描零字节改动（`git diff --stat` 核实；本片生产 diff 仅 `src/facade.rs` + reader 的普查计数）。未触发停手条件。
- [x] 2.4 拒绝非引用返回类型（design Open Question 2）：返回段以 `[` 开头或为基本类型描述符时拒绝，不误纳。

  `descriptor.result()` 非 `Some(非数组 Object 分量)` 一律拒绝（数组 `is_array()`、基本/`V` 走 `object_name()==None`，同一 match 臂）——拒绝码 `anonymous_super_return_type_unproved`（"not a directly named reference type"）。数组探针实测（`descriptor-probes/ArrayReturn`）；基本类型与数组同臂（等长补丁约束下无基本类型描述符，见 1.3 登记）。
- [x] 2.5 **不越界**（design Non-Goals）：不放宽含 `$` 的 `anonymous_super_source_type_unproved`……不做传递闭包；不放宽参数表；不改站点形；不新建层级 walk 共享件。

  全部守住：负例 (d) 实测 `$` 判据仍拒（`anonymous-capture` 不在本环闭合范围）；传递闭包两层形保持拒绝（`indirect-supertype-return` 冻结）；参数表合取未放宽（环 3 四负例除转正例外逐字拒绝）；`report.rs`/`emit.rs`/站点扫描零改动；未新建层级 walk。**新增一道 design 未枚举的门改动（共享 owner 普查自调用允许臂）经停手请示后落地**：施工实测发现锚在返回门之后还撞 `prove_anonymous_owner_xrefs` 的 `anonymous_interface_child_additional_use`（child 体 `invokevirtual 自身.seed`，环 0/1/3 锚体形均无此形故从未行使）——实现者停手提案、经父会话通道追认后按**具名判别 `AnonymousOwnerCensusPath`**（仅父类路径 ∧ DirectReturn 传使能值；`InvokeSpecial` 分支按收紧判据不开放）实现，判据与归因存档见证据目录 `root-replies-verbatim.md`，**待 root 验收独立复核**。配套遏制：接口路径探针 fixture 渲染逐字节不变；环 1 的 `unresolvable-child-read`（同普查、LocalDeclInitializer 形）在首版"仅按路径遏制"下曾被顺带打开（corpus 首跑第 3 处差异），补站点形遏制后复扫归零——事件全程登记于证据 README §四。
- [x] 2.6 （本片追加，登记普查允许臂的验证义务）接口路径遏制探针、环 1 遏制 SHA、grandchild 与既有匿名正负例逐字节不变、child 体自分配仍拒、corpus 双腿差异恰本形+转正例——五项全部实测满足（见 3.2 与证据 README §五）。

## 3. 验收

- [x] 3.1 锚 `anonymous-top-level` 渲染源集 `javac --release 8` 从 exit 1 转 exit 0、`java -Xverify:all` 事件日志与原 class 逐行一致；呈现为 `Renderer create() { … return new Base(choose()) { … }; }`（声明返回按接口、分配点按父类）。

  实测（证据 `baseline/` 与 `fixed/`）：渲染源集三文件 javac **exit 1 → exit 0**；运行输出 `value=23:captured` / `events=capture,choose,base(23),render` / `counts=1,1,1,1` 与原类 **diff 为空**；呈现行逐字为目标形（声明按接口、`new Base(choose())` 按父类、零 `$1` 引用）。`Base`/`Renderer` 渲染前后逐字节相同。
- [x] 3.2 零回退（design 五条不变量）：环 0/1/3 三锚渲染逐字节不变；环 1 遏制负例 SHA 仍为 `1badfcb5…`；环 3 五个负例仍响亮拒绝（除 1.1 裁定应转正例者）；`recover-proved-anonymous-local-capture`(6/6)、`recover-proved-anonymous-inner-this`(8/8)、`inline-proved-anonymous-super-arguments`(8/8)、`recover-ctor-reorder-dispatch-guard` 三向负例逐字通过。

  corpus 双腿扫描（111 渲染/腿，`corpus-two-leg-scan/`）：差异恰 2 处 +SUMMARY——锚与 `supertype-return` 转正例；三锚、`anonymous-local-decl-interface-hold`（SHA-256 `1badfcb5…` 两腿一致）、环 3 四个负例、接口匿名形全部零差异；首跑曾现第 3 处差异（`unresolvable-child-read`，见 2.5），遏制修正后复扫归零。全仓两轮测试含上述四套件逐字通过（轮 2 全绿）。
- [x] 3.3 门禁（实现者实测数字，root 独立复跑留 3.4）：`cargo test --workspace --tests --locked --no-fail-fast` 两轮——轮 1 2958 passed / 1 failed（`ordinary_generic_projection`，handoff 已知 flake 家族，隔离复跑两轮 14/14 绿判 flake；首轮另有 2 个预期失败 `p5_corpus_fingerprint` 与 `jarde-reader` 普查，按各自测试自述程序处理：census pinned (545,2381,251,1692,8) 更新 + fingerprint `--ignored regenerate_corpus_fingerprint` 再生，diff 纯新增 16 文件条目）；**轮 2 2959 passed / 0 failed**（基线 2953 + 本片 6 新测试，恰合）。`cargo fmt --all -- --check` 干净（首检 6 处 diff，fmt 后复检过）。clippy 从 `ci.yml` 46–76 行逐字生成（`--all-features`、29 项 `-A`、`-D warnings`）**exit 0**。`openspec validate --all --strict` **273/273**。corpus 双腿扫描差异恰本形+转正例。`git diff --check` 干净（含 staged 检查）。磁盘：每轮构建前 `df -h /` 53→44→27→26→24 Gi 全程 >12Gi；报告前 `cargo clean`。
- [ ] 3.4 root 独立复核返回段判据（一层直接关系、internal name 全等、复用 `parent_read` 无新解析通路）、`T` 与 `parent_name` 双可拼写检查、发射的返回位/分配位拼写、五条不变量（尤其环 1 遏制 SHA 与环 3 负例归类）、三方行为与账本更新（DT-06 匿名父类域 + `present-proved-java-structure` 5.3 剩余范围 + ctor-reorder 阻塞链），并对 1.1 的负例转正例裁决做独立复核。（**留 root**；另请 root 对普查允许臂的追认归因做鉴别——两份通信原文存档于证据目录 `root-replies-verbatim.md`。）
