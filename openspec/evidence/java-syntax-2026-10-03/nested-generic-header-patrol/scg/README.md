# 同类泛型绑定证明实现证据（recover-same-class-generic-bindings，2026-10-03）

主线 `b90400f7` 之上的 worktree 实现。固定 fixture SHA 与前后对照见 [results/](results/)；
本文件记录两个拒绝门的取证结论、使用点清单数据源、证明规则、Z1 逐项重放、既有测试翻转、
corpus 双腿扫描与门禁。

## 1. 拒绝点取证（任务 1.1）

两个门都在消费层 `src/class_source.rs`（reader 只提供 `Signature` 解析、作用域与逐位置擦除证明）：

- **方法门** `generic_call_binding_unproved`（原 `project_method_signature` 内，整池扫描）：
  `CpEntryKind::{MethodRef, InterfaceMethodRef}` 中 `owner == this_class && name == 本方法名`
  即拒——**连 descriptor 都未核对**，也无任何消费位事实。
- **字段门** `field_generic_body_unproved`（原 `project_field_signature` 内）：
  `CpEntryKind::FieldRef` 中 `owner == this_class && name 同 && descriptor 同` 即拒。

拒绝时已有事实：物理成员（name/descriptor/access_flags）、已解析 `Signature`（`parse_*_signature`）、
`prove_*_erasure_with_class_scope` 的逐位置擦除证明、类 scope（headers 片注入的
`class_scope.type_parameters`）、同轮 body 候选（`GenericReturnCandidate`/`GenericConstructorCandidate`）。
**缺的事实**：哪个成员体在哪个 BCI 实际消费了该条目（常量池条目与指令消费是两回事）。

Z1 家族重放（本片前、headers 片后基线，release `examples/class_source_file`）：

- `Z1.class`：4× `generic_call_binding_unproved`（`add`/`first`/`map`/`named`）+
  2× `field_generic_body_unproved`（`items`/`index`）——巡查 README 表中"8×/4×"为旧主线
  （headers 合入前）口径；当前主线实测即 4+2，本片以此为准。
- `Z1$Box.class`：头 `class Z1$Box<U>` 与 `U value` 均已投影（headers 片交付），自有池无同类引用，
  与本片无关。

## 2. 使用点清单数据源与证明规则（任务 2.1–2.4）

**清单数据源**：同次选定类、每个实际运行 body 的成员各一次 `scan_member_uses`
（`src/facade.rs`，与既有 `scan_array_helper_uses` 同形）：`MethodIr` 已解码 Code 的
`instructions × operands(effective_opcode, constant_pool_index)` + 同池 `cp_entry`，记录：

- `invoke*`（0xb6–0xb9）→ `(caller 标签, bci, opcode, owner/name/descriptor)`；
- 字段访问（0xb4/0xb5/0xc2/0xc3）→ 同上 + 读位分类（见下）；
- `ldc(_w/_2w)` 的 `MethodHandle` 与整张 `BootstrapMethods` 表（首 body 捕获一次，类级共享）
  → 非正文引用源（ref_kind 1–4=字段、5–9=方法）；
- 完整性：`code.stopped_at.is_none()` 且任何不可解析条目/未知 0xba 目标清 `complete`。

**读位分类**（`classify_field_read_consumers`，同轮 SSA）：`getfield`/`getstatic` 产出的 value
经 `SsaInstruction.reads` 追到每个消费者，按物理 opcode 分类：

- 安全：比较/判空、checkcast、instanceof、monitor、athrow、areturn、pop、**aastore 的被存值位**、
  **putfield/putstatic 的被存值位**（擦除证明 + JVM 校验 ⇒ 投影后仍是不受检合法赋值）、
  **astore 入局部**（呈现层局部类型恒为擦除级——描述符拼写，参数化值对其赋值即原有不受检赋值；
  经局部的成员选择由局部自身擦除类型统治，与字段头无关）；
