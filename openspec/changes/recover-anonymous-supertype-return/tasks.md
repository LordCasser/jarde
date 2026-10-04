## 0. 前置（root 门禁，未满足不得开工）

- [ ] 0.1 **确认环 0/1/3 均已合入主线**（`e1c89d57` / `25f2589e` / `d906464e`）。本环改的门与环 3 改的是**同一处**（环 3 刚把它条件化为 `plain_return`/`capture_return`），串行实施会 rebase 冲突。用 `git log --oneline --grep=parameterized-root` 或查环 3 的 tasks 3.4 是否已勾确认。若未合入，**停手报告**。
- [ ] 0.2 重读 [design.md](design.md) 的六条决策、五条不变量、两条 Open Questions，以及 [anonymous-chain-rings-2-3](../../evidence/java-syntax-2026-10-04/anonymous-chain-rings-2-3/README.md) 的设施盘点（**无现成的类级可赋值性证明件**，故本环 MVP 只做一层、复用 `parent_read`）。
- [ ] 0.3 **重验行号锚点**：design 引用的 `facade.rs` 约 4735–4757（返回段判据）与约 4758（`resolve_class_source_dependency_read_raw`）在环 3 合入后的位置——本会话 facade.rs 已两次因相邻片漂移 100+ 行，**不得照抄行号**，以锚点名（`anonymous_super_return_type_unproved`、`plain_return`/`capture_return`、`resolve_class_source_dependency_read_raw`）定位。

## 1. 取证与冻结（**1.1 是阻塞项，须在写任何生产代码前解决**）

- [ ] 1.1 **复核 root 已裁定的 Open Question（design 已实测钉死，勿重复取证，但要独立复现一次）**：环 3 已冻结的负例 `anonymous-parameterized-root-refusals/supertype-return` 与本环锚形高度相似，**root 已用 javap 裁定它满足本环判据、放宽后应转为正例**（事实：child `super_class`=`Base`；`Base` 的 `interfaces: 1` = `Renderer`；`Renderer`/`Base`/`ParameterizedSupertypeReturn` 三名均不含 `$`；且该 fixture 源码注释自己写明 "ring 2 boundary … this slice widens only the parameter table, never the return type"——环 3 实现者有意为环 2 冻结）。请**独立复现**该裁定（跑一次 javap 核对上述四项事实、读该 fixture 注释），确认与本环锚 `anonymous-top-level` 的唯一区别确为"根方法带一个捕获参数"。**若你的复现与 root 裁定不符，停手报告**（说明 root 的事实有误）。若相符：**它是范围变更**（须把该 fixture 从环 3 负例重新归类为正例、更新环 3 spec/tasks 与账本）——**你不得自行修改环 3 的 spec 文件**，须在报告中提出，由 root 验收时裁决执行。
- [ ] 1.2 重放锚 `anonymous-top-level`：记录当前拒绝码（应为 `anonymous_super_return_type_unproved`）、当前呈现（root 实测为 `static Renderer create() { return new AnonymousTopLevel$1(choose(), captured); }`）、**渲染源集** `javac --release 8` 当前 **exit 1**（`找不到符号`，因渲染文本引用非法标识符 `AnonymousTopLevel$1`）、原 class `java -Xverify:all` 事件日志基线。用 javap 复核 design Context 的每项事实（child `super_class`=`Base` 顶层、ctor `(JLjava/lang/String;)V`、根方法 descriptor `()LRenderer;`、`Base implements Renderer`）。**注意：`anonymous-top-level` 的 fixture 是单文件双顶层类（`Renderer`/`Base`/`AnonymousTopLevel` 同在 `AnonymousTopLevel.java`），渲染源集须含全部相关类，不要只渲染一个类就编译（root 自己踩过：只渲染 root 类会因缺 `Base`/`Renderer` 而报无关的"找不到符号"）。**
- [ ] 1.3 冻结负例（各须**响亮失败**并实测 `javac` 退出码）：(a) 返回类型与父类**无任何层级关系**；(b) 返回类型是父类的**祖父类或间接接口**（两层，须拒绝——验证 MVP 只做一层）；(c) 返回类型是**数组**或**基本类型**；(d) 返回类型或父类名**含 `$`**（嵌套，须仍撞 `anonymous_super_source_type_unproved`）。**先自检脚手架**（文件名与 public 类名一致、每项独立目录、`jar` 从父目录打包保留包前缀、`javac … > err.txt 2>&1; code=$?` 直判 javac、脚本先在已知正负例上自检——见 handoff "验证脚手架必须先自检"，本会话 root 与实现者共踩 6+ 次）。

