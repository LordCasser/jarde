## 0. 前置确认（root 已完成）

- [x] 0.1 root 已用**决定性单变量实验**证明缺口存在并锁定判据落点：同一 `N1.java` 用 javac 23 `--release 8` 编 → `requireNonNull` 形 → 渲染引注 3 且 `arg1.new Inner(9).total()` 折叠；用真 javac 8（Corretto 1.8.0_432）编 → `getClass()` 形 → 引注 **18**、整方法 `not recovered`。两处判据（`init.rs:985-998`、`member_inner.rs:126-131`）已读码核实。取证与全部实验记录见 [qualified-outer-alloc-getclass-patrol](../../evidence/java-syntax-2026-10-04/qualified-outer-alloc-getclass-patrol/README.md)。（root 已完成）
- [x] 0.2 root 已核实两条拒绝文本（`init.rs:997`、`member_inner.rs:131`）在**全仓（含 in-file 单测）无任何测试断言**，故决策 4 的文本变更安全；两片既有测试的文本断言是 `tests/inner_class_static_mixed_folding.rs:336`（`contains("new N1().new Inner(3).total()")`）与 `:341`（`contains("return arg1.new Inner(9).total();")`），须**保持通过**。（root 已完成）

## 1. 取证与基线

- [ ] 1.1 复现 root 的单变量实验并记录：以 Corretto 1.8.0_432（`/Users/lordcasser/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home/bin/javac`）编译冻结的 `N1.java`，javap 逐条对照 `N1$Stat.use` 与 `N1.main` 的分配序列，确认与 [results/javap-idiom-comparison.txt](../../evidence/java-syntax-2026-10-04/qualified-outer-alloc-getclass-patrol/results/javap-idiom-comparison.txt) 一致（**唯一差别是 BCI 6 那一条**）。记录修复前的引注数（应为 18）与 `N1$Stat.use`/`N1.main` 的 `not recovered` 状态。
- [ ] 1.2 冻结 fixture（**必须用真 javac 8 编译**，这是本片的全部意义；须在 fixture 目录内记录编译器版本与命令行）：
  - **正例 A（家族形）**：`N1.java` 真 javac 8 产物族（`N1`/`N1$Inner`/`N1$Stat`），覆盖**两种限定符形**——参数限定符 `outer.new Inner(9)`（在 `N1$Stat.use`）与分配限定符 `new N1().new Inner(3)`（在 `N1.main`；root 已实测真 javac 8 对它**同样**发射 `getClass()` null-check）。行为基线 `10`/`7`/`13`。
  - **正例 B（最小形）**：一个只含 `outer.new Inner()` 的最小类族（标识符与形态须与 A 不同，满足 handoff "验收锚不得是唯一正例"）。
  - **负例 C（健全性，最关键）**：显式 `o.getClass();` 语句 + 紧邻的 `o.new In()`（root 已冻结源 [G-explicit-getclass.java](../../evidence/java-syntax-2026-10-04/qualified-outer-alloc-getclass-patrol/fixture/G-explicit-getclass.java)）。要求：呈现中用户的 `o.getClass();` **保留为语句**（不得被吞掉或误折叠为 null-check），整类可编译、行为与原 class 一致。
  - **负例 D/E**：`pop` 缺失形、检查与构造器非连续形（各须仍按既有码响亮拒绝）。若某形在 javac 产物中不可表达，改用等长描述符补丁的**合成探针**并 javap 存证（环 2 的做法），并在证据中说明为何不可表达。
  - 各 fixture `java -Xverify:all` 通过并记录实现前后行为。
- [ ] 1.3 记录修复前基线：`tests/fixtures` 内 `getClass` 形为 **0 类**、`requireNonNull` 形为 **8 类**（root 精确普查：按"dup→调用→pop 连续"匹配，非字面 grep）。故 corpus 双腿扫描在 `tests/fixtures` 内**预期零差异**——本片的正例来自新冻结的真 javac 8 fixture。

## 2. 判据扩展（单一所有者 + 两处投影）

