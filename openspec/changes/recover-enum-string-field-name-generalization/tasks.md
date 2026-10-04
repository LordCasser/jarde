## 1. 取证与冻结

- [x] 1.1 复核 proposal 的五行落点表（`facade.rs` 14197 / 16009 / 16023 / 16026 / 16031，行号随主线漂移、以锚点名为准），并确认数据通路缺口：`PendingEnumConstructorEdge`（约 13942）只有 `caller`/`call_bci`/`target_owner`/`target_descriptor` 四项，**不携带字段名**；14197 处 `EnumCodeReference::Field { name, … }` 的 `name` 即已证名但被 `matches!` 丢弃。（2026-10-04 实现者复核：五行落点在 `c226413c` 行号逐个吻合；另证 source fields 与物理 fields 表同序号空间——`resolve_enum_constant_body_relations` 同时持有两表，arbitrary tail 通道已在用物理索引读 `source_fields`，故走 design 决策 2 的 index 主路径，未用退回路径。）
- [x] 1.2 重放六形对照（[证据](../../evidence/java-syntax-2026-10-04/enum-string-field-name-hardcode/)）：确认判别变量是字段名本身（C1/C3 的 `op` 投影成功，C2/C4/C5/C6 的 `t`/`value`/`x` 不投影），且 C1 渲染源集 `javac --release 8` **exit 0**、C2 渲染源集 **exit 1 `此处需要枚举常量`**（root 已实测，须复现）。（2026-10-04 实现者以冻结 `fam.jar` 在基线/修复双二进制上重放：C1/C3 双腿投影、C2/C4/C5/C6 仅修复后投影，见 [six-shape/replay.txt](../../evidence/java-syntax-2026-10-04/recover-enum-string-field-name-generalization/six-shape/replay.txt)；javac 前后对照见 evidence 目录 `javac/`。首次重放两次踩 harness 坑（丢 `demo/` 前缀 + grep 模式错拼常量名），均按"空/错输出先疑脚本"纪律捕获后修正。）
- [x] 1.3 冻结**非 `op` 字段名的正例** fixture（源级，如字段名 `label` 或 `t`，两常量各带 ASCII String 实参与常量专属体），并确认它在当前主线上**不投影**（作为本片的红灯锚）。**该正例必须有 CI 测试引用**（handoff 强制纪律），否则缺陷无法被 CI 发现——这也是 `recover-proved-string-arg-enum-constant-bodies` 当初漏掉的原因。（冻结于 `tests/fixtures/enum-string-field-name/`（源 + `-g:none` classes + Probe + 行为基线），CI 引用 `tests/enum_string_field_name.rs` 两测试；开发中 `git stash` 实证两测试在未修 lib 上 FAILED、修复后 PASSED。）
- [x] 1.4 冻结负例：两个私有 String 字段且写入目标不唯一；构造器 Code 不完整；`putfield` 目标非 String 描述符。（`string_constructor_negatives_refuse_loudly`：双 store 拒、形状破坏拒/停（响亮、保持物理文本）；`string_constructor_identity_binds_the_written_field_and_refuses_everything_else`：非 String 描述符/外部 owner/未解析引用/缺失/重复/静态-enum 旗标目标逐一单元拒绝。另按 design Risks 落了放宽边界正例 `a_second_unwritten_string_field_leaves_the_proved_target_unique`。字节级"类型一致的非 String putfield"不可构造，端到端落在成员运行停止（响亮），门本身单元覆盖。）

## 2. 实现