## 2. 实现

- [ ] 2.1 放宽 `anonymous_super_return_type_unproved` 的返回段判据（design 决策 1/2/6）：从"返回段恰为 `parent_name`"改为"返回段 ∈ {`parent_name`, `parent_read.facts.super_class`, `parent_read.facts.interfaces` 各元素}"，**一层直接关系**、internal name 全等比较。**把检查移到 `parent_read` 可用之后**（约 4758），或就地补取，**不得新建第二套父类解析通路**。与环 3 的参数表放宽**正交组合**（design 决策 6：先定返回段、再定参数表，二者独立组合；不得写四条平行字符串恰等比较）。
- [ ] 2.2 返回段取出后，`T` 与 `parent_name` **两者**都过既有 `anonymous_super_source_type_unproved` 判据（不含 `$`、每段合法标识符、同包），**复用不另写**（design 决策 3）。
- [ ] 2.3 发射：返回位置按 `T` 拼写、分配点按 `parent_name` 拼写（design 决策 4）——目标形 `Renderer create() { return new Base(choose()) { … }; }`。**不得**把 `new` 的目标改成 `T`（会改变构造器绑定语义）。不改 `emit.rs`（环 1 已提供声明位重拼通道）；若取证发现必须改 emit.rs，**停手报告**。
- [ ] 2.4 拒绝非引用返回类型（design Open Question 2）：返回段以 `[` 开头或为基本类型描述符时拒绝，不误纳。
- [ ] 2.5 **不越界**（design Non-Goals）：不放宽含 `$` 的 `anonymous_super_source_type_unproved`（故 `anonymous-capture` **不在本环闭合范围**，其验收锚只能是 `anonymous-top-level`）；不做传递闭包；不放宽参数表；不改站点形；不新建层级 walk 共享件。

## 3. 验收

- [ ] 3.1 锚 `anonymous-top-level` **渲染源集** `javac --release 8` 从 exit 1 转 **exit 0**、`java -Xverify:all` 事件日志与原 class 逐行一致；呈现为 `Renderer create() { … return new Base(choose()) { … }; }`（声明返回按接口、分配点按父类）。
- [ ] 3.2 零回退（design 五条不变量）：环 0/1/3 三锚（`anonymous-super-mixed-direct`/`anonymous-super-args`/`anonymous-super-args-debuginfo`）渲染**逐字节不变**；环 1 遏制负例 `anonymous-local-decl-interface-hold` 渲染源码区 SHA-256 仍为 `1badfcb5…`；环 3 五个负例仍响亮拒绝（**除 1.1 裁定应转正例者**）；`recover-proved-anonymous-local-capture`(6/6)、`recover-proved-anonymous-inner-this`(8/8)、`inline-proved-anonymous-super-arguments`(8/8)、`recover-ctor-reorder-dispatch-guard` 三向负例逐字通过。
- [ ] 3.3 门禁：`cargo test --workspace --tests --locked --no-fail-fast`（**基线以开工时主线实测为准**，环 3 合入后为 **297 目标 / 2947 passed**；已知 flake 家族见 handoff.md，单测复跑两轮判定）；`cargo fmt --all -- --check`；clippy **从 `.github/workflows/ci.yml` 46–76 行逐字生成**（含 `--all-features`、29 项 `-A`、`-D warnings`）；`openspec validate --all --strict`；corpus 双腿扫描（差异应仅本形 + 1.1 裁定转正例者；**出现其它差异类即停下报告**）；`git diff --check`。**新增 fixture 后须再生 corpus fingerprint**（`cargo test --test p5_corpus_fingerprint --locked -- --ignored regenerate_corpus_fingerprint`）——环 3 实现者正是漏了这步，自报 2947/0 却在合并态首跑 1 failed，被 root 独立门禁抄出，勿重蹈。磁盘纪律：每轮构建前 `df -h /`，低于 12Gi 先 `cargo clean`，**报告前必 clean**。
- [ ] 3.4 root 独立复核返回段判据（一层直接关系、internal name 全等、复用 `parent_read` 无新解析通路）、`T` 与 `parent_name` 双可拼写检查、发射的返回位/分配位拼写、五条不变量（尤其环 1 遏制 SHA 与环 3 负例归类）、三方行为与账本更新（DT-06 匿名父类域 + `present-proved-java-structure` 5.3 剩余范围 + ctor-reorder 阻塞链），并对 1.1 的负例转正例裁决做独立复核。（留 root）
