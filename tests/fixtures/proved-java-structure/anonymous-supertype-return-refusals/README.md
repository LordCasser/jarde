# 环 2（根方法返回父类直接超类型）的拒绝与遏制边界

`recover-anonymous-supertype-return` 的冻结负例与遏制探针。源码以 `javac --release 8 -g:none`
编译后冻结；每个子目录是独立形态。各形的"前/后拒绝码"以实现前后的主线二进制实测为准
（证据 `openspec/evidence/java-syntax-2026-10-04/recover-anonymous-supertype-return/`）。

## 冻结负例（javac 自然可产形）

| 目录 | 形态 | 边界 |
| --- | --- | --- |
| `indirect-supertype-return/` | 根方法声明 `Top`，`Mid extends Top`、`Base implements Mid`——`Base <: Top` 需两次层级遍历 | 本能力只做一层（父类自身、其直接父类、其直接实现接口），两层保持拒绝：`anonymous_super_return_type_unproved`（前后同码） |
| `nested-supertype-return/` | 根方法声明嵌套 `Holder.Marker`（binary 名 `Holder$Marker` 含 `$`），它恰在父类 `Base` 的直接接口里——层级事实一层可证，但 `$` 名不可拼写 | 既有 `anonymous_super_source_type_unproved` 不放宽（实现前为 `anonymous_super_return_type_unproved`：门未放宽到超类型；实现后拒绝点移到可拼写判据——两码均为响亮拒绝，渲染保持物理文本） |
| `interface-self-invocation/` | **遏制探针**：接口路径匿名体 `toString()` 调用继承的 default `shout()`，javac 产生 `invokevirtual 自身.shout` ——符号 owner 为匿名类自身的方法调用 | 接口路径共享 owner 普查且不接受环 2 的自调用允许臂（判别参数为限制值）：`anonymous_interface_child_additional_use`，呈现/状态逐字节不变——本环不得顺带打开接口路径（环 1 判据 5 同型遏制） |

## CI 合成探针（不冻结为 fixture——javac 产物不可表达）

`tests/anonymous_supertype_return.rs` 以冻结锚 `anonymous-top-level` 的根类字节为底，
定向改写 `create` 的描述符（等长替换，保持其余字节与常量池不动），断言响亮拒绝：

- **(a) 无层级关系**：`()LRenderer;` → `()LWrongOne;`（等长，同包可拼写但与父类无任何层级关系）
  → `anonymous_super_return_type_unproved`。此形在合法 Java 里不可由 javac 产生
  （声明返回类型必须可由分配类型赋值——这正是本门所证的纪律），故不冻结为语料。
- **(c) 数组返回**：`()LRenderer;` → `()[LWrongOn;`（等长，数组描述符）
  → `anonymous_super_return_type_unproved`（非 `L…;` 引用返回一律拒绝）。同长度约束下无法
  构造可解析的基本类型返回描述符（`()J` 等仅 3 字节）；基本类型与数组命中同一 match 臂
  （`DescriptorComponent::is_array()` ∨ `object_name() == None` → 同一拒绝），数组探针即覆盖该拒绝路径。

## 与其它负例的关系

- 跨类使用（child 名被其它类引用）由既有 `anonymous-cross-class-use` 守卫；
- child 体内自分配（`new` + child owner）由既有 child-body 自分配负例守卫——环 2 的允许臂
  只放行 **`InvokeVirtual` Method 符号**（私有自 helper 的 `invokespecial` 自调用虽为 javac 8
  可达形，按收紧判据**未开放**，保持拒绝并登记——不为论证存疑的分支放宽生产允许集）；
  Class 符号（分配点）不受影响；
- 环 3 的五个负例中 `supertype-return` 经 root 追认转为本环对照正例（其重归类——环 3 的
  spec/tasks/fixture README 行/账本——留 root 执行；实现者的裁定复现与归因存档见证据目录
  `openspec/evidence/java-syntax-2026-10-04/recover-anonymous-supertype-return/` 的
  README §二与 `root-replies-verbatim.md`），其余四个仍逐字拒绝。
