# recover-javac8-getclass-null-check-idiom —— 实现与取证记录（实现者，2026-10-04）

实现 change `recover-javac8-getclass-null-check-idiom` 的落地记录。**结论：四条决策全部按
design 落地并通过全部可执行门禁；但主锚（N1 族引注 18→0）被一个本片范围外的第三形态挡住
——真 javac 8 对 `N1.main` 的分配限定符形（`new N1().new Inner(3)`）发射的 null 检查舞蹈
落在两处拼写判据都不覆盖的位置（详见下文「停手报告」）。参数限定符形（`outer.new Inner(9)`）
已由本片修复并双腿端到端验证（隔离族 N1x：引注 8→0、重编双编译器、行为 13/10/10 逐行一致）。**

## 一、四条决策的落点

1. **单一所有者 + 两处投影**（design 决策 1）：`crates/jarde-java/src/facts.rs` 新增
   `NullCheckSpelling { RequireNonNull, GetClass }` + `pub fn is_discarded_null_check(
   kind, owner, name, descriptor, interface_reference) -> bool`，紧邻既有
   `STRUCTURAL_REFLECTION_METHODS` 词表（与 `ACC_*` 同归属惯例，未新建模块）。
   kind 与 symbol 用一个 `match` 元组**成对匹配**（`NullCheckSpelling::stated_by`），
   叉积（`invokestatic Object.getClass`、`invokevirtual Objects.requireNonNull`）结构性落空。
   `!interface_reference` 在谓词内。文档注释写明：两拼写返回值均单槽故 `pop` 对两者同真；
   叉积非任何 javac 产物故不接受；「结果被丢弃」是调用点事实、留在调用点。
   - `crates/jarde-java/src/init.rs`（verify_member 的成员构造臂）：投影
     `call.kind()/owner().as_bytes()/name().as_bytes()/descriptor().as_bytes()/is_interface_reference()`。
   - `src/member_inner.rs`（prove_family_call_site）：opcode→`InvokeKind` **就地映射**
     （`0xb8 → Static`、`0xb6 → Virtual`，其余提前 refuse）——**Open Question 1 的查证结论**：
     `facts.rs` 没有任何既有 opcode→InvokeKind 映射（全仓 `0xb6|0xb7|0xb8|0xb9` 的字面量
     只出现在 facade.rs 的非 InvokeKind 用途与各调用点字面比较），故按 design 倾向就地映射、
     不新建第二套。`MethodRef` 三字节串直投；接口引用投影为 `false`（池条目已是 `MethodRef`
     而非 `InterfaceMethodRef`，该投影忠实于事实，附注释）。
2. **无版本判据**（决策 2）：全 diff 无 `java_release`/`major_version`（`git diff | grep`
   为空，任务 2.5 自查）。
3. **位置约束逐字保留**（决策 3）：init.rs 的 `block.get(index+2..index+6)`、`pop.bci() >= at`、
   `Operation::Load{..}`、`Operation::Duplicate`、`pop.opcode() != 0x57`；member_inner.rs 的
   `head/copy/qualifier/qualifier_copy/check/pop` 六条 opcode 与 bci 检查、`cp_class_name`
   子类名核对、构造器 `0xb7` 与 name/descriptor 核对、
   `first != qualifier_copy.bci || ordinary.iter().any(|bci| bci <= pop.bci || bci >= call_bci)`
   ——逐字未动（可对照 git diff：init.rs 的 if 头与 member_inner.rs 的其余检查原样）。
   `pop.opcode()==0x57` 检查留在两处调用点，未移入谓词。
4. **拒绝文本与注释同步**（决策 4）：`init.rs` →
   `"the member constructor at BCI {at} lacks the contiguous local load, dup, discarded null check, pop"`；
   `member_inner.rs` → `"member call has no exact early discarded null check"`（opcode 映射
   的提前 refuse 分支同文本）；`member_inner.rs:37` 注释的 "explicit `requireNonNull; pop`
   pair" → "explicit discarded-null-check call"。改前 grep 全部引用点：除本 change 的
   spec/design/tasks 文档外，仅历史证据转录（FV4-before.txt、patrol README 等）——均不改写；
   无任何测试断言这两条文本（复核 root 0.2 结论成立）。