- [ ] 2.1 在 `crates/jarde-java/src/facts.rs` 定义**唯一**的 null-check 拼写词表与谓词（design 决策 1）：`NullCheckSpelling { RequireNonNull, GetClass }` + `is_discarded_null_check(kind, owner, name, descriptor, interface_reference)`，**两条拼写成对匹配**（kind 与 symbol 一起判，禁止叉积）。带文档注释说明：两种拼写的返回值都是单槽故 `pop` 对两者同真；叉积不是任何 javac 的产物故不接受。归属 `facts.rs` 与既有 `STRUCTURAL_REFLECTION_METHODS`、`ACC_*` 同惯例，**不新建模块**。
- [ ] 2.2 `init.rs:985-998` 改为调用该谓词（投影 `call.kind()/owner()/name()/descriptor()/is_interface_reference()`）。**逐字保留**：`[qualifier, copy, check, pop]` 的连续四条取法（`block.get(index+2..index+6)`）、`pop.bci() >= at`、`Operation::Load{..}`、`Operation::Duplicate`、`pop.opcode() != 0x57`。
- [ ] 2.3 `member_inner.rs:126-131` 改为调用同一谓词：opcode→`InvokeKind` 映射按 design Open Question 1 处理（**先查证 `facts.rs` 是否已有同类映射，有则复用，不新建第二套**），从 `CpEntryKind::MethodRef` 取三个字节串。**逐字保留**：`head/copy/qualifier_copy/pop` 的 opcode 与 bci 检查、`cp_class_name` 子类名核对、构造器 `0xb7` 与 name/descriptor 核对、`first != qualifier_copy.bci || ordinary.iter().any(|bci| bci <= pop.bci || bci >= call_bci)` 区间检查。
- [ ] 2.4 同步拒绝文本与注释（design 决策 4）：`init.rs:997` 与 `member_inner.rs:131` 改为不提单一拼写的表述；`member_inner.rs:37` 注释的"explicit `requireNonNull; pop` pair"同步。**不得**因文本变更削弱任何断言（0.2 已核实无测试断言这两条文本）。
- [ ] 2.5 **不引入任何 `java_release`/`major_version` 判据**（design 决策 2：两种产物 major version 均 52，且违反 `classfile.rs:139-141` 原则）。实现中若出现按版本分支的代码即为本片范围外，须删除。

## 3. 回归与验收

- [ ] 3.1 `cargo test --workspace --tests --locked --no-fail-fast` 全绿（基线 **299 目标 / 2959 passed / 0 failed**；已知 flake 家族见 handoff.md，单测复跑两轮判定）；fmt；clippy 从 `ci.yml` 46–76 行逐字生成（含 `--all-features`、29 项 `-A`、`-D warnings`）；`openspec validate --all --strict`（**273 项**）；`git diff --check`（注意：证据里 `diff` normal-format artifact 的 `< ` 空行会被报 trailing whitespace，那是忠实记录**不得删**，只修自己写的 `.md` 装饰性空白）。**新增 fixture 后必须再生 corpus fingerprint**（`cargo test --test p5_corpus_fingerprint --locked -- --ignored regenerate_corpus_fingerprint`），并确认 `corpus_files_match_the_recorded_fingerprint` 通过——环 3 实现者曾漏此步，自报数字在合并态首跑即 1 failed。
- [ ] 3.2 三方对照（原 class / 固定 JADX dev / Jarde 重编，均 `java -Xverify:all`）：正例 A/B 的渲染源集 `javac --release 8` **exit 0**、运行输出与原 class 逐行一致（A 为 `10`/`7`/`13`）；**双腿**——同一源码的真 javac 8 产物与 javac 23 `--release 8` 产物**都**须通过（单腿正是该缺口不可见的原因）。
- [ ] 3.3 javac 9+ 零回退：8 个 `requireNonNull` 形 fixture 类渲染**逐字节不变**；`tests/inner_class_static_mixed_folding.rs`（含 `:336`/`:341` 两条文本断言）与 `recover-proved-member-inner-construction` 全部测试通过；负例 C 的用户 `getClass()` 语句保留、D/E 仍拒。
- [ ] 3.4 corpus 双腿扫描：差异类**只应是此前因 `getClass` 形被拒的类**；`tests/fixtures` 内预期**零差异**（1.3 基线）；出现任何 `requireNonNull` 形差异即回退失败，**停下报告**。
- [ ] 3.5 root 独立复核：两处判据是否投影同一谓词（不得各自硬编码）、位置约束是否逐字未放宽、无版本判据、拒绝文本与判据一致、真 javac 8 与 javac 23 双腿行为、负例 C 不误折叠；更新 **DT-03** 账本（`declarations-types.md` 证据边界列已记载本差距，须更新为已修复）与 `summary.md` 的 DT-03 状态归属。（留 root）
- [ ] 3.6 **独立债务登记（不并入本片）**：(a) `accessor.rs:480` 有意拒绝**返回值形写访问器** `(LC;I)I`（真 javac 8 对内部类写外部私有字段发射 `dup_x1; putfield; ireturn`），呈现为空 stub → 整类不可编译（响亮）；root 普查 `tests/fixtures` 532 类中**含 `access$` 的类为 0**、`p3_accessor_edges.rs:146-149` 合成访问器**全为读形**，故写臂从未被真实产物行使。取证须先读 `d09f5dea`（"recover proved parent field writes and private setter helper"，已合入主线，改 `build.rs`+250/`field.rs`+40）已交付的通路。(b) 整个 `tests/fixtures` 语料对该构造**只有 javac 9+ 拼写**（`getClass` 形 0 类），CI 结构性地无法发现此类版本耦合——是否做一次"真 javac 8 双腿语料"的系统性补强属独立大颗粒项，须单独评估。（留 root）
