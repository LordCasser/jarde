# recover-anonymous-parameterized-root 交付报告（coder，2026-10-04）

**实现完成，待 root 验收。** 基线 `c7dec2fd`，实现 tip `9f0cd47d`（worktree 内两笔实现/测试
提交 + 本证据提交；未 push、未动 main）。本报告没有引用任何 `ask_parent` 答复——本任务全程
**未调用** `ask_parent`，所有范围决定均为任务书钉死的 root 决策或本地保守实现选择，无任何
内容记为"root 裁决"。

## 一、五条 root 决策的落点与实现方式

**决策 1（只放宽参数表，返回部分仍恰为 `Lparent;`）** — `src/facade.rs` 根方法门
（`project_class_source_anonymous_super` 内，环 1 注释块之后）。`DirectReturn` 形的接受集从
`descriptor == "()Lparent;"` 扩为"`()Lparent;` 或 `(P)Lparent;`"，其中 `P` 取
`anonymous_val_capture_field(&child_facts)` 的字段描述符（child 形门 4671 已保证：有字段的
child 恰有这一个 `val$` 字段）。返回段 `)L{parent};` 逐字节参与比较，超类型返回与多参数保持
`anonymous_super_return_type_unproved`（负例 `supertype-return`、`multiple-parameters` 实测仍
该码拒绝，环 2 未被顺带放宽）。**判据的值流不在门上**：描述符匹配只开门，值流在捕获站点证明
里闭合（见决策 2）。

**决策 2（捕获实参来源扩为根方法参数，沿用接口先例判据）** — `src/facade.rs` 捕获站点证明
（`mixed` 分支）。新增 `parameterized_shape` 判定位（`site_shape == DirectReturn` 且描述符为
`(P)Lparent;`），其上要求接口先例（facade ~3540-3549）的合取：**ACC_STATIC**（`root.methods`
的成员旗标 0x0008；实例方法槽 0 是接收者，故静态是"槽 0 = 首个声明参数"的前提）、
**scan.complete**（站点普查完整）、`site.verified`、实参数 `== super_slots + 1`、**BCI 与 AST
对齐**（`class_source_anonymous_return_site` 的实参 BCI 序列与站点一致）——后三项是环 0 既有
代码，未动。接口先例的 `site.argument_parameter_slots == [Some(0)]` 一项**不能直接复用**：其
计算器 `anonymous_direct_double_parameter_slot`（report.rs:1549）只接受 `dload` 族操作码
（0x18/0x26-0x29，double 专属特化），`aload_0` 恒产 `None`。等价判据改为在捕获站点证明内对
SSA 直接闭合：实参的产值指令读 `Slot::Local(0)`、该值 `Definition::Entry{slot 0}`、
`uses().len() == 1`（恰被该分配实参消费一次）。这同时把 design Open Question (b) 的形
（捕获参数另有消费）钉在默认拒绝上。**未改站点扫描、未改 `report.rs` 的扫描器**（决策 5）。

**决策 3（不硬编码 `b"D"`，呈现类型走 `parameter_presented`）** — 捕获描述符全程取自 child
字段实际描述符（门上 `anonymous_val_capture_field`、证明里 `mixed.field_descriptor`，锚为
`Ljava/lang/String;`）；呈现类型沿用环 0 通道：实参 `Local` 表达式的
`argument.presented` → `AnonymousSuperCaptureSite.presented` →
`ProvedCapturedParameterRead.parameter_presented`（report.rs:1270 消费），无任何 `b"D"`/`D`
特化进入父类路径。接口路径残留的五处 `b"D"`（~3455/3490/3771/3800/3896）**逐字未动**
（`git diff` 核实：本片对 `facade.rs` 的改动全部位于 `project_class_source_anonymous_super`
函数体内，接口函数零改动）。

**决策 4（不改站点扫描、不改 `emit.rs`）** — `class_source_direct_return_new`、
`class_source_anonymous_site`、`emit.rs` 三个文件**零字节改动**（`git diff --stat` 核实）。
参数形的发射完全复用既有 `emit_class_source_anonymous_return` 的 `hidden_outer_argument_bci`
通道（捕获实参按 BCI 隐藏，super 实参集为空时实参列表为空 → `new Base()`）。站点形判别位
（环 1 的 `AnonymousSiteShape`）未触碰，接口路径的 `DirectReturn` 前置保持。

**决策 5（划分退化形显式覆盖）** — 锚的 super 实参集为空、child 构造器唯一参数是捕获角色。
`partition_anonymous_val_constructor`（`src/member_inner.rs`）**零改动**：读码确认该退化形在
既有代码上天然闭合——`super_slots` 为空时"super 实参是前导物理参数"的排序检查空真，
`capture_slots == parameter_slots[0..]` 恰成立，`super_descriptor`（`()V`）与空 super 参数表
互证。锚的端到端投影成功即该形的行为证明；`anonymous-super-args`（两 super 实参 + 一捕获，
环 0 锚）在 corpus 扫描中零差异，证明两形互不干扰。