- invoke 参数位：仅当该位 callee 描述符段为 `Ljava/lang/Object;` 或等于字段自身 descriptor；
- 不安全（→ 字段保持拒绝）：**invoke 接收者位**（最深栈位——参数化接收者改变成员适用性，
  `items.add((Object)x)` 即此类）、经 `dup`/phi/invokedynamic 捕获、链式 getfield 接收者。

**方法绑定证明**（`prove_same_class_method_binding`，类级提交阶段）：

1. 层级闭合：`super_class == java/lang/Object` 且 interfaces 为空（外部同名候选不可能），且
   方法名不在 `is_object_instance_method_name` 表内；
2. 同名兄弟：无 → 无竞争；有 → 仅当**描述符参数个数互异**（且双方均非 varargs）可证
   （调用位参数个数物理固定，适用集不可能交叉）；同 arity/varargs/同描述符重复 → 拒；
3. 完整清单（见下）下，每个同类同名 invoke 位：descriptor == 候选描述符 → 物理目标唯一
   （擦除证明保证投影头与描述符逐位等同，发射文本的实参已由 build 层按池描述符定型，
   适用性保持）；descriptor 指向兄弟 → 兄弟自身绑定，不影响候选；**指向两者皆非 → 不可解释源 → 拒**；
4. 任何指向候选的 MethodHandle/bootstrap 引用（函数位）→ 保守拒；
5. **未消费条目不阻断**：清单完整且无任何位消费 → 发布（spec 场景 1）。

**字段绑定证明**（`prove_same_class_field_binding`）：同名异描述符兄弟字段（合法字节、无源码拼写）→ 拒；
每个同类同名同描述符位：写恒安全；读按上述分类。**清单完整门限**：
`structure_complete && !ended && 全部应跑 body 均有 complete 扫描 && bootstrap 表已读`——
任何未解码/停止 body ⇒ 清单不完整 ⇒ 全部暂候成员整项保持现拒绝文本
（"未检查的使用点不得当成不存在"）。委托在字段循环（先）与方法循环（后）中返回 `Deferred`，
**提交阶段在两循环后统一执行**：每个成员要么经既有 `project_generic`/字段发布路径整体发布，
要么写既有拒绝 marker——无半个头、无伪完整（预算/取消在提交中途传播为请求停止，成员级原子）。

**形状准入**：发布仍走既有判据。本片新增一条已证形状：`ordinary_parameterized_declaration`
接纳 `VoidBody` 候选（完整直线体、参数槽不可变、全参数为类域类型变量、无 throws）——
读侧兼容性即 `report.rs` VoidBody 评注的论证（T 的首界擦除即描述符位 ⇒ 读仍可赋给原上下文；
写侧由候选的参数槽不可变拒绝）。

## 3. Z1 逐项重放（任务 3.2）

| 成员 | 基线 | 本片后 | 依据 |
| --- | --- | --- | --- |
| `index` 字段 | `field_generic_body_unproved` | **投影** `Map<String,List<T>>`，marker 注明 `same-class uses at <init>()V@23` | 唯一使用 = 构造器 putfield（写恒安全） |
| `items` 字段 | `field_generic_body_unproved` | 保持同码同文拒绝 | `add` 体 `this.items.add((Object)…)` 为接收者位读（design Risk 明示：首片不能完整追踪的链保持拒绝、不做文本替换） |
| `add` 方法 | `generic_call_binding_unproved` | **投影** `public void add(T arg1)` + `same-class call binding proved` | 无同名兄弟、层级闭合、main@11/17 物理描述符一致（发射文本未含该调用位——被丢弃的位无从重绑）；VoidBody 形状 |
| `first` 方法 | `generic_call_binding_unproved` | 拒绝推进为 `ordinary_generic_source_unproved`（无同轮返回证明） | 绑定已证；但 `return (Comparable) this.items.get(0)` 在 `T first()` 下不可类型化（Comparable ↛ T），固定正文下投影会发布不可编译源——保持拒绝是 spec"正文类型可表达"的正确执行 |
| `map` 方法 | `generic_call_binding_unproved` | 拒绝推进为 `generic_source_shape_unproved` | 绑定已证；形状无候选（循环体 + 通配参数） |
| `named` 方法 | `generic_call_binding_unproved` | 拒绝推进为 `generic_source_shape_unproved`（嵌套 `$` 返回无成员路径证明） | 绑定已证；形状未证 |
| `Z1$Box` | 头+`U value` 已投影 | **逐字不变** | 自有池无同类引用（Unreferenced 快路径） |

