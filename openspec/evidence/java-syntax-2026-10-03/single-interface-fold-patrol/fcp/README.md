# 折叠上下文投影保留 —— 实现证据（change `recover-fold-context-projection-preservation`）

基线：worktree 主线 `c6b64900`（debug 构建重放，冻结 Y1 fixture SHA 复核一致：`Y1.class`
`fdc6a6de…`、`Y1$StrFn.class` `e7bab1f1…`）。实现于本 worktree（提交 `af37db70` 起）；
本文件记录取证（design Context (a)–(c)）、两案对比与择案、实现、验收与登记缺口。

## 1. 取证（task 1.1，design Context (a)–(c)）

### (a) 折叠 context 置 `None` 的投影输入集合与既有载体

首轮装配（`src/facade.rs` 类装配缝，text context 构造）与折叠重投影
（`project_class_source_member_fold`）的通道差异恰为五项：

| 通道 | 首轮来源 | 报告中的既有载体 |
| --- | --- | --- |
| `initializer_field_order` | `project_static_initializer_group` | `initializer_proof::Proved{fields}`（**已序列化**，按 `<clinit>` 写序）——可从事实推导 |
| `enum_projection` | `prepare_enum_constant_source_projection` | 无文本载体（私有 `enum_constant_proof` 是证明非文本） |
| `array_helper_indices`（省略成员：数组 helper + lambda companion） | 数组/lambda 通道 commit | 无载体 |
| `array_method_texts`（投影成员文本） | 同上 | 无载体（仅存在于装配后 `root.text`） |
| `array_helper_markers` | 同上 | 无载体 |

`member_table`/`declared_methods` 在折叠前置（execution Complete）下与首轮恒等，无需携带。
`methods[i].text` 保持物理文本（数组/lambda 通道只写侧车）；桥投影（`project_bridge`）改写
`methods[i].text` 本身，天然在重投影中存活——桥不是本缺口的一部分。

### (b) 两案对比（Y1 + corpus 全族）

- **案 1（保留）**：`ClassSourceReport` 新增可选序列化字段 `projection_inputs`（五通道中的
  四项；初始化器序由既有 `initializer_proof` 事实推导），折叠以报告事实重建同一 context。
  serde 兼容：`skip_serializing_if`（空时整个字段不出现在 JSON——无投影类逐字不变，实测
  Z3 无键、Y1L 有键）；消费面：仅折叠通道（root 门、child 门、成员 staging、嵌套子块）；
  重跑等价：字节级（门变为一致性检验——保留事实重投影 ≠ root.text 即拒绝）。
- **案 2（重建）**：lambda 内联决策不可从报告重建——候选发现、inline 计划与发射依赖
  same-run AST 侧车（`method_asts`）与发射期预算，报告不携带；在折叠内重建须复刻整段
  装配逻辑（含拒绝路径与序），等价性不可证，且违反设计决策 2"首轮投影输入保留为事实
  （非重算）"。枚举投影同理（证明可私有保留，文本须重发射）。
- **择案 1**。判据逐项：serde 兼容（案 1 空-跳过兼容，案 2 不动报告但等价不可证）、
  消费面（案 1 收敛于折叠通道）、重跑等价可证性（案 1 事实 vs 案 2 重算）。

### (c) token 锚定门：不随案 1/2 消解，需独立健全锚

折叠的 token 扫描在成员**物理 recovery 文本**上进行（`project_static_fold_owner_texts`），
与 root 重投影无关。Y1 的失败 token 是 `viaLambda` 局部声明类型 `Y1$StrFn`，覆盖段
bcis={5}=`astore_1`（无 CP 索引）；命名该类型的 CP 条目在 bci 0（`invokedynamic #7` 描述符
`()LY1$StrFn;`）与 bci 8（`invokeinterface #11` owner），均不在覆盖段集合。**实测取证**
（临时探针，已移除）：SSA 中 bci 5 的 astore 读 `Stack(0)`，其值恰由 bci 0 的
invokedynamic 定义——生产者链成立。故补**健全锚**（见 §2），不放宽覆盖段含 CP 索引的
直接匹配（direct matcher 的 CP 种类未变）。

## 2. 实现（task 2.1/2.2）