## 二、验证记录（门禁真实数字）

| 门禁 | 结果 |
| --- | --- |
| `cargo test --workspace --tests --locked --no-fail-fast` | 全绿（修复 fixture-sweep 计数后两轮；单测复跑纪律见下） |
| fixture-sweep 计数点（`classfile.rs` repository_class_fixtures…） | 新增 7 类/19 体后按该测试的既定程序再实测并更新为 `(552, 2400, 251, 1692, 8)`，附注释行 |
| `cargo fmt --all -- --check` | 干净 |
| clippy（ci.yml 46–76 逐字生成，含 `--all-features`、29 项 `-A`、`-D warnings`） | exit 0 |
| `openspec validate --all --strict` | 274/274 |
| `git diff --check` | 干净（未新增任何尾随空白） |
| corpus fingerprint 再生 | `regenerate_corpus_fingerprint` 通过后 `corpus_files_match_the_recorded_fingerprint` ok |

## 三、本片验收判据的实测结果

- **主锚（真 javac 8 N1 族，任务 3.2 的 A 腿）**：**未达成——被第三形态挡住**（见停手报告）。
  修复后 N1 呈现仍为 13 处引注、`use`/`main` 未恢复（prefix 全文
  `prefix-realjavac8-N1-rendered.txt` vs postfix `postfix-realjavac8-N1-rendered.txt`）。
- **隔离正例（参数限定符形，N1x + Wrap，任务 1.2 正例 B 扩充为两条）**：
  - N1x（无 main 的 N1 同构族）：prefix 8 引注/不折叠 → postfix **0 引注/折叠**
    （`return arg1.new Inner(9).total();`），重编双腿（真 javac 8、javac 23 `--release 8`）
    exit 0，`java -Xverify:all` 输出 **13/10/10** 与原 class 逐行一致；
  - Wrap（局部变量限定符、不同标识符）：prefix 7 引注 → postfix 0，双腿均折叠为
    `local1.new Seed(2).grow(3)`，运行输出 5。
- **负例 C（显式 `getClass()` 语句，G）**：postfix 呈现保留 `arg1.getClass();` 为语句、
  仅折叠 `arg1.new In()`、`class In` 折叠进根文本；javac 23 腿同形（且 baseline 上也同形——
  该腿本就通过）。窄通道呈现约定（签名保持池形 `G$In`）与既有片一致、非本片改动；
  恢复体的可编译与行为以 sibling-unit 集合验证：重编 exit 0、`-Xverify:all` 输出
  `explicit->1 / plain->1` 与原 class 一致。
- **负例 D/E（等长字节补丁合成探针）**：D（pop→astore_1）命中既有形状门，baseline/fixed
  均响亮拒绝（`use` 整方法 not recovered）；E（aload_1;dup→nop;aload_1）在更早的既有解码门
  `ir_frame_deferred` 响亮拒绝（两腿、修复前后一致）——等长补丁无法在保持解码干净的前提下
  把「窗口错位」送到形状门，理由记录在 `negatives-d-e-probes.txt`。
- **javac 9+ 零回退**：8 个 requireNonNull 形 fixture 类渲染逐字节不变
  （`zero-regression-8-requirenonnull.txt`，8/8 IDENTICAL 且非空）；整库 **545 类**
  baseline/fixed 双二进制逐类渲染对比 **0 差异**（`corpus_dual_leg_scan.py`，
  545 identical / 0 differing / 0 errors——与 root 普查基线「tests/fixtures 内 8 个
  requireNonNull 形、0 个 getClass 形」钉死的零差异预期一致）。
