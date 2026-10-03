# 单静态接口子折叠 —— 取证（change `recover-single-static-interface-fold`）

基线 worktree HEAD `6be15d9b`（主线，`df9dcf68` 之后）。固定 Y1 fixture SHA 复核一致
（`Y1.class` `fdc6a6de…`、`Y1$StrFn.class` `e7bab1f1…`，与 `../results/fixture-sha256.txt` 逐字相同）。
诊断快照复现（`member_family.state=prepared`、`capture.reason="static no-capture target was not
proved from the class-level relation"`、`projection.reason="capture proof is incomplete"`）与
`../results/member_family.txt` 一致。

**结论（本片交付的是否证取证与真实边界，而非实现）**：变更前提——"单直接静态接口子落回窄通道
`Candidate`，该通道无接口准入 → 不折叠，是 fold/mixed 两片选择交互留下的洞"——**不成立**。
受控 2×2 与巡逻基线重放表明，决定是否折叠的是 **根类文本是否携带 lambda 伴生投影**，与子行是
`class` 还是 `interface` **无关**；两个备选案都不能使 Y1 折叠，且都不是无副作用的等价选择。
详见 §3/§4。

## 1. 固定复现（任务 1.1）

- `class-source` 对 `../fixture/fam.jar:Y1`：`Y1` 不折叠（输出 SHA `9c5f9ddc6299…`，
  见 `fold-outputs/Y1-HEAD.jarde.java`），与巡逻记录同源。
- 巡逻基线 `a7762a92`（本片独立构建 `/tmp` 二进制重放）在同一 fixture 上**同样**不折叠；
  即"遗漏"在登记时已存在，不是本批切片引入，也没有被后续切片修好。
- `corpus-patrol-baseline-a7762a92.txt` = a776 的 372 类；`corpus-HEAD.txt` = HEAD 的 388 类；
  372 个共同键中 19 类文本/退出码不同，全部可归因于 `a776..HEAD` 间的 nested-generic-header 与
  lambda 切片（与 `recover-single-static-interface-fold` 无关）。

## 2. 选择点与改道面（任务 1.1 取证义务）

`src/member_inner.rs::scan_family_root` 的静态行走 `if row.access_flags & 0x0008 != 0 {
static_names.push(..); static_members.push(next); continue; }`（行判据
`source_spellable_member_row` **已含 0x0608/0x0609 接口准入**，由 `75a0114c` 引入，早于巡逻基线）。
返回序：`declaration_pair` → 窄通道 `Candidate`/`StaticMembersWithInstance` → `static_members.len() >= 2`
为 `StaticMembers` → 其余单行 `static_members.pop() → Candidate`。

消费方隔离确认：`Candidate` 通道的其它消费方是（a）`prepare_physical_class_source`
内 `if let FamilyRootScan::Candidate(candidate) … candidate.access_flags & 0x0008 != 0`
才去证 `prove_static_family_target`（即**只对静态行**）；（b）非静态子与枚举/注解投影对走
`scan_nested_enum_root`/`scan_nested_annotation_root` 自己的 `Candidate` 返回，与本扫描互不影响。

## 3. 两案对比（design 决策 1）——两案都失败

受控矩阵（配对 fixture：子行 `class`/`interface` × 根文本带/不带 lambda；源码在 `fixtures/`）：

| 用例 | 子行 | lambda | HEAD 折叠 | 巡逻基线 a776 折叠 |
| --- | --- | --- | --- | --- |
| `Z3` | iface | 无 | **是** | **是** |
| `Q1` | iface | 无 | 是 | 是 |
| `Q3` | class | 无 | 是 | 是 |
| `Q2` | iface | 有 | 否 | 否 |
| `Q4` | class | 有 | 否 | 否 |
| `Z6` | iface | 有 | 否 | — |
| `Y1M` | class | 有（与 Y1 逐字同 lambda 内容） | 否 | 否 |
| `Y1` | iface | 有 | 否 | 否 |
| `Y1X` | iface | 无（Y1 形态的等价非 lambda 写法） | **是** | — |

判别量是 lambda，不是子行种类：`Z3`（单静态接口子、无 lambda）**已经折叠**，正是变更假设其
"不折叠"的反例。`Y1M` 与 `Y1` 只差子行种类、lambda 内容逐字相同，两者都不折叠。