- **保留（案 1）**：`ClassSourceProjectionInputs`（`omitted_methods`/`markers`/
  `member_texts`（含 `emission`：发射体 + 成员锚段表）/`enum_projection` 文本）随报告
  序列化；lambda 通道在 commit 成员上捕获发射体与其 `emit_source_map` 重放锚表（仅保留
  本成员锚定的段；预算停则表降级为空前缀，投影照常）。`initializer_field_order` 由已序列化
  的 `initializer_proof` 推导（proved 序 + 余下物理序，与首轮同置换）。
- **折叠重投影**：root/child 门均以保留事实重建 context；门仍是 `source_text(...) ==
  text`——现在是一致性检验（其它通道已认领的文本照样拒绝）。child 的枚举/初始化器投影
  （嵌套子块不携带）拒绝折叠，防降级；恒等序 + 无 `<clinit>` 的空 proved 组不视为投影。
- **staging 组合**：带发射体的成员在发射体上扫描/锚定/编辑/放置（`projected_body_*`
  放置器与 `lambda_projection_text` 同构，带声明改写）；仅有文本无发射体的成员（改名
  companion、数组改写）需改写时**显式拒绝**（"cannot re-spell the projected member text"），
  不回退物理体；实例重跑（override）或桥/捕获写编辑叠加投影体时同样显式拒绝。嵌套子块
  （`nested_static_member_source_text`）携带 child 自己的省略成员/标记/投影文本。
- **token 健全锚**：覆盖段无直接 CP 命中且无异常处理器时，取覆盖指令为**单局部写 + 单栈
  读**的 store，其读值的唯一定义指令若 CP 条目命名目标类型（`Class` 名；`FieldRef`/
  `MethodRef`/`InterfaceMethodRef` owner 或类型/返回描述符分量；`InvokeDynamic`/`Dynamic`
  返回描述符分量），则以**生产者 bci** 为锚——局部声明的类型即所存值的类型，生产者的池
  条目正是该类型文本的物理出处（lambda 位点即 invokedynamic 描述符返回型）。覆盖段含
  CP 索引的直接匹配要求未放宽；§5 的 InterfaceMethodRef owner 直接锚仍属另片（WCallI
  现状不变）。

## 3. 变体/负例前后（task 1.2；`results/variants-fcp.txt`，SHA 为投影文本）

| 用例 | 形态 | 基线折叠 | 本片折叠 | 重编 `--release 8` | `java -Xverify:all`（新输出 vs 原类） |
| --- | --- | --- | --- | --- | --- |
| `Y1`（frozen fam.jar） | 接口子 + lambda 根 | 否 | **是**（SHA `b3bf9580…`） | 0 错误 | `hi!`/`45`/`[b, aa]`/`8` = 原类逐字 |
| `Y1M` | **类子** + 同 lambda | 否 | **是**（`29f2a5d6…`） | OK | `hi!`/`45`/`[b, aa]`/`8`/`7` = 原类 |
| `Z6`/`Q2` | 接口子 + lambda 根 | 否 | **是**（`b7f454c3…`/`e0486da9…`） | OK | `ok`/`7` = 原类 |
| `Q4` | 类子 + lambda | 否 | **是**（`2a072563…`） | OK | `7` = 原类 |
| `Z3`/`Q1`/`Q3`/`Y1X` | 无 lambda 根 | 是 | 是（行为不变；JSON 无新键） | OK | 与基线一致 |
| `WCallI` | InterfaceMethodRef 直接锚 | 否 | **否**（§5 另片边界保持） | — | — |
| ARR/ARRL | 数组返回 lambda 根 | 否 | **是**（`int[]::new` 内联保留） | OK | `[0, 0, 0]` = 原类 |
| EN/ENL | **枚举常量投影根** + 静态子 | 否 | **是**（常量表/隐式成员省略保留） | OK | `7`/`7` = 原类 |
| 测试 PLAIN | 无投影根零回退断言 | 是 | 是（报告无 `projection_inputs` 键） | OK | `5` = 原类 |

数组构造通道（"projected N array-constructor site(s)" 标记形）无独立 fixture 被钉住；
其投影文本保留但无发射体——需改写时按 §2 显式拒绝（登记，非放宽）。

## 4. 重跑不降级（task 2.1 不变量）

