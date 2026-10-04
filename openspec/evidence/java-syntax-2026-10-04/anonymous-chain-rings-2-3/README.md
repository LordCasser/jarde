# 5.3 链第二、三环的机制判定（2026-10-04，root）——回答"是否一定要新增机制"

`recover-anonymous-mixed-super-capture`（已验收 `e1c89d57`）落地后，root 实测四处 ctor-reorder fixture **仍全部未内联**，其阻塞链共三环（详见 [recover-ctor-reorder-dispatch-guard/tasks.md](../../../changes/recover-ctor-reorder-dispatch-guard/tasks.md) 的"终局解关联（实测更正）"表）：

| 环 | 阻塞门 | 受影响 fixture | 状态 |
| --- | --- | --- | --- |
| 环 1 | 分配点在局部声明初始化位 + 赋值左端匿名类型名不可拼写 | `anonymous-super-args` | 已立项 `recover-anonymous-local-decl-site`（实施中） |
| **环 2** | `anonymous_super_return_type_unproved`：根方法返回类型必须**恰为**父类 `()Lparent;` | **仅 `anonymous-top-level`**（父类 `Base` 顶层可拼写、返回 `Renderer` 是 `Base` 实现的接口）。**更正**：`anonymous-capture` 不属本环单独可解——其父类 `AnonymousCaptureCases$Base` 与返回类型 `AnonymousCaptureCases$Renderer` 均含 `$`，**先撞 `anonymous_super_source_type_unproved`**，需环 2 + 嵌套父类名可拼写两者（root 2026-10-04 第三次实测以 javap 核实） | **未立项** |
| **环 3** | 同一门：根方法必须**无参** | `anonymous-super-dispatch`（根方法 `create(String captured)` 带参） | **未立项** |

本文件回答 Goal 要求的"是否一定要新增机制"，结论是**两环不同**：环 3 有同文件内的既有先例可复用（**不需要新机制**），环 2 需要一个新的类级可赋值性证明（**需要新能力，但可能可复用既有层级 walk**）。

## 环 3（根方法带参数）：不需要新机制——同文件内已有可复用先例

`src/facade.rs` 的**接口匿名路径**已实现"根方法把捕获值作为参数接收"的完整形态：

```text
facade.rs:3490  let captured_root_parameter = if capture_descriptor == Some(b"D") { … }
facade.rs:3491      expected_descriptor = [b"(D)L", interface_name, b";"]      ← 根方法带一个 double 参数
facade.rs:3545      let return_matches = if captured_root_parameter.is_some() {
                        root_method.descriptor.0 == [b"(D)L", interface_name, b";"]   ← 带参分支
                    } else { root_method.descriptor.0 == expected_return }            ← 无参分支
facade.rs:3802      消费 (parameter_slot, parameter_name)
```

其判据完整可读（`facade.rs:3522-3530`）：`root_method.descriptor == expected_descriptor`、`access_flags & ACC_STATIC`、`matching_scan.complete`、`site.verified`、`site.argument_bcis.len() == 1`、`site.argument_parameter_slots == [Some(0)]`、`allocation_argument_bcis == Some(site.argument_bcis)`、`parameter_name.is_some()`。

**故环 3 的实现路径是把这套既有判据移植到父类路径**（`project_class_source_anonymous_super` 目前**完全没有**带参分支：`facade.rs:4698-4699` 只有 `expected_return = "()Lparent;"` 一条恰等检查），并且：

- **不再硬编码 `b"D"`**。`recover-anonymous-mixed-super-capture` 已把 double 专属的 `expr.presented = Some(Type::Double)` 换成 `ProvedCapturedParameterRead.parameter_presented`（携带真实呈现类型），正是为跨类型复用做准备；环 3 应沿用该字段，把接口路径残留的 `b"D"` 特化（`facade.rs:3455`、`3490`、`3771`、`3800`、`3896` 五处）一并泛化——但**须分片**：先做父类路径的带参形（解锁 `anonymous-super-dispatch`），接口路径的 `b"D"` 泛化另立一片（它有 6/6 已验收测试守着，不宜与父类路径混改）。
- 锚 `anonymous-super-dispatch` 的形是 `private static Base create(final String captured)` → `return new Base() { … }`，即**根方法带参但父类构造器无参**（捕获值只进 `val$` 字段，不进 super 实参）。这比 `anonymous-super-args` 更简单：super 实参集为空，划分退化为"全部参数都是 capture 角色"。**注意**：此形的捕获是 `String` 而非 `double`，故它同时验证了 `parameter_presented` 泛化的必要性。