### 案 A（单静态行改道 `StaticMembers`）

- 改动：`scan_family_root` 尾部去掉 `static_members.pop() → Candidate`，单行也返回
  `StaticMembers([row])`。corpus 双腿扫描（`corpus-optionA.txt` vs `corpus-HEAD.txt`）：
  **1 个类变化** —— `VN3`（`nested-name-spelling-patrol/fixture-variants/VN3/vn3.jar:VN3`）
  由 `OK`（文本 SHA `e69fdd07…`）变为 `EXIT4`，新增诊断
  `anonymous_child_shape_unproved: the selected child is not an Object subclass of exactly one
  source-spellable interface`，`member_family` 由 `prepared` 变 `absent`，`anonymous_interface_projection`
  由 `absent` 变 `refused`；**文本本身逐字节不变**（1204 B）。机制：改道后该类的合成成员行（匿名
  `VN3$1`）不再是 `Candidate`，`static_fold_rows` 非空使 `report.member_family` 被折叠通道接管
  （`src/facade.rs` 约 L1800–L2049），而根含合成成员行时折叠整体拒绝 —— 报告的家族面与退出码被改变。
- **案 A 不能使 Y1 折叠**（改道后 Y1 仍 0 个嵌套声明；拒绝点仍是下文 §4 的门）。
- 因此案 A 不是"无副作用的最小等价改动"：它改变了非静态候选窄路径之外的一个真实类（VN3）的
  报告面与退出码。

### 案 B（窄通道加 0x0608 准入）

- **行判据没有可加的准入**：`source_spellable_member_row` 已在 `75a0114c`（早于巡逻基线 a7762a92）
  承认 0x0608/0x0609 接口行，接口 `Candidate` 已经选出并到达 `prove_static_family_target`。
- 窄通道唯一"类形态专属"的判据在 `src/facade.rs::prove_static_family_target`
  （~L24153）：要求子类恰有 **一个 `()V` 构造器**，接口永无构造器 → 恒 `Ok(None)` → capture
  `Refused`。实测（临时补丁打印）：接口子以 `0 constructors` 到达该点后无法构造 target。
- 故案 B 的实现面不是"加准入"，而是**新增接口形态的 no-capture 证书**（新的证明机制）。
- **案 B 同样不能使 Y1 折叠**：即使补上接口证书，Y1 仍在 §4 的两道门被拒。

## 4. Y1 的真实阻塞链（与子行种类无关）

对 Y1 逐门取证（临时 `eprintln` 补丁，已回退；源码树保持干净）：

1. **折叠根重投影门**（`src/facade.rs::project_class_source_member_fold`）
   `class_source::source_text(declaration, &root.fields, &root.methods, &ctx) != root.text` →
   `"root class has another source projection outside the member fold"`。
   `root.text` 已经过 lambda 伴生投影（省略/重命名伴生 + 调用点内联），而折叠的
   `ClassSourceTextContext` 把 `array_helper_indices/array_method_texts/array_helper_markers`
   以及 `enum_projection`/`initializer_field_order` 一律置 `None`，重投影得到的是**物理文本**。
   实测首个差异行：重投影 `public Y1() {` vs 实际
   `// jarde: omitted physical lambda helper "lambda$viaLambda$0" …`。
   → **任何根文本带 lambda（或数组辅助/枚举/初始化器）投影的类都命中**，与子行种类无关
   （`Y1M`/`Q4` 同为 class 子仍命中）。this gate 是**保守正确**的：临时跳过它后，折叠输出会把已
   投影的 lambda 文本**退回物理形态**（`Y1M.lambda$viaLambda$0((String) p0)` 且重新出现
   `private static … lambda$…` 成员），即丢失 lambda 内联呈现——不是可以简单放开的口。