## 二、划分退化形的处理（补充）

见决策 5。**无新分支、无新判据**：退化形由既有划分代码的空真分支承接。这是本片与环 0 锚的
关键差异点，验收两形都在：锚（空 super 集）投影成功，环 0 锚（非空 super 集）逐字节不变。

## 三、参数名来源为 AST 的证明方式（`-g:none` 腿）

替换名取 `class_source_single_parameter_name(root_ast, 0)`（report.rs:609）——同轮 AST 投影的
`parameter_names` 表（`NameTable` 通道），**函数体内无任何 LVT 读取**。证明腿：新冻结
`tests/fixtures/proved-java-structure/anonymous-super-dispatch-nodebug/`，源文件与锚逐字节相同、
`javac --release 8 -g:none` 编译，`javap -p -l` 计数 **0/0/0**（对照腿 3/2/1，见
`two-leg-lvt-counts.md`）。实现后该腿投影为 `create(java.lang.String arg0)`、捕获读取重拼
`arg0`（AST 命名通道的发明名），javac exit 0、运行输出与原 class 逐行一致；`-g` 腿拼 `captured`
（既有命名通道的 LVT 来源，上一环已登记的既有行为）。两腿的**重拼机制**一致，证明实现不依赖
调试信息。CI 守卫：`the_parameterized_root_projection_does_not_need_debug_information`
（`tests/anonymous_parameterized_root.rs`）断言 `arg0` 拼写与完整 javac+java 重放。

## 四、主锚的可观察行为对照

| 维度 | 前（基线 `c7dec2fd`） | 后（tip `9f0cd47d`） |
| --- | --- | --- |
| `create` 呈现 | `return new AnonymousSuperDispatch$1(captured);`（物理文本） | `return new Base() { void observe() { AnonymousSuperDispatch.observed = captured; return; } };` |
| 拒绝状态 | `anonymous_super_return_type_unproved`（"the root method return descriptor is not the exact superclass type"） | 投影（JSON `anonymous_interface_projection.state = absent`，CLI exit 0） |
| 渲染源集 `javac --release 8` | **exit 1**（`AnonymousSuperDispatch.java:20: 错误: 找不到符号`） | **exit 0** |
| `java -Xverify:all` | 原 class：`observed=captured-value` / `visibleDuringSuper=true` | **逐行一致**（diff 为空） |
| `-g:none` 腿 | （同前，物理文本） | `create(java.lang.String arg0)` + `new Base() { … arg0 … }`，exit 0、逐行一致 |

证据：`baseline/`（前）、`fixed/`（后）、`corpus-two-leg-scan/anchor-diff-content.txt`（扫描器
视角的同一转变）。

## 五、负例（tasks 1.3）实测

| 负例 | 前拒绝码 | 后拒绝码 | 渲染文本 | 渲染源集 javac |
| --- | --- | --- | --- | --- |
| multiple-parameters | `anonymous_super_return_type_unproved` | 同前（未变） | 逐字节相同 | exit 1 |
| capture-from-local | `anonymous_super_return_type_unproved` | `anonymous_capture_argument_unproved`（落点移入新值流证明——"not the root method's own parameter"） | 逐字节相同 | exit 1 |
| parameter-also-consumed | `anonymous_super_return_type_unproved` | `anonymous_capture_argument_unproved`（"consumed beyond the allocation argument"） | 逐字节相同 | exit 1 |
| instance-method | `anonymous_super_child_shape_unproved`（两字段 child：`this$0`+`val$captured`） | 同前（未变；投影自身的 ACC_STATIC 判据为纵深防御，javac 形不可达） | 逐字节相同 | exit 1 |
| supertype-return | `anonymous_super_return_type_unproved` | 同前（环 2 边界未放宽） | 逐字节相同 | exit 1 |

脚手架自检按 handoff 纪律执行且抓到两处真实脚手架错误并已修正：(a) `supertype-return` 首版源
本身编译失败（`Renderer` 无 `render()`），修正为与环 2 锚同形（接口声明 `render()`）；(b) 负例
javac 证据首版被 `cp` 覆盖（渲染根被 fixture 原源覆盖，exit 0 假绿），重排复制顺序后重测，
并以"渲染根首两行是 `public final class <名>`"自检后才采信 exit 1。

## 六、Open Questions 的处置（tasks 2.6，均按默认拒绝并登记）