**风险与不变量**：放宽"根方法无参"会让更多方法成为投影候选，故必须保持 `recover-anonymous-mixed-super-capture` 已建立的两条不变量——**已证分配点唯一**（`_anonymous_return_sites.len() == 1` 语义）与**捕获值来源可证**（该参数槽恰一次写，或参数未被写）。另须遵守 `recover-anonymous-local-decl-site` 的判据 5：站点扫描是**接口路径与父类路径共享**的（`class_source_direct_return_new` → `class_source_anonymous_return_site` → `_anonymous_return_sites`；接口投影在 `facade.rs:3293` 解构该向量后于 3367 委派父类投影），任何对站点形的放宽都会同时作用于两路径，**必须显式遏制**（站点判别位），否则接口投影会在无取证无测试的情况下被静默激活——root 已实测证明这种激活确实会发生（见 [interface-path-activation-probe](../interface-path-activation-probe/README.md)：`state` 由 `absent` 变 `refused`）。

## 环 2（根方法返回父类的**超类型**）：需要新的证明能力

锚形（root 逐行读源确认）：

```java
interface Renderer { … }
static abstract class Base implements Renderer { … }     // Base 是 Renderer 的实现
private static Renderer baseArgumentAndCapture() {       // 声明返回类型是接口 Renderer
    return new Base(choose()) { … };                      // 分配的是 Base 的匿名子类
}
```

`facade.rs:4698` 要求 `root_method.descriptor.0 == "()LBase;"`，而实际是 `()LRenderer;`，故拒绝。**放宽它不是删掉检查，而是要把检查换成一个更强的证明**：

> 声明返回类型 `T`（此处 `Renderer`）与匿名类的直接父类 `P`（此处 `Base`）之间必须存在**已证的**可赋值关系 `P <: T`（`P` 是 `T` 的子类，或 `P` 直接或间接实现接口 `T`），且该关系须在同轮快照内可解析、层级可完整遍历。

否则会出现两类新风险：
1. **静默改变绑定**：把左端/返回位置拼成 `Renderer` 而实际分配 `Base` 的匿名子类，若可赋值关系未被证明，重编后的源码可能选择不同的重载或改变成员解析。
2. **不可编译**：若 `T` 与 `P` 其实无关（例如原码里 `T` 是 `P` 的父类但快照内无法解析 `P` 的层级），发射的文本无法重编。

**现有设施盘点（root 已查，结论：无直接可用的类级可赋值性证明）**：

| 设施 | 位置 | 能否用于环 2 |
| --- | --- | --- |
| `prove_snapshot_hierarchy_widenings` | `facade.rs:17743` | **不能**。它证明的是方法 IR 内**值**的快照层级放宽（为 `recover-bridge-admission-gates` 的协变返回擦除门而建），输入是 `MethodIr`，不回答"类 P 是否可赋给类型 T" |
| `subtype_of` | `crates/jarde-jvm/src/members.rs:1490` | **概念匹配但不能直接复用**。它确实做类层级 walk（`caller.supertypes()` + `Layers` 展开 + 命中 `declaring` 判定），但为 `fn`（私有）且绑定 `HeaderClosure`/`Search`/`ClassSite` 这套**成员访问检查**机制，facade 的匿名投影拿不到这些上下文。可选做法是把该 walk 的层级遍历能力**提取为可复用件**（在 `jarde-jvm` 或 reader 层暴露一个"给定两个类名，证明子类型关系"的入口），再由环 2 消费——这算**新增一个小的共享能力**，但不算新机制族（层级 walk 的语义与缓存策略已存在，只是所有权与可见性需调整） |
| `prove_direct_generic_superclass_parent` | `facade.rs:13593` | 部分可参考：它已用 `resolve_class_source_dependency_read_raw` 解析父类定义并读其 `super_class`/`interfaces`，说明"解析一个类的直接超类型集合"这条通道是通的；但它只做**一层**且服务于泛型父类投影，不含传递闭包与接口继承 |
| `prove_no_body_generic_hierarchy` | `src/class_source.rs:3133` | **不能，且方向相反**。它不 walk 层级，而是**保守拒绝**：`!matches!(no_body_kind, Abstract) ‖ class_flags&(ACC_ANNOTATION\|ACC_ENUM)≠0 ‖ class_internal.contains('$') ‖ class_superclass≠Some("java/lang/Object") ‖ !class_interfaces.is_empty()` 即 `Err(generic_inherited_contract_unproved)`——只接受"Object 父类 + 无接口"的顶层类 |
| `hierarchy_complete` | `crates/jarde-jvm/src/members.rs:234` | **不能（易误判，root 已踩）**。它是 `MemberOutcome` 上的**成员解析搜索读取完备性**（`self.unread.is_empty()`，"本次 member 搜索进入的每个层级分支是否都读到"），服务于 `resolve_member`/`resolver.rs`，**不是**类级子类型关系判据 |
| `scan_hierarchy` | `crates/jarde-query/src/xref/metadata.rs:546` | **不能**。只做 xref 元数据记录（对 `super_class` 与各 `interface` 各发一个 `XrefOperation::SuperClass`/接口站点），**单层、不传递、不做子类型判定**；`jarde-query` 亦无公开的超类型查询 API（`grep -rnE "pub fn .*(supertype\|hierarchy\|ancestors\|implements)" crates/jarde-query/src` 无命中） |