- [x] 2.1 打通"已证字段身份 → 发射处"的通路（**先读 design 决策 2**）：权威来源是构造器 BCI 6 处 `putfield` 的目标字段（`facade.rs:14197` 的 `EnumCodeReference::Field { name, … }`，当前被 `matches!` 丢弃）。**优先传 `field_index`（u64，指向已证 `source_fields` 位置）**而非裸名字——发射处 `prove_enum_constant_body_group`（约 15598）本就在 `source_fields` 上按属性筛选，传 index 可复用已证字段的 `declaration.name`、避免第二套匹配逻辑。**取证先确认 (1) 构造器边证明与 (2) body-group 证明是否同一字段序号空间**（design Open Question 1）：若是→传 index；若否→退回传已证名字字节、在发射处按名唯一定位（此退回路径 design 已批准，不因此停手，但须在报告说明为何 index 不可用）。**不得**在发射处重新猜测或回退到字面量。（按 index 主路径实现：`PendingEnumConstructorEdge.assigned_field: Option<{index, name, descriptor}>`，证明函数内一次解析、发射处 `source_fields.get(index)` 直取。）
- [x] 2.2 把 16009/16023/16026 三处的 `== b"op"` 判据改为"**被该构造器 `putfield` 写入的 String 字段**恰一个"，其余既有属性检查（描述符、owner、非 static/synthetic/隐式枚举成员、`ACC_PRIVATE`、有 `declaration`、无 markers）**逐字保留**。（16023 计数键为已证 (name, descriptor)；唯一性计数不再按裸名。）
- [x] 2.3 把 16031 的发射文本 `this.op = arg0;` 改为用已证字段名拼写；14197 的错误文本 `"the String constructor does not preserve Enum and op semantics"` 中的 `op` 措辞同步改为不依赖固定名（该文本可能出现在既有断言里，须核实并如实更新，不得为让旧断言变绿而削弱判据）。（全仓搜证：旧错误文本与 `this.op` 在 `src/`、`crates/*/src/`、`tests/` 无任何测试断言，仅定义点；相邻形状拒绝文本 `…pure \`op = arg0\`…` 同步去 `op` 化，亦无断言。）
- [x] 2.4 **不越界**（proposal Non-Goals）：不放宽 String 实参可拼写性；不支持多个 String 实参；不处理其它描述符；不改常量体匿名子类投影判据；不泛化接口匿名路径的 `b"D"` 特化。

## 3. 验收

- [x] 3.1 1.3 冻结的非 `op` 正例：投影成功、渲染源集 `javac --release 8` **exit 0**、`java -Xverify:all` 运行结果与原 class 一致；**`TestEnums2a/DoubleOperations`（`op` 名）呈现逐字节不变**（最重要的零回退锚）。（javac exit 1→0、`-Xverify:all` 输出逐字节一致见 evidence 目录；`op` 锚 baseline/fixed 渲染 `diff` 为空（SHA 记于 `renders/op-anchor-byte-identical-proof.txt`），对冻结转录的残差仅今日主线的 class-Signature 标记与 jar 组成 SnapshotId，子类字节摘要逐字相同。）
- [x] 3.2 负例全部响亮拒绝；`recover-proved-string-arg-enum-constant-bodies` 的既有测试零回退；全仓测试通过。（`enum_constant_body_relation_tests` 35/35 含全部既有 DT-12 测试；全仓数字见 evidence README 门禁段与交付报告。）
- [x] 3.3 门禁：`cargo test --workspace --tests --locked --no-fail-fast`（基线数字以开工时主线实测为准；已知 flake 家族见 handoff.md，含 `bulk_recovery_delivery::one_declaration_bounds_the_librarys_own_presentation_too`，单测复跑两轮判定）；`cargo fmt --all -- --check`；clippy **从 `.github/workflows/ci.yml` 46–76 行逐字生成**（含 `--all-features`、29 项 `-A`、`-D warnings`）；`openspec validate --all --strict`；corpus 双腿扫描（差异应仅非 `op` 名的 String 实参枚举形；**出现其它差异类即停下报告**）；`git diff --check`。磁盘纪律：每轮构建前 `df -h /`，低于 12Gi 先 `cargo clean`，**报告前必 clean**。（corpus 双腿：单类腿 532/532 零差异；family 腿 532 渲染恰 1 差异=新增 `demo/LabeledOps` 本身，62 个退出码双腿完全一致；详见 evidence `corpus/`。）
- [x] 3.4 root 独立复核已证名的数据通路、四处判据替换、发射文本、`TestEnums2a` 逐字节零回退与非 `op` 正例的三方行为，更新 **DT-12** 账本（"含匿名常量体的枚举"——本缺陷所在单元；DT-10 是"空枚举、普通枚举常量"，不涉常量体，故不属本片账本范围）。（**root 2026-10-04 验收记录**；合并主线 `4734a4a1`，实现者 tip `a54a667d`；全部独立复跑与端到端实测，不采信实现者自报：