- **(a) 实例方法形**：默认拒绝。直接判据是捕获站点证明的 ACC_STATIC 合取（先例 3524 对齐）；
  javac 产生的实例方法匿名类必带 `this$0`，故该形实际先被既有 `anonymous_super_child_shape_unproved`
  拒绝（负例 `instance-method` 实测），ACC_STATIC 为非 javac 形的纵深防御。未发现可安全放宽的
  取证，未放宽。
- **(b) 捕获参数另有消费**：默认拒绝。实现者分析该形**可能**可证安全（根方法体整体重发射保留
  全部消费点、参数留在作用域内、命名通道全方法一致；javac 8 的 effectively-final 只禁写不禁多读，
  而既有判据已证参数永不写）——但任务书将该形钉在 tasks 1.3 负例清单且要求"默认拒绝并登记"，
  故实现为**消费点恰一**（`uses().len() == 1`），负例 `parameter-also-consumed` 实测响亮拒绝。
  **此为本实现者的保守收紧，非 root 决策**；若 root 验收时认为可放宽，删除该单一合取即可，
  放宽决定归 root。

## 七、门禁真实数字（tasks 3.3）

| 门禁 | 结果 |
| --- | --- |
| `cargo test --workspace --tests --locked --no-fail-fast` | **两轮：297 目标 / 2947 passed / 0 failed**（主线基线 296/2943 + 本片 4 个新测试；第 1 轮曾现 2 个预期失败：`p5_corpus_fingerprint` 与 `jarde-reader` fixture 普查计数——均为"新增 fixture 须重测"的既有机制，按测试自述程序重测/再生后全绿） |
| `cargo fmt --all -- --check` | 干净（首检发现 2 处 diff，`cargo fmt` 后复检通过） |
| clippy（`sed -n '46,76p' .github/workflows/ci.yml` 逐字生成；`--all-features`、29 项 `-A`、`-D warnings`） | exit 0 |
| `openspec validate --all --strict` | **271/271**（与主线一致） |
| corpus 双腿扫描 | **99 渲染/腿，差异恰 2 处**（锚 + 同形 `-g:none` 腿，同一差异类 = 本形；接口路径、环 0/环 1 锚、全部既有负例零差异；无第 2 个差异类） |
| 环 1 遏制负例 SHA-256 | `anonymous-local-decl-interface-hold` 渲染 = `1badfcb5…`，与冻结基线**逐字节相同** |
| `git diff --check` | 干净 |
| 磁盘 | 每轮构建前 `df -h /`：50→56→54→36→31 Gi，全程 >12 Gi；**报告前已 `cargo clean`**（见文末） |

CI 守卫（handoff 强制"新增冻结 fixture 必须有 CI 测试引用"）：`tests/anonymous_parameterized_root.rs`
以 `include_bytes!` 引用 nodebug 腿 3 类 + 五个 refusal 全部 16 类，含完整 javac+java 重放两个
正腿与逐负例的物理文本/拒绝码断言。

## 八、流程事故登记（不影响交付，供验收参考）

1. **CLI 二进制陈旧**：裸 `cargo build` 只构建根包，不构建 `jarde-cli` 成员——首次改码后用旧
   CLI 复测得到"仍拒绝"的假象，经二进制 mtime 对比定位，改用 `cargo build -p jarde-cli` 后
   全部证据以正确二进制重采。**fixed/ 与 corpus 扫描的全部证据均出自最终源码态的二进制。**
2. 本会话**未调用** `ask_parent`，无任何"root 答复"需要存档或归因（决策归因纪律的空满足）。

## 九、遗留缺口与登记

- **`-g:none` 腿的参数名与 `-g` 腿不同**（`arg0` vs `captured`）：spec 场景"两腿的捕获读取重拼
  结果相同"按**机制一致**实现（同为"child val$ 读取 → 根方法参数名"通道，均投影成功、行为
  一致），非拼写逐字节相同——无 LVT 时原参数名不可恢复，与环 1 的双腿先例（`-g` 腿拼回 LVT 名、
  `-g:none` 腿发明名）同一口径。已在 fixture README 与本报告钉死该解释，供 root 验收裁决。
- **LocalDeclInitializer 形下的参数捕获**是环 1 已发布行为（该形无根方法返回门，捕获站点的
  Entry/never-written 检查本就放行参数源），本片未触碰也未收紧——其 ACC_STATIC/单消费判据
  不含于本片 spec（spec 场景限定直返位）。若 root 认为需对齐，属后续片的范围决定。
- 环 2（返回父类超类型）按边界保持拒绝；其判据实现仍未立项。
- 本片未改 `handoff.md`、未改本片之外任何 change 的 spec 文件；tasks.md 仅勾选 0.1–3.2 中
  属于本实现者的项，3.3/3.4 留 root。