**故环 2 的判定是**：需要一个**类级可赋值性证明**入口（传递闭包 + 接口继承 + 快照内可解析性 + 层级不完整时拒绝）。实现上有两条路：
- **优先**：把 `members.rs::subtype_of` 的层级 walk 提取为共享能力，环 2 消费它（避免第二套层级遍历，符合"不新增平行状态"）。
- 次选：在 facade 内用 `resolve_class_source_dependency_read_raw` 自建一层层 walk。代价是与 `members.rs` 形成两套层级遍历，**root 不推荐**（本会话已两次因"两套逻辑漂移"被迫加同形判据：`bridge_superclass_contract_generic` 刻意与投影的 `parameterized_superclass` 计算同形以防漂移）。

**环 2 的验收锚（2026-10-04 root 第三次实测后更正）**：**只能是 `anonymous-top-level`**（`static Renderer create()`，父类 `Base` 顶层可拼写、返回类型 `Renderer` 是 `Base` 直接实现的接口）——其完整源集 `javac --release 8` 须从当前状态转为 exit 0，且 `java -Xverify:all` 事件日志与原 class 逐行一致。**`anonymous-capture` 不能单独作为环 2 的锚**：其父类 `AnonymousCaptureCases$Base` 与返回类型 `AnonymousCaptureCases$Renderer` 均含 `$`，先撞 `anonymous_super_source_type_unproved`（父类须为"同包、可直接拼写的源码类型"），故须待"嵌套父类名可拼写"能力落地后才可能被环 2 解锁——把它当环 2 锚会让实现者误判自己已完成而实际仍拒绝。

**MVP 判据（root 降成本的实测发现）**：两 fixture 中父类→返回类型都是**一层直接**关系（`Base implements Renderer`，javap 核实 `abstract class AnonymousCaptureCases$Base implements AnonymousCaptureCases$Renderer`；`abstract class Base implements Renderer`），而投影本就解析父类 class file（`prove_direct_generic_superclass_parent` 已用 `resolve_class_source_dependency_read_raw` 读父类定义，`ClassMemberFacts.interfaces` 可直接取到）。故**环 2 的 MVP 可只做一层判据**（声明返回类型 ∈ 父类的 `super_class` ∪ `interfaces`），把传递闭包留作后续，避免一上来就新建层级 walk 共享件。

**负例必须包含**：声明返回类型与父类**无关**的形（须响亮拒绝，不得发射不可编译文本）、父类 class file 在快照内**不可解析**的形（拒绝而非猜测）、以及返回类型为**子类型**（比父类更窄）的形——后者在 Java 源码里不可能由 javac 生成，若出现说明输入非 javac 产物，应拒绝。

> **一处 root 自查更正**：本文件早先版本写"层级在快照内不可完整解析的形（`hierarchy_complete` 为假时拒绝）"。root 复核发现 `crates/jarde-jvm/src/members.rs:234` 的 `hierarchy_complete` 是**成员解析搜索的读取完备性**（`self.unread.is_empty()`，即"本次 member 搜索进入的每个层级分支是否都读到了"，服务于 `resolve_member`），**不是**类级子类型关系的完备性判据，不可直接用于环 2。上文已改为按"父类 class file 可解析"表述。

## 优先级建议（按 Goal "优先大颗粒里程碑 + MVP 思维"）

1. **环 1**（在飞）：解锁 `anonymous-super-args`，且是链中唯一涉及 emitter 新能力（左端重拼）的一环，风险最高，宜先单独落地并稳定。
2. **环 3**：**先于环 2**。理由：不需要新机制（移植同文件既有判据 + 沿用已泛化的 `parameter_presented`），锚形更简单（super 实参集为空），MVP 成本最低，且能顺带验证 `parameter_presented` 跨类型泛化的正确性。
3. **环 2**：最后。需要先做"提取共享层级 walk"的架构决定（涉及 `jarde-jvm` 的所有权与可见性调整），颗粒最大、波及面最广，且其正确性依赖一条新的证明不变量（可赋值性），不宜与前两环混做。

**三环不得合并为一片**：环 1 改站点扫描 + emitter，环 3 改根方法门 + 泛化 `b"D"`，环 2 需要新的共享层级能力。三者触及的所有者不同（emitter / facade 门 / jarde-jvm 层级件），合并会使单片验收无法定位失败原因，也违背本会话反复验证的"一片一个不变量"纪律。
