## ADDED Requirements

### Requirement: 结果丢弃的分配 SHALL 呈现为独立语句

当一次对象分配的实例值**没有任何读者**（零指令读取该实例——分配作为独立语句、仅构造器副作用生效）且其构造器调用按既有链可证（实参链证明不受影响）时，系统 SHALL 把该分配呈现为独立语句 `new Owner(args…);`——语句与其前后语句按字节码序排列，构造器副作用序保持。

实例读者计数 **大于 1** 时 SHALL 保持既有拒绝（"一个构造实例只有一个 Java 拼写位"不变量的多拼写位方向）；恰为 **1** 时既有表达式路径 SHALL 逐字不变；`single_use_at` 判据本体 SHALL 零改动。多读者形、丢弃后复用 slot 的复杂形不在本能力范围。

#### Scenario: 丢弃分配恢复

- **WHEN** `static void discarded(){ new N(); System.out.println("after"); }`（N 的构造器可证、结果零读者）经 `class-source` 呈现
- **THEN** 方法体呈现 `new N();` 独立语句 + `"after"` 打印（序保持）、0 引注；整类渲染源集 `javac --release 8` exit 0、`main` 输出与原 class 逐行一致

#### Scenario: 消费形零回退

- **WHEN** 同类中赋值形（`N n = new N();`）、链式形（`new N().hi();`）、实参形（`takes(new N());`）经呈现
- **THEN** 既有呈现逐字不变——只有零读者分支新增

#### Scenario: 多读者仍拒绝

- **WHEN** 分配实例被多于一处真实消费（既有负例族）
- **THEN** 保持既有拒绝——本能力不触碰 >1 方向

#### Scenario: 副作用构造级联解锁

- **WHEN** `new C();`（C 为三级 this()/super() 委派链构造器、含字段初始化副作用）作为丢弃语句出现在方法中
- **THEN** 该语句恢复呈现（委派链与字段初始化序由构造器证明既有域保证）
