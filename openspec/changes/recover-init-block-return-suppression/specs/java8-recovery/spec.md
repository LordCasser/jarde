## ADDED Requirements

### Requirement: 初始化块呈现不含 return 语句

系统 SHALL 在静态初始化块（`<clinit>`）与实例初始化块的语句呈现中抑制 return 语句（尾随与块内形）。方法与构造器中的 return SHALL 逐字不变；初始化块呈现的其余语句 SHALL 与本变更前逐字一致。

#### Scenario: 静态块可重编

- **WHEN** 含 `static { …; return; }` 降低的类（F2 形）三方 Java 8 重编
- **THEN** 初始化块呈现无 return、`javac --release 8` 通过，运行与原 class 一致

#### Scenario: 方法呈现不变

- **WHEN** 输入为普通方法或构造器的 return
- **THEN** 输出与本变更前逐字一致
