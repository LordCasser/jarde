## ADDED Requirements

### Requirement: lambda 呈现可重编（体内联或非冲突伴生名）

系统 SHALL 对直线体的伴生 `lambda$…` 方法（单 return 表达式或直线序列、形参按位只读、单用途）将其体内联进 lambda 表达式并隐藏伴生方法；复杂体伴生 SHALL 重命名为非冲突名（`$jarde` 后缀）并同步调用位。伴生多用途形状 SHALL 保持既有呈现；无 lambda 类输出 SHALL 逐字不变。

#### Scenario: 含 lambda 类可重编

- **WHEN** Y1 形（四 lambda 位点 + 伴生）三方 Java 8 重编
- **THEN** lambda 表达式呈现伴生体（或非冲突伴生名），`javac --release 8` 通过，运行与原 class 逐字一致（`hi!`/`45`/`[b, aa]`/`8`）

#### Scenario: 无 lambda 与负例不变

- **WHEN** 输入为无 lambda 类或伴生多用途形状
- **THEN** 前者与本变更前逐字一致；后者保持既有呈现
