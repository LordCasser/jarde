## ADDED Requirements

### Requirement: 泛型字段写入值的源码适配

系统 SHALL 仅在同类使用清单完整且每一个已呈现写位的实际 RHS 源类型可赋给投影字段类型时发布字段 Signature。物理擦除相同 MUST NOT 替代此证明；拒绝或未完成发布的成员 Signature MUST NOT 被当作参数源类型。不能证明时 SHALL 原子保留物理字段声明与明确拒绝原因，不得发布不可赋值的泛型字段或通过添加未证明的 cast 掩盖缺口。

#### Scenario: 已发布参数与同一类型变量
- **WHEN** TypedSetter<T> 的 put(T) 头与完整正文获证明，字段与参数引用同一 class-scope T，或写入值为 null
- **THEN** T 字段 SHALL 保持投影；双编译腿整类重编译、字段值行为和泛型反射 SHALL 与原类一致

#### Scenario: 擦除形构造器和Object写者
- **WHEN** ObjectSetter/ObjectHold 或 Hold 的实际源参数仍是 Object，且字段 Signature 是 T
- **THEN** T 字段投影 SHALL 拒绝，字段保持 Object、origin和字段身份不变；冻结族整类文本 SHALL 编译，并保持对应已恢复方法的值与调用行为，不宣称保守字段的泛型反射已恢复

#### Scenario: 不同类型变量不能因擦除合并
- **WHEN** CrossSetter<T,U> 将实际发布的 U 参数写到 T 字段，或者同名变量属于不同 binder
- **THEN** 字段 SHALL 保持可靠拒绝，不能因它们都擦除为Object判定源码可赋值

#### Scenario: 所有写位与源码类型必须齐备
- **WHEN** 同一字段同时有安全与未证明写位，或RHS来自改写参数、未证明phi/cast/调用，或成员类型发布未完成
- **THEN** 该字段 SHALL 拒绝整体投影；一个安全写者不得覆盖其他写者的证明缺口

#### Scenario: 既有可赋值参数化字段
- **WHEN** 已冻结的参数化字段读写在其实际源类型下合法，包括已证明的raw到参数化赋值
- **THEN** 已有合法投影 SHALL 不退化；不得把全部字段写位一律拒绝代替源码类型判定

#### Scenario: 停止状态保留
- **WHEN** 扫描、类型证明或候选发布遇到预算或取消
- **THEN** 系统 SHALL 传播既有停止状态并保持成员级原子发布，不将未扫描写位视为不存在