- **corpus 双腿扫描**：同上，`tests/fixtures` 内零差异；无任何 requireNonNull 形差异。

## 四、停手报告：主锚被分配限定符形（第三形态）挡住

**触发条件**：任务书停手条件 (a)——root 的字节码事实记载有误。

**root 的记载**（proposal/design/spec）："两处的 null 检查均为
`dup; aload_1; dup; invokevirtual Object.getClass:()Class; pop`"，并据此把修复范围钉在
两处拼写判据（init.rs verify_member 臂 + member_inner.rs prove_family_call_site 臂），
design 明言本片是「并列增加一种拼写」而非「新增机制」。

**实测（`main-allocation-qualifier-legs-javap.txt` 有两腿逐条 javap）**：

- `N1$Stat.use`（参数限定符）：该说法成立——`aload_1; dup; invokevirtual getClass; pop`
  落在 `[qualifier, copy, check, pop]` 连续四条窗口内，两处判据的既有位置约束恰好覆盖，
  本片修复后即折叠（N1x 隔离族端到端证明）。
- `N1.main`（分配限定符 `new N1().new Inner(3)`）：**两腿形状不同**——
  - javac 23：`new N1; dup; invokespecial N1.<init>` 之后**直接** `iconst_3; invokespecial
    Inner.<init>`，**无任何 null 检查**（新分配外层可证非空）——init.rs 分配限定符臂
    （`// javac again writes no null check for it`）正是按这个形状写的；
  - 真 javac 8：同一位置插入 `dup(23); invokevirtual getClass(24); pop(27)`——舞蹈在
    **嵌套构造之后、成员构造器之前**，即分配限定符臂的 `member_ordinary_arguments`
    实参窗口内。该窗口要求每条指令都是构造器实参的值依赖，`dup` 不是
    → 既有门 `jre_new_member_order`（"the instruction at BCI … is not an ordinary
    argument dependency"）拒绝，与拼写无关。
- **连锁**：main 的重恢复 retaining fallback → 整族 joint fold 拒绝 → 家族回退
  statics-only → `use` 的重恢复不带 member targets（`read_class_source_member_inner_targets`
  因窄通道 prove_target 的 ACC_PUBLIC 门不准备包私有成员）→ new@1 效果扫描拒绝
  `jre_new_interleaved_effect`（BCI 5 的 Duplicate）→ `use` 不折叠。
  即：**两处拼写判据的修复无法触达主锚**——诊断过程与证据见
  `postfix-realjavac8-N1-rendered.txt` 的 diagnostics 与本目录的 javap 对照。

> ## root 验收裁定（2026-10-04，追加于实现者记录之后；上文为实现者原文，不改写）
>
> **实现者的停手结论成立、处置正确，予以接受**：分配限定符形确实不被本片的拼写判据触达，主锚未达成，属机制扩展而非本片范围。实现者未自行扩权，符合任务书停手条件 (a)。
>
> **但其机制归因中的"第一道门是 `jre_new_member_order`"属实现者的内部推断，root 未复核、亦未证伪。** root 以**合并态**二进制重渲染真 javac 8 的 `N1.main` 并读其 diagnostics，实测的**表层**拒绝码为 `jre_new_interleaved_effect` + `jre_new_shape` + `jre_new_sites`，诊断原文：
>
> - `"the construction at BCI 12 was not presented: the instruction at BCI 16 is an Allocate { ty: \"N1\" } between the allocation's copy and its constructor call, and presenting the construction would write that effect somewhere else"`
> - `"the construction at BCI 16 was not presented: the instance the allocation at BCI 16 builds is read only by instructions this build quotes (BCIs 23), so the construction has no place in the body"`
> - `"4 construction candidate(s) read under new@1: 2 presented as \`new\`, 2 refused"`
>
> **`jre_new_member_order` 不在 `main` 的诊断列表中。** 但 root **只实测了表层诊断码、未用插桩复现实现者的连锁**，故**不宣称其连锁为假**——其连锁末步（"new@1 效果扫描拒绝 `jre_new_interleaved_effect`"）恰与 root 实测码相符，两者可能在不同层级各自成立（root 测的是 `main`，实现者的连锁同时涉及 `main` 与 `use`）。
>
> **对后续片的强制要求**：第一道门究竟在 `verify_member` 的实参窗口、还是在 `new@1` 的嵌套站点"实例读者集合"判定，**必须由后续片自行以插桩或增量实验重新推导，不得继承本文件任一方的归因**。该问题决定修复落点与健全性负例的设计，两者工作量与风险完全不同。root 的疏失已如实登记于 `openspec/changes/recover-javac8-getclass-null-check-idiom/tasks.md` 3.5：先未经核实照录实现者归因，后又用"证伪"这种超出自身证据强度的措辞去更正它——两处都不对。
>
> **本片实际达成范围（root 独立实测）**：参数限定符形 + 隐式 this 形已修。`N1x`/`Wrap` 族（真 javac 8 产物，root 用 javap 核实其 `getClass`≥1 / `requireNonNull`=0 / major=52）源码区 quotes=0、not-recovered=0，渲染源 `javac --release 8` **exit 0**，root 自建 driver 运行 `java -Xverify:all` 输出 **`7`/`13`/`10`** 与原 class 同 driver 输出**逐行一致**。分配限定符形仍未折叠（`N1` 族源码区 quotes=13）。

