## ADDED Requirements

### Requirement: 根方法返回父类直接超类型的匿名类可内联

系统 SHALL 在匿名类分配点位于根方法的直接返回位置、且根方法的声明返回类型 `T` 是该匿名类直接父类 `P` 的**已证一层直接超类型**（`T == P` ∨ `T == P` 的 `super_class` ∨ `T ∈ P` 的 `interfaces`，均按 internal name 全等从 `P` 的 class file 事实读取）时，把该匿名类投影为源级 `new P(…) { … }`，并把根方法的返回位置按 `T` 拼写。`T` 与 `P` **两者**都 SHALL 通过既有的可拼写检查（不含 `$`、每段合法 Java 标识符、与根类同包）。匿名类自身方法体内以匿名类为符号 owner 的 `InvokeVirtual` 自调用（javac 对匿名体内 `this` 上继承/自有方法的常量池拼写）SHALL 由共享 owner 普查在**且仅在**父类路径的直返站点形下放行；接口路径与局部声明初始化位形 SHALL 保持共享普查的既有拒绝（该普查为多路径共享件，放宽须按判别类型显式遏制）。`T` 是 `P` 的间接超类型（祖父类或间接接口）、`T` 非引用类型（数组或基本类型）、`T` 与 `P` 无层级关系、或任一名字不可拼写时，系统 SHALL 保持物理类文本并记录拒绝原因，不得发射不可编译或改变构造器绑定的文本。

#### Scenario: 返回父类直接实现的接口时可内联

- **WHEN** 冻结 fixture `anonymous-top-level`（根方法 `static Renderer create()`，descriptor `()LRenderer;`；child `AnonymousTopLevel$1` 的 `super_class` 为顶层 `Base`；`Base implements Renderer`；分配点直返、super 实参 `choose()` 与捕获局部并存）所属**渲染源集**经 `javac --release 8` 与 `java -Xverify:all`
- **THEN** 呈现为 `static Renderer create() { … return new Base(choose()) { … }; }`——**声明返回位置按接口 `Renderer` 拼写、分配点按父类 `Base` 拼写**；渲染源集编译通过（基线为 exit 1 `找不到符号`，因渲染文本引用 `AnonymousTopLevel$1`）、事件日志与原 class 逐行一致

#### Scenario: 返回类型与父类无层级关系时拒绝

- **WHEN** 根方法的声明返回类型既不是父类本身、也不是父类的 `super_class`、也不在父类的 `interfaces` 中
- **THEN** 保持物理类源码文本并记录拒绝原因，不产出半投影

#### Scenario: 返回类型为间接超类型时拒绝

- **WHEN** 声明返回类型是父类的**祖父类**，或是父类所实现接口**再继承**的接口（即需两层以上层级遍历才能证明）
- **THEN** 保持物理类源码文本并记录拒绝原因——本能力只做一层直接关系，传递闭包属后续切片

#### Scenario: 返回类型非引用类型时拒绝

- **WHEN** 根方法返回描述符的返回段以 `[` 开头（数组）或为基本类型描述符（非 `L…;` 形）
- **THEN** 拒绝，不得误纳为非引用返回类型的形

#### Scenario: 超类型名不可拼写时拒绝

- **WHEN** 声明返回类型 `T` 或父类 `P` 的 binary 名含 `$`（嵌套类型）
- **THEN** 仍按既有 `anonymous_super_source_type_unproved` 拒绝——本能力**不得**放宽该判据（故 `anonymous-capture` 形，其 `T` 与 `P` 均为嵌套名，不在本能力范围内）

#### Scenario: 既有三环的锚与遏制负例零回退

- **WHEN** 环 0/1/3 已交付的锚（`anonymous-super-mixed-direct`、`anonymous-super-args`、`anonymous-super-args-debuginfo`）与环 1 的接口路径遏制负例（`anonymous-local-decl-interface-hold`）经同一投影
- **THEN** 各自渲染**逐字节不变**；遏制负例渲染源码区的 SHA-256 仍为 `1badfcb5b9dcb9a46bf017e3b285073e8424c8143a239f3a6bac9606efc98ce5`（接口路径只认 `DirectReturn` 站点形，本能力改的是父类路径的返回段判据，不得触及它）
