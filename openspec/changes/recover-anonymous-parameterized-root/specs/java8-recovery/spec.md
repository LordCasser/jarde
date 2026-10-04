## ADDED Requirements

### Requirement: 根方法带捕获参数的匿名类可内联

系统 SHALL 在匿名类分配点位于根方法的**直接返回位置**、捕获值经由**根方法的一个参数**传入、且该参数的角色与 child 构造器的捕获写入一一对应时，把该匿名类投影为源级 `new Base(…) { … }`，并把捕获读取重拼为**根方法的参数名**。根方法描述符的**返回部分仍须恰为父类类型**（`()…Lparent;` 的参数表放宽，返回部分不放宽）；参数多于一个、返回类型非父类本身、分配点非直返位、参数名不可从 AST 取得、或分配点唯一性不成立时 SHALL 保持物理类文本并记录拒绝原因。

#### Scenario: 直返位带捕获参数的纯捕获形可编译且行为一致

- **WHEN** 冻结 fixture `anonymous-super-dispatch`（根方法 `private static Base create(java.lang.String)`，descriptor `(Ljava/lang/String;)LBase;`；分配点在 BCI 0；child `AnonymousSuperDispatch$1` 的 ctor 为 `putfield val$captured` 先于 `invokespecial Base."<init>":()V`，即 super 实参集为空）所属完整源集经 `javac --release 8` 与 `java -Xverify:all`
- **THEN** 呈现为 `return new Base() { … }` 而无物理构造器与 `val$captured` 字段，捕获读取重拼为根方法参数名（`captured`），完整源集编译通过、运行输出与原 class 逐行一致（含"虚调用发生在 `Base` 构造器返回之前且捕获值已可见"这一该 fixture 的既有观察点）

#### Scenario: 无调试信息时同样可恢复

- **WHEN** 同形的 `-g:none` 冻结对照（class 内**无** `LocalVariableTable`）经同一投影
- **THEN** 呈现与 `-g` 腿一致，参数名来自 AST 而非调试信息；两腿的捕获读取重拼结果相同

#### Scenario: 根方法参数表不可证明时保持物理文本

- **WHEN** 根方法带**多个**参数、或分配实参不是单一参数槽、或该参数在根方法内除分配实参外另有消费而消费点不可证、或根方法为实例方法且该形未被证明安全
- **THEN** 保持物理类源码文本并记录拒绝原因，不产出半投影

#### Scenario: 返回类型非父类本身时不因本片放宽

- **WHEN** 根方法返回父类的**超类型**（例如父类 `Base` 实现接口 `Renderer`，而根方法声明返回 `Renderer`）
- **THEN** 仍按既有 `anonymous_super_return_type_unproved` 拒绝——该形属独立的后续切片，本片不得顺带放宽