2. **token 锚定门**（同函数，`"static fold token in method N is not tied to one proved class
   reference"`）。Y1 的失败 token 是 `viaLambda` 里 `Y1$StrFn local1 = (String p0) -> p0 + "!";`
   的**局部变量声明类型** `Y1$StrFn`。该 token 的**覆盖 source-map 段只给出 `bcis={5}`**，而
   bci 5 是 `astore_1`（opcode 76，无 CP 索引）。**恒定池确实命名该类型**——bci 0 的
   `invokedynamic #7`（描述符 `()LY1$StrFn;`）与 bci 8 的 `invokeinterface #11`（owner
   `Y1$StrFn`）都命名它——但 bci 0/8 不在该 token 的覆盖段集合里，故锚定门找不到"证明过、且
   CP 命名该类型"的指令而拒绝。类形态的 `Y1M` 不命中这枚 token（其子类类型出现在 `new`/`checkcast`
   的 CP 条目上），但 `Y1M` 在门 1 已拒。

两门都对 class 子同样生效 ⇒ 变更把 Y1 归因于"接口子"是**变量混淆**（Y1 fixture 同时具备
"接口子"和"lambda 根"两个变量）。`Y1X`（Y1 形态、去掉 lambda、保留接口子与 `interface StrFn`
声明）**已经折叠**，是这一结论的直接反例。

## 5. 真实存在的接口专属缺口（附带发现，非本变更验收面）

宽扫描定位到一处**确为接口形态专属**的 token 锚定缺口：锚定匹配器只认
`CpEntryKind::Class/FieldRef/MethodRef` 的 owner，**漏 `InterfaceMethodRef`**。
配对 `WCallI`（接口子 `M.sv()` 静态调用，invokestatic 指向 `InterfaceMethodRef`）不折叠，其 class
孪生 `WCallC` 折叠。最小补丁（`| InterfaceMethodRef { owner, .. }` + `InvokeDynamic/MethodType/
NameAndType` 的 descriptor 提及）实测：

- `WCallI` 折叠；corpus 双腿（`corpus-interface-anchor-G2.txt`）**4 类变化**，全部为新增折叠：
  `super-default-patrol/fixture/fam.jar:F1`、`sd/sd-negatives-orig.jar:SDAbstract`、
  `…:SDIndirect`、`sd/sd-variants.jar:pkg.SDPacked`（均 class 形态家族、恰好经接口调用）。
- 既有家族逐字不变：`M1`/`M2`/`FV1`/`FV3`/`FV5` 文本 SHA 与基线相同。
- 折叠行为核对：`F1` 折叠文本 `javac --release 8` 0 错误，`java -Xverify:all` 输出 `AB` / `I:A`
  与原 class 逐字一致。
- **但该补丁不能使 Y1 折叠**（Y1 仍卡在门 1 与门 2 的变量声明 token）；故它不满足本变更的验收。

## 6. 三方对照（JADX dev 构建）

固定 JADX（`~/workspace/testzone/jadx/…/bin/jadx`，dev）对 `../fixture/fam.jar` 的输出：

- **JADX 的结构性折叠达成**：`interface StrFn { }` 作为 `Y1` 的嵌套声明呈现，域内拼写
  `Y1.StrFn strFn = str2 -> …`（无 `$` 池拼写）——即变更想要的结构呈现，JADX 已做到。
- **JADX 输出不可编**：`javac --release 8` 报 `找不到符号: 方法 length()`（lambda 形参被推成
  `Object`，`v0.length()`/`str.length()` 无法解析）等 ≥6 处错误，无法重编运行。
- **Jarde 基线（不折叠）可编**：Y1 的物理输出 `javac --release 8` 0 错误（原 jar 作 classpath），
  故 Jarde 当前在"可编"这一维优于 JADX 在此 fixture 上的表现——**JADX 的文本相似度不构成本变更的
  行为验收依据**（与 `handoff.md` 的既定口径一致）。
- 该对照说明：本变更想要的"接口子嵌套呈现"是一个真实能力目标；但达成它需要在 Jarde 的证明面
  （根重投影 + token 锚）上补足，而不是把 Y1 判成"接口行准入缺失"。

## 7. 变体/负例前后（任务 1.2）

源码见 `fixtures/`，实现在 HEAD 时的前后状态（`4da1fe84` = member-class-staging 片：

