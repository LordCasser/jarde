# 任务 1.2 / 1.5：冻结变体与负例（两条 javac 腿）

构建脚本：`sh results/03-fixtures/build-fixtures.sh`（可重跑；`api1/` 是各类**真正编译时**依赖的定义，
只用于编译、绝不入环境；`api2/` 是**环境里那一份**矛盾定义，即两个拒绝格的成因）。

| 腿 | 命令 |
| --- | --- |
| `v8-javac8` | `/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home/bin/javac`（真 javac 8） |
| `v8` | `javac --release 8 -Xlint:-options`（javac 23.0.1） |

fixture 目录：`tests/fixtures/p3-interface-header-projection/`（接口腿）、
`tests/fixtures/p3-nested-parent-projection/`（父类腿）；README 与 sha256 见各自目录。

## 接口腿变体（1.2）

| 格 | fixture | 期望终态（实现后） | 钉在哪 |
| --- | --- | --- | --- |
| 单接口参数化（正例） | `IfaceImpl implements java.lang.Comparable<IfaceImpl>` | 类头 `implements java.lang.Comparable<IfaceImpl>`；`compareTo(Object)` 桥**隐藏**；整类 `javac --release 8` 通过、`-Xverify:all` 经接口引用调用得 `0` | `tests/parameterized_interface_headers.rs::a_parameterized_interface_entry_publishes_its_arguments_and_hides_the_bridge` |
| 多接口部分可证 | `MultiIface implements java.lang.Comparable<MultiIface>, java.lang.Runnable` | 参数化项投影、裸项保持；桥隐藏；三向运行 `1\nrun\n` | `…::a_multi_interface_header_projects_the_parameterized_entry_only` |
| `$` 名实参（`BR$Impl` 形） | 既有 `br-family/v8/BR$Impl.class` | 类头 `implements java.lang.Comparable<BR$Impl>`；桥隐藏；族重编运行 `0\n0` | `tests/class_source.rs::br_family_bridges_admit_through_the_extended_gates` + `…::br_family_recovered_source_recompiles_and_runs_like_the_original` |
| 接口不可解析 | `Unresolved implements MissingApi<Unresolved>`（环境只装 `Unresolved.class`） | 类头保持物理裸拼写 `implements MissingApi`；桥**保持可见**；无 header 拒绝 | `…::an_unresolvable_interface_keeps_the_physical_spelling_and_the_visible_bridge` |
| arity 不符 | `Arity implements ArityApi<Arity>`（环境装的是 2 形参的 `ArityApi`） | 类头投影**拒绝**：`an interface definition contradicts the class Signature's type arguments` | `…::an_interface_definition_that_contradicts_the_signature_refuses_the_header` |
| 类型注解形（零回退） | `TypeUse implements java.lang.@Mark Comparable<TypeUse>` | 与实现前**逐字相同**：裸头 + 桥可见（自有头门 → 静默裸头，不新增拒绝） | `…::a_type_use_annotated_implements_clause_keeps_the_raw_header_and_visible_bridge` |
| 实参拼写边界（记录） | `ErasedCall implements java.lang.Comparable<ErasedCall>`（body 里经接口引用调用） | 类头投影、桥隐藏；body 的擦除调用仍拼 `(java.lang.Object)` ⇒ **该 body 不可编**（调用点实参重定域片的债，响亮记录） | `…::a_body_calling_its_own_erased_contract_records_the_argument_spelling_boundary` |

## 父类腿负例（1.5）

| 格 | fixture | 期望终态 | 钉在哪 |
| --- | --- | --- | --- |
| `$` 父名但父类自身有非 Object 父 | `NestedExtends extends NB$Box<String>`（`NB$Box extends NB$Root`） | 保持拒绝（`direct superclass does not resolve to one proved single-parameter parent definition`） | `…::the_parent_cells_whose_criteria_are_unmet_keep_their_refusals` |
| `$` 父名但 arity 不符 | `ArityExtends extends NB$Twin<String>`（环境装 2 形参 `NB$Twin`） | 保持拒绝（同上文本） | 同上 |
| 多段嵌套名（中段带实参、`$` 连接） | `Multiseg extends MO<String>.Mid`（Signature `LMO<Ljava/lang/String;>.Mid;`） | 保持既有拒绝（`only a single direct Parent<String> superclass is supported…`） | 同上 |
| `$` 父名但无实参（普查对照组） | `BareBox extends NB$Box` | 无投影、无拒绝：裸头 `public class BareBox extends NB$Box` 逐字不变 | `…::a_bare_dollar_named_parent_keeps_the_raw_header` |
| `$` 父名 + 实参（正例） | 既有 `bridge-superclass-precondition/v8/Spec.class`、`br-family/v8/BR$StrBox.class` | 池形参数化头 + 桥隐藏（`cc4b6f11` 四向表闭环） | `tests/class_source.rs::the_parameterized_superclass_header_hides_the_parameter_bridge`、`…::br_family_bridges_admit_through_the_extended_gates` |

## 脚手架自检（纪律要求）

- `render-cells.sh`：先断言真渲染带 `// jarde: presentation of` 自述头（假零防线），并断言不存在的类名
  **不得**被当作源码应答；自检通过后才逐格测量。
- `q2-replay.sh` / `iface-position.sh`：都带"缺失名必须编译失败"的负例自检（脚本能失败才可信）。
- 变体格的 javac 腿在测试内实跑（正例：重编 + `-Xverify:all` 运行 + 与原 class 轨迹逐字比较；
  边界格：实跑 javac 并断言其失败文本关于该调用）。
