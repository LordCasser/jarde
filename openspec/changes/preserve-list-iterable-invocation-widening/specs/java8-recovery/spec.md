## ADDED Requirements

### Requirement: Java 8 List arguments preserve Iterable calls

当选定 Java 8 运行配置中，调用实参的已呈现源类型精确为 `java.util.List`，而被调用方法的 descriptor 要求 `java.lang.Iterable` 时，系统 SHALL 将该标准平台上溯作为安全引用转换证据，保留原调用并生成可由 Java 8 重编译的参数表达式。该转换 MUST 只求值实参一次并保留参数与调用来源；MUST NOT 因该单一关系引入任意类/接口层级推断或泛型兼容性推断。其它引用关系仍按既有证据门槛处理。

#### Scenario: List result reaches an Iterable parameter
- **WHEN** `Arrays.asList(...)` 的结果作为唯一参数传给 `consume(Iterable)`，且无关 foreach 或重载不参与该方法调用
- **THEN** 原/JADX/Jarde 完整类源码 SHALL 均可按 Java 8 重编译；以 `-Xverify:all` 运行时 SHALL 保留一次 `consume` 调用及其输出，且三者逐行一致

#### Scenario: Unproved reference relations remain refused
- **WHEN** 调用实参与目标形参是 `List`/`Iterable` 之外的不同引用类型，且既有证据未证明其安全上溯
- **THEN** 系统 MUST 保留原调用实参生产者、调用 BCI 与拒绝原因，MUST NOT 只凭目标 descriptor 合成可能失败的引用 cast