| 用例 | 形态 | 原类 `-Xverify:all` | HEAD 折叠 | 说明 |
| --- | --- | --- | --- | --- |
| `Y1` | 单静态接口子 + lambda 根 | `hi!`/`45`/`[b, aa]`/`8` | 否 | 本变更目标 |
| `Y1X` | Y1 形态去 lambda | `hi!`/`45`/`[b, aa]`/`8` | **是** | 反例：同形态无 lambda 已折叠 |
| `Y1M` | 单静态**类**子 + 同 lambda | `hi!`/`45`/`[b, aa]`/`8`/`7` | 否 | 与 Y1 只差子行种类 |
| `Z3`/`Q1` | 单静态接口子、无 lambda | `ok`/`7` | 是 | 已折叠 |
| `Z6`/`Q2` | 单静态接口子、lambda 根 | `ok`/`9` | 否 | lambda 阻塞 |
| `Q3`/`Q4` | 单静态类子、无/有 lambda | `7`/`9` | 是/否 | class 子同现 lambda 阻塞 |
| `WCallI`/`WCallC` | 单静态子 + 静态调用 | `8`/`9` | 否/是 | 唯一真接口专属缺口（§5） |
| `V1` | 单静态接口子 + null 使用 | `n`/`hi!` | 否 | 与 A4/A5 同类 |
| `V2` | 单静态抽象方法类 | `5` | 是（`static abstract class Base`） | 类形态已折叠 |
| `V3` | 单静态类子 | `5` | 是 | 既有 |
| `V4` | 非静态成员类 | `5` | 是（instance fold） | 既有 |
| `V5` | 单静态注解 | `5` | 否（`nested_annotation_family=refused`，未达两方成员证据） | 既有边界 |
| `V6` | 单静态接口子 + 实现类兄弟 | `hi!` | 是（2 声明） | 多子已折叠 |

`4da1fe84` = member-class-staging 片；上表 HEAD 与 4da 在这些用例上的折叠判定一致（抽查
`SDAbstract`/`F1`/`M1` 亦一致），即缺口不是本批折叠片引入。

## 8. 门禁与磁盘

- `cargo test --workspace --tests --locked --no-fail-fast`：**全绿**（HEAD 干净树；
  仅 `ignored` 1 项）。日志 `/tmp/baseline-tests.log`。
- `cargo fmt --all -- --check`：**通过**。
- CI `.github/workflows/ci.yml` 的完整 29 项 `-A` 清单 clippy（`--workspace --all-targets
  --all-features --locked -- -D warnings`）：**通过**。
- `openspec validate --all --strict`：**258 passed, 0 failed**（HEAD 未新增 change 条目）。
- 语义：本片未改产品代码（`git status` 仅新增本证据目录与 tasks.md 说明），上述门禁只证明未引入
  回归，不构成变更实现验收。

## 9. 遗留与未验证

- **未实现**：change `recover-single-static-interface-fold` 的两案均不能达成其验收（Y1 折叠、
  既有家族逐字不变）。真正需要的改动跨出"窄切片"：折叠必须能够重投影根类**已投影**的文本
  （需把 lambda/数组/枚举/初始化器的投影输入保留进 `ClassSourceReport` 这一共享结构，或在折叠
  内重建同 context），并为"覆盖段不含命名该类型的 CP 条目"的类型 token 提供**健全**锚（现仅
  `Field/ClassDefinition/MethodPoint/MethodSignature/ConstructorParameter` 五类）。这是共享契约
  与证明面变更，超出本实现职责，需架构决策。
- **未做**：Y1 折叠产物不存在，故无 Y1 的三方行为验收；Y1 的物理输出重编仅作基线参考。接口锚
  补丁未合入（仅作为 §5 的可行性取证）。
- **待 root 复核**：tasks.md 3.3 保持未勾选。
- 本片 `fixtures/`/`fold-outputs/`/`results/`/`reproduce.sh` 为新增取证产物；产品源码树保持与
  HEAD 干净一致。

## 10. 产物

- `fixtures/`：受控变体源码（Z3/Z6/Q1–Q4/Y1M/Y1X/WCallI/WCallC/V1–V6）。
- `fold-outputs/`：HEAD 对 Y1/Z3/M1/M2 的 `class-source` 文本。
- `results/corpus-*.txt`：corpus 双腿扫描（HEAD / 案 A / 接口锚补丁 / 巡逻基线 a7762a92）。
- `results/fold-gate-census.txt`：corpus 中命中折叠门（`root-reprojection` / `token-anchor`）的
  16 个类。