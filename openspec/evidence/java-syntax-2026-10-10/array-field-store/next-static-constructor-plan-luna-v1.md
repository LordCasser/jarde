# EM-18 下一片：数组字段初始化位置基线计划

## 推荐顺序

下一条最小完整类基线选 JADX `TestArrayInitField.TestCls`，对应测试 `TestArrayInitField.test()`，源文件：
`/Users/lordcasser/workspace/testzone/jadx/jadx-core/src/test/java/jadx/tests/integration/arrays/TestArrayInitField.java`。

原测试同时含 `static byte[] a = new byte[] { 10, 20, 30 }` 与实例字段
`byte[] b = new byte[] { 40, 50, 60 }`，断言两者分别呈现为字段声明初始化器。它紧邻已验收的
`TestArrayInit.test2`：后者证明方法体内 fresh 数组最终写入实例字段时，数组证明可组合到 `putfield`；
本测试把消费位置移到字段初始化阶段，能区分 `<clinit>` 与构造器初始化，而不引入新数组类型或元素语法。

建议基线保留这两个字段位置，并在数组元素中用小型有序 `mark(int)` 记录可见副作用：首次读取静态字段
后断言静态字段按声明顺序完成且 trace 顺序精确；随后构造实例并断言实例字段初始化发生在构造器正文之前，
而且静态 trace 不会重跑。数组内容也逐项观察。首片先测正常完成路径；静态初始化抛错会使类进入初始化
失败状态、需要独立加载器/独立类来隔离，建议留作后续而不混入这个最小基线。预期值只能由原始 Java 8
class 的实际运行记录确定，不能从本计划推定。

## 现有机制与边界

- `ArrayInitializers::prove` 已证明有序分配/元素写入闭合链、索引、别名、异常与消费者约束；
  `array-field-store/README.md` 的 root 验收进一步证明它可与方法体的实例 `FieldWrite` 消费点组合。
- `<clinit>` 同轮事实已有有序 `ClassInitializerStep::FieldWrite`、字段读取身份及 RHS AST；静态字段访问计划也
  能区分 `putstatic`。EM-06 报告记录了普通类静态字段候选的结构证明和 writer，但当前普通类投影入口为
  `NotApplicable`，已有启用范围是接口。故值得先测“已知 array proof + 已有有序静态 field-write 事实”能否形成
  一组声明初始化器；这只是候选方向，尚未证明这里存在差距。
- 实例字段 initializer 的 JVM 写入位于构造器 `super`/`this` 链之后；现有字段机制对
  `UninitializedThis` 有专门身份约束，初始化投影也已经消费数组事实。`TestArrayInitField` 的实例字段可作为
  同类控制臂，检查字段呈现是否跨越构造器接缝，但不要据此把已验收的普通方法体 `putfield` 等同于构造器字段提升。

## 判定约束

把原 class、fresh JADX 与 Jarde 的**完整类**分别用冻结 Java 8 工具重编并运行；比较原程序的 stdout、stderr、
exit 作为语义 oracle，并观察完整源码是否把两个字段放回各自合法的声明初始化位置。不得手工改写任一
反编译源码。静态副作用顺序、实例初始化相对构造器正文的次序、字段数组内容都是不变量；任何候选只有在
独立重编和运行保持这些不变量后才算证据。若输出仍为语义正确的完整 `static` 块或构造器赋值，记录为
呈现差异，不能仅凭 JADX 的代码形状断言机制缺失。异常静态初始化、try/catch 与多构造器保持后续边界。

本计划不表示 EM-18 已完成，也不确认缺口；完成本基线前不立实现 spec。
