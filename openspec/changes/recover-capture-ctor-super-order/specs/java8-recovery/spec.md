## ADDED Requirements

### Requirement: 捕获形伴生构造器的 super 次序 SHALL 可编译

当匿名/内部类伴生的构造器在字节码中把捕获字段赋值（`this.val$x = argN`）发射在 `super()` 调用**之前**、且满足（1）super 前语句**全部**是捕获字段赋值、（2）super() 实参不读取任何被赋值的 val$x 时，系统 SHALL 把渲染次序重排为 `super(); this.val$x = argN; …实例块语句…`——使渲染源在标准 Java（含 8）下合法。

任一条件不满足（super 前有非捕获语句、或 super 实参依赖捕获值）时 SHALL 保持既有字节码次序呈现（渲染头声明 not claimed to compile——响亮）。无捕获形伴生的既有呈现 SHALL 逐字不变。

#### Scenario: 捕获形双括号恢复

- **WHEN** `new ArrayList<String>() {{ add(s); }}`（捕获 `s`）的伴生 `DB$2` 与宿主 `DB` 渲染拼接后经 `javac --release 8` 编译
- **THEN** 编译 exit 0（修复前因 super 前赋值 exit 1）；`main` 输出与原 class 逐行一致（`2/z`）

#### Scenario: 无捕获形零回退

- **WHEN** `new ArrayList<String>() {{ add("a"); }}`（无捕获）的伴生经呈现
- **THEN** 既有呈现（super 后内联）逐字不变

#### Scenario: 依赖形保持现状

- **WHEN** super() 实参读取捕获值（合成负例）
- **THEN** 保持字节码次序呈现（不重排）——重排需数据流证明，非语法猜测
