## ADDED Requirements

### Requirement: 已证明的匿名接口参数捕获

当一个完整 Java 8 根方法直接返回唯一匿名接口实例，且其物理 child 的唯一 synthetic-final `double` 字段已被证明只保存创建时的根方法参数值，系统 SHALL 将匿名体中的所有该字段读取恢复为同一根参数引用，并原子地发布完整匿名体源码。投影 MUST 保持捕获值的求值次数和创建时刻、覆盖方法的效果以及数值位模式；物理 child 仍 MUST 可独立查询。

#### Scenario: 单个 double 参数值捕获

- **WHEN** 唯一 `Runnable` 匿名体的构造器只把根方法 `double` 参数写入 synthetic-final 字段，覆盖方法读取该字段，所有 owner 使用和方法恢复均完整
- **THEN** 根类 SHALL 输出直接返回的 `new Runnable() { ... }` 与参数引用，不再引用源级物理 `Capture$1`；原始、固定 JADX 与 Jarde 的完整源码 SHALL 在 Java 8 重编、`-Xverify:all` 下对普通值和负零值一致

#### Scenario: 捕获链不能闭合

- **WHEN** 创建点不再传入该根参数、构造器参数槽或字段 descriptor 不匹配，字段还有额外写入/跨类使用，或覆盖方法存在未证明的读取
- **THEN** 系统 MUST 拒绝源级匿名内联并保留可审计的物理 child 和拒绝原因，不得凭 synthetic 字段名推测参数身份

#### Scenario: 恢复停止

- **WHEN** 依赖读取、证明、源码呈现或整类提交因预算或取消停止
- **THEN** 系统 MUST 不发布局部匿名体、删除局部物理声明或改写其中部分字段读取；停止状态 MUST 可见
