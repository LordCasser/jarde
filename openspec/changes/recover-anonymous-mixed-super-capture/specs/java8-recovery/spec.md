## ADDED Requirements

### Requirement: 混合参数匿名类可内联为源级匿名表达式

系统 SHALL 在匿名类的每个物理构造器参数角色被唯一证明时，把**直返位置的**分配点投影为源级 `new Base(args…) { … }`：流入直接父类构造器实参位的参数保持左到右序发射为 `args`，流入 synthetic 捕获字段 `putfield` 值位的参数按既有词法替换通道隐藏为其分配点根方法局部（或参数）引用。捕获字段声明、物理构造器与字段写入 SHALL 被隐藏（由 javac 从匿名类语法重建），pre-super 写入序不再需要呈现层重排。任一参数无消费、被两类角色消费、被同一角色消费两次、super 实参序与物理参数序不一致、捕获字段二次写入、分配点不唯一或正文非 `structured` 时 SHALL 保持物理类文本。

#### Scenario: 混合形完整源集可编译且行为一致

- **WHEN** 直返方法中的 `Base` 匿名子类（两个真实父类构造实参 `text(…)`、`number(…)` 与一个捕获局部 `captured` 并存，物理构造器 `(Ljava/lang/String;ILjava/lang/String;)V`，无 `this$0`；冻结 fixture `anonymous-super-mixed-direct/`）所属完整源集（根类 + `Base`，`$1` 保持可查询但不入源集）经 `javac --release 8` 与 `java -Xverify:all`
- **THEN** 编译通过（该形verbatim呈现退出 1）、`event()` 事件日志与原 class 逐行一致（末行含捕获值本身而非 `null`），且呈现为 `new Base(text(…), number(…)) { … }` 而无物理构造器与捕获字段，捕获读取以根方法局部名重拼

#### Scenario: 既有匿名片逐字不变

- **WHEN** 输入为无捕获的父类实参形（`inline-proved-anonymous-super-arguments`）、词法 `Inner.this` 形（`recover-proved-anonymous-inner-this`）或接口形单捕获（`recover-proved-anonymous-local-capture`）
- **THEN** 输出与本变更前逐字一致

#### Scenario: 角色划分失败时保持物理文本

- **WHEN** 任一物理参数无消费、被 super 与 capture 两类角色同时消费、super 实参序与物理序不一致、捕获字段二次写入、存在第二分配点或第二个 `val$` 捕获字段
- **THEN** 保持物理类源码文本并记录拒绝原因，不产出半投影

#### Scenario: 赋值初始化形保持物理文本（5.3 后续切片）

- **WHEN** 混合形分配点位于局部声明初始化位置（如冻结 fixture `anonymous-super-args/AnonymousSuperArgs$1` 在 `main` 中的 `AnonymousSuperArgs$1 local2 = new …`）而非直返位置
- **THEN** 保持物理类文本：站点扫描不接受该形态、根方法返回门不放宽、赋值左端的匿名类型名不重拼；该形态（站点扫描扩展 + 局部声明初始化形 + LHS 声明类型重拼）由后续切片 `recover-anonymous-local-decl-site` 承接，不属本片
