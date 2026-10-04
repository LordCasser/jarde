## ADDED Requirements

### Requirement: 混合参数匿名类在局部声明初始化位置可内联

系统 SHALL 在混合参数匿名类的分配点位于局部声明初始化位置（分配表达式为唯一初始化值的局部声明）且该声明的左端类型名可被证明重拼时，把它投影为源级 `new Base(args…) { … }`，并把赋值左端的匿名子类类型名重拼为父类源码名；站点扫描、根方法门或左端重拼任一不可证明时 SHALL 保持物理类文本。

#### Scenario: 赋值初始化形完整源集可编译且行为一致

- **WHEN** 冻结 fixture `anonymous-super-args/AnonymousSuperArgs$1`（`main` 内 `AnonymousSuperArgs$1 local2 = new …`，两父类实参与一个捕获局部并存）所属完整源集经 `javac --release 8` 与 `java -Xverify:all`
- **THEN** 编译通过（基线退出 1）、事件日志与原 class 逐行一致，呈现为 `Base local2 = new Base(text(…), number(…)) { … }`，左端类型名为父类源码名

#### Scenario: 左端重拼不可证明时保持物理文本

- **WHEN** 赋值左端类型名不可拼写（池形含 `$` 且被结构反射消费、数组形、或声明形态不完整）
- **THEN** 保持物理类源码文本并记录拒绝原因