**门禁（root 在干净合并态实测）**：`cargo test --workspace --tests --locked --no-fail-fast` → **298 目标 / 2953 passed / 0 failed**（主线基线 297/2947 + 本片 1 个新测试文件 6 个测试，与实现者报告一致；**含 `corpus_files_match_the_recorded_fingerprint ... ok`**——正是环 3 实现者自报时漏掉的那项，本片实现者主动再生了 fingerprint）；`cargo fmt --all -- --check` 通过；clippy 从 `ci.yml` 46–76 行逐字生成（含 `--all-features`、29 项 `-A`、`-D warnings`）**exit 0**；`openspec validate --all --strict` **273/273**；`git diff --check` 见下"diff-check 说明"。

**缺陷确已消除（root 用自己的六形探针独立复测，非采信实现者的 C1–C6 重放）**：以合并后二进制重跑 root 原始取证探针（`demo` 包、`IOps{double apply(double,double)}`、六形仅字段名不同）：

| 探针 | 字段名 | 修复前 | 修复后 |
| --- | --- | --- | --- |
| C1 | `op` | 投影 | **投影**（零回退） |
| C3 | `op` | 投影 | **投影**（零回退） |
| C2 | `t` | 不投影 | **投影** |
| C4 | `t` | 不投影 | **投影** |
| C5 | `value` | 不投影 | **投影** |
| C6 | `x` | 不投影 | **投影** |

即判别变量（字段名）已消除，而 `op` 形零回退。**root 自查纠错**：首次复测时误用 `^\s+[AB]\(` 模式（探针常量名实为 `TIMES`/`DIVIDE`），得到"六形全部 0"的假结果——正是 handoff "验证脚手架必须先自检"纪律所述情形，改用正确模式后得到上表。

**跨时零回退证明（比实现者的 before/after 自证更强）**：root 以 `javac --release 8 -g:none` 重编 DT-12 冻结源（`TestEnums2a/*.java`，`package demo;`），用合并后二进制渲染 `demo.DoubleOperations`，与 **2026-09-27 已验收的 DT-12 转录**（`…/fixed/outputs/TestEnums2a/g-none/jarde-source/demo/DoubleOperations.java`）逐行 `diff` → **完全一致**（含 `TIMES("*") { public double apply(…) }` 常量列表形与 `private demo.DoubleOperations(java.lang.String arg0) { this.op = arg0; }` 构造器文本）。即该锚的呈现跨越 7 天与本片改动**逐字未变**，与实现者"同一 javac 23.0.1 重编、子类字节 SHA 与冻结转录逐字相同"的结论相互印证。

**非 `op` 正例端到端（root 独立复现）**：新冻结 `tests/fixtures/enum-string-field-name/`（字段名 `label`）→ 渲染为 `TIMES("*") { … }, DIVIDE("/") { … };` + `this.label = arg0;`、**0 引注**；渲染源集（`LabeledOps` + `LabeledValue` + `Probe`）`javac --release 8` **exit 0**（修复前该形渲染为字段声明式常量 → exit 1 `此处需要枚举常量`），`java -Xverify:all` 输出与 fixture 的 `original-behavior.txt` **逐行一致**（`TIMES=*:0:6.0:demo.LabeledOps$1` / `DIVIDE=/:1:2.0:demo.LabeledOps$2`——含匿名子类名与 ordinal，是**可观察**基线而非仅返回值）。

**数据通路复核（design 决策 2 的主路径）**：`PendingEnumConstructorEdge` 新增 `assigned_field: Option<PendingEnumAssignedField{index,name,descriptor}>`，**仅 String 形私有构造器边为 `Some`，其余三处构造点为 `None`**；发射处 `source_fields.get(index)` 直取，**无按名二次匹配**（避免两套逻辑漂移）。实现者对 Open Question 1 的回答（(1)(2) 两处证明**同一字段序号空间**，依据是 `resolve_enum_constant_body_relations` 按 `read.facts.fields.iter().enumerate()` 逐条 push 且同一 Vec 传给 `prove_enum_constant_body_group`，并指出已上线的 arbitrary-tail 通道早已按物理索引读 `source_fields`）——root 认为该论证成立且与主线既有行为同构，**认可走 index 主路径而非退回名字路径**。