Z1 整类不可编仍如巡查所示（main 的 lambda/伴生与 String→Comparable 转换缺口独立于本片）；
本片只对本门诊断逐项归因。前后全文与 SHA 见 [results/](results/)。

## 4. 既有测试翻转（预期内，均有测试内注释）

| 测试 | 翻转 | 依据 |
| --- | --- | --- |
| `nested_generic_header_projection::separated_nested_generic_units…`（static-bound/non-static） | `N n;`/`U value;` 从保持擦除 → 投影（反射期望随之 `N\nN\n`/`U\nU\n`） | 该测试原注释明写"the next chain, outside this change"；`get()` 的 areturn 位读安全 |
| `ordinary_generic_projection::same_class_overload_caller…` | `bodyOverload` 拒绝 → 投影 | 无同名兄弟、描述符一致；断言改为正向 |
| `generic_method_projection::same_class_caller_to_overloaded_name…` | `choose(T,T,boolean)` 拒绝 → 投影 | 3-arity 候选 vs 2-arity 兄弟 arity 互异；改名 `…_arity_disjoint_overload_proves_projection` |
| `generic_method_projection::parameterized_null_return…self_calls` | Binding 探针（`caller()` 调 `empty()`）拒绝 → 投影；拆出独立正向测试 | 唯一调用位描述符一致 |
| `generic_method_projection::generic_instance_null_return…bindings` | `value()`（0-arity）+ `value(Number)`（1-arity）兄弟 → 投影；Binding 例移出负例集 | arity 互异 |
| `generic_throws_projection::no_body_method_local…` | `target` 拒绝 → 投影（`wait` 仍拒：Object 名单） | 无同名兄弟 + 层级闭合 |
| `generic_constructor_projection::same_class_constructor_binding…` | 泛型构造器拒绝 → 投影 | 唯一 `<init>`、`of` 的 `new` 位描述符一致 |

负例保留/新增：`wait`（Object 名单）、同 arity 兄弟（SCGD）、接收者位字段读（SCGF、
FieldSignatureConflict 既有例不变）、未消费条目（SCGC 变造，先过 `-Xverify:all`）、
预算/取消（新测试 + 既有族）。

## 5. 正反例三方对照（任务 1.2/3.1）

成功子集 SCGA（无竞争调用+VoidBody）/SCGB（字段读写）/SCGE（字段遮蔽）：
原 `.class`、固定 JADX（dev）与 Jarde 三方源码 + SHA 见 [results/three-way/](results/three-way/)；
Jarde 单元 + 独立 Runner 以 `javac --release 8` 重编、`java -Xverify:all` 双侧运行
（重编单元侧 vs 原族侧）逐路径一致，含泛型反射断言（`getGenericParameterTypes`/`getGenericType`
与原类相同）——由 `tests/same_class_generic_binding.rs` 固定。JADX 输出仅作对照不入准入。

## 6. corpus 双腿扫描与门禁（任务 3.3 预备）

双腿扫描见 [results/corpus-scan.txt](results/corpus-scan.txt)（脚本
[results/scan_corpus.sh](results/scan_corpus.sh)，剔除 `ngh/` 与本目录）；门禁记录见
[results/gates.txt](results/gates.txt)。