**为什么不在本片内修**：接受分配限定符位置的舞蹈需要 (a) init.rs 嵌套/分配限定符臂在
实参窗口内接受一条被丢弃的 null 检查调用（`member_ordinary_arguments` 现在拒绝一切非实参
依赖指令），(b) member_inner.rs 的 6 条窗口 `[new, dup, aload, dup, check, pop]` 为该位置
扩展新形（嵌套构造占位时窗口内容完全不同），(c) 各自的健全性负例（用户显式语句在嵌套
位置的对照、区间检查新形）——这是任务书明令禁止的「自行改判据或造新机制」，且 design 的
「非新机制」前提对该形态不成立。

**给 root 的候选方向（未经实现，仅取证）**：
1. 扩展分配限定符臂：实参窗口允许「连续 `[dup, 被丢弃的 null 检查调用, pop]`」一段（复用
   本片谓词），并以「被检查值 == 嵌套构造产物、单次使用」锚定；member_inner.rs 为
   `[new, dup, <嵌套构造>, dup, check, pop]` 形扩展成员调用证明的锚点集。
2. 或者收窄主锚：把「N1 族 18→0」改判为「N1x/Wrap 参数限定符族 0 引注 + N1 族的
   `use` 单独腿」，把分配限定符形立为独立 change（它对 requireNonNull 形同样不存在——
   javac 23 不发射——是**纯 javac 8 版本耦合**，与根因同族但机制面完全不同）。

## 五、文件索引

- `fixture/N1.java`（root 巡查冻结源副本）、`fixture/n1-realjavac8-rerun/`（真 javac 8 重编，
  SHA256 与 root 冻结类逐字节一致）、`fixture/n1-javac23/`（javac 23 --release 8 重编腿）、
  `fixture/Wrap.java` + `wrap-realjavac8/` + `wrap-javac23/`（正例 B）、
  `fixture/g-realjavac8/`（负例 C 类，源为巡查冻结 G-explicit-getclass.java）、
  `fixture/position-negatives/`（D/E 合成探针 + javap 存证）。
- `results/`：javap 复现、main 两腿对照、prefix/postfix 渲染、负例记录、零回退记录、
  corpus 扫描脚本与说明、普查脚本（census_null_check_idioms.py，自检内置）。
- CI：`tests/fixtures/recover-javac8-getclass-null-check-idiom/`（README 记录编译器版本与
  命令行）+ `tests/recover_javac8_getclass_null_check_idiom.rs`（双腿正例、负例 C 双腿、
  D 合成探针拒绝）。