- 测试 `folded_lambda_root_second_round_does_not_degrade`：折叠单元重编（`-g:none`）后再读，
  仍折叠、嵌套声明与 lambda 内联均在、无物理 companion 调用/声明。
- **既有缺口如实登记**：显式类型 lambda 呈现重编后，javac 生成的字节码含恢复层局部赋值
  证明拒绝的写（P3 2b.2），`viaStream` 二轮出现 fallback 段。此退化**先于本片存在**：基线
  （未折叠）首轮文本以原 jar 作 classpath 重编后二轮同样出现 4 处同族 fallback（实测两腿
  相同），与折叠无关；本片折叠产物首轮可编可跑（§3），二轮折叠/内联呈现不降级。

## 5. corpus 双腿扫描（task 3.1/3.2）

`results/corpus-fcp-{baseline,change}.txt`：主线 `c6b64900` 独立 worktree 二进制 vs 本片，
逐 jar 逐类 `class-source --format text` SHA256（388 项，0 MISSING，退出码无变化）。
**8 类文本变化**（`results/corpus-fcp-diff.txt`），全部为新折叠，全部重编 + `-Xverify:all`
运行与原类**逐字一致**：

`NamedMemberFamilyCalls`（实例路）、`lambda-inline-patrol:fam.jar:Y1`、`Z1`、`NG1`（实例
路）、`NG2`、`NG3`、`NG4`、`sif:fam.jar:Y1`。其中 `NG4` 即 nested-generic 测试钉的 owner
local 形（测试已随健全锚更新为新边界）。其余 380 类（含 §5 门禁 census 中的
`F1`/`SDAbstract`/`SDIndirect`/`SDDiamond`/`SDPacked`/`H3` 等接口方法引用/直接锚形）逐字
不变——另片边界保持。

## 6. 三方对照（task 3.2；固定 JADX dev `~/workspace/testzone/jadx/…/bin/jadx`）

| 腿 | Y1 行为 |
| --- | --- |
| 原 class（fam.jar） | `hi!`/`45`/`[b, aa]`/`8` |
| 固定 JADX Java-input | 结构折叠（`interface StrFn` 嵌套 + lambda）但 `javac --release 8` **6 错误**（形参被推成 `Object`，`v0.length()` 不可解析）——不可编，行为腿不可用（与 sif README §6 基线口径一致，非本片回归） |
| Jarde 折叠单元重编（SHA `7fdddb52…`，`fold-outputs/Y1-fcp.jarde.java`） | `hi!`/`45`/`[b, aa]`/`8` 与原类逐字一致 |

## 7. 门禁（task 3.1）

- `cargo test --workspace --tests --locked --no-fail-fast`：全绿；整仓运行中
  `p4_plugins::the_plugin_plane_leaves_the_structural_planes_own_answer_untouched` 失败一次，
  单测复跑通过（handoff.md 已登记 flake 家族，重跑判定非回归）。
- `cargo fmt --all -- --check`：通过。
- clippy：CI `.github/workflows/ci.yml` 实有 29 项 `-A` 清单
  （`--workspace --all-targets --all-features --locked -D warnings`）：零警告。
- `openspec validate --all --strict`：**260 passed, 0 failed**（HEAD 未新增 change 条目时
  同为 260；本片未动 spec 目录结构）。
- 磁盘纪律：每轮构建/测试前 `df -h /`（17–46Gi 区间），报告前已清理。

## 8. 遗留与登记

- **接口方法引用直接锚**（`WCallI`/`F1`/`SD*`/`H3` 族）：另片已立（sif §5），本片未动；
  生产者侧 CP 匹配天然接受 `InterfaceMethodRef`（返回描述符或 owner），但直接覆盖段匹配
  不变。
- **数组构造通道**的投影成员无发射体（文本保留、改写拒绝）——若需该族折叠改写，须为
  `emit_class_source_array_constructors` 补同样的锚表发射（与 lambda 通道同构，未在本片
  验收面）。
- **改名 companion**（rename 分支）成员文本保留但不可改写（显式拒绝）；改名成员体内出现
  折叠目标 token 的家族将因此拒绝折叠——现状如此登记。
- **二轮恢复层的显式类型 lambda 局部赋值缺口**（§4）：独立恢复层缺口，两腿一致。
- 待 root 复核：tasks.md 3.3 保持未勾选。