**判据替换与不变量复核（root 读码逐处核实）**：`facade.rs:14234` 的 putfield 门仍要求 `owner == enum_owner && descriptor == b"Ljava/lang/String;"`（**String 限定未被放宽**）；`16092` 起定位字段的属性检查（描述符 `Ljava/lang/String;`、`ACC_PRIVATE`、有 `declaration`、无 `markers`）**逐字保留**，并**新增**三项更强检查：已证 (name, descriptor) 身份匹配、`String::from_utf16` 可解码、`is_java_identifier`；唯一性判据改为按已证 (name, descriptor) 计数（`!= 1` 仍拒 `the source String field is ambiguous`）。发射文本由字面量 `this.op` 改为 `this.{assigned_name}`；错误文本 `"…not a pure \`op = arg0\` constructor"` → `"…not a pure single-argument field store"`。**生产码中已无 `b"op"` 字面量**（残留 4 处 `this.op` 全在测试区，其中 `assert!(!text.contains("this.op = arg0;"))` 正是本片要求的"非 `op` 正例"守卫）。

**放宽边界已实测（design Risks 项）**：`twofields`（两个 String 字段、构造器只写一个）修复前拒（count 0）、修复后接受（count 1）——符合"写入目标唯一即可"；`twostores`（构造器写两个 String 字段）双腿均响亮拒绝。

**测试覆盖（root 核实全部 6 个新测试均在门禁中 ok）**：`the_labeled_string_argument_fixture_projects_the_proved_field_name`（含 `assert!(!text.contains("this.op"))`）、`the_labeled_projection_recompiles_and_runs_as_the_original`（真实重编运行腿）、`labeled_field_name_string_bodies_project_from_the_proved_name`、`a_second_unwritten_string_field_leaves_the_proved_target_unique`、`string_constructor_negatives_refuse_loudly`、`string_constructor_identity_binds_the_written_field_and_refuses_everything_else`。冻结 fixture 有 **3 处** CI 测试引用（handoff 强制纪律），fingerprint 已再生且纯新增。

**生产改动范围复核**：prod diff 实为 **`src/facade.rs` +108 行**（root 按 hunk 归属统计；其余 453 行在测试区）+ `crates/jarde-reader/src/classfile.rs` 的 5 行**亦在测试区**（冻结 fixture 计数元组 `(527,2346,251,1692,8)`→`(532,2359,251,1692,8)`，附注释留档）。即真正的生产改动是单一文件 108 行，与"打通数据通路 + 四处判据替换 + 发射文本"的规模相称，无越界。`prove_static_assignment_suffix` 的 `totalUnits`/`sumUnits`、`value`/`intValue`、接口路径 `b"D"` 特化**均未触碰**（design Non-Goals 守住）。

**diff-check 说明（root 判定为非阻塞，且不得"修"）**：`git diff --check` 报 2 处，一处在 root 自己写的 `recover-anonymous-supertype-return/proposal.md:14`（纯装饰性空行，**root 已修**）；另一处在实现者的证据 artifact `renders/labeled-before-after.diff` 第 24/27 行。root 核实该文件是 `diff` **normal 格式**输出（行首 `<`/`>`/`---`/`8c8,25`），其 `< ` + 空内容是**忠实记录的产物**（左侧空行），**剥离它会篡改证据**，故保留不动。CI 不跑 `git diff --check`，非 CI 阻塞。

**归因（本会话第三起核验，此次清白）**：实现者报告"全程未调用 `ask_parent`，无任何 root 裁决引用"——root 核实其证据目录确无答复存档，且实现落点与 design 决策逐条一致，**无幻影授权**。其主动披露两起自身流程事故并自行修正：(a) 一次**极性反转 bug**（拒绝条件误抄 find 的 `== 0` 形）被临时插桩自检捕获；(b) 一次**陈旧二进制事故**——`cargo test` 不重建 bin target，导致"修复后"CLI 首轮仍渲染旧物理形，被"同字节进程内可投影"的对照揭穿，重建后全部证据以新二进制重摄。两者都印证 handoff 的脚手架自检纪律。

**遗留（如实登记）**：(1) `CI 上的实际运行`本地无法验证（root 会在 push 后跟踪 CI）；(2) DT-12 账本已按本任务要求更新（见本次提交），另 DT-11 的 `totalUnits`/`sumUnits` 窄首片限制已由 root 在同日审计中登记到 [declarations-types.md](../../evidence/jadx-feature-inventory-2026-09-27/declarations-types.md) 与证据 README，属**文档记载的可接受窄首片**，不在本片范围；(3) 字节级"类型一致的非 String putfield"负例不可构造（值类型强制描述符），实现者已在证据说明并改由 14197 门的单元覆盖 6 种拒绝——root 认可该替代。）
