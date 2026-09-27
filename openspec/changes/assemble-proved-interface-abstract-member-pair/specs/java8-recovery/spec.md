## ADDED Requirements

### Requirement: Proven declaration-only interface and abstract member pair

当一个 Java 8 普通根类在同一选定环境中恰好有一个直接静态接口成员和一个直接静态抽象类成员，并且两个 child 与根的 `InnerClasses` 关系、物理定义、合法声明及无构造使用全部获证时，系统 SHALL 在一次根源码投影中同时输出两份嵌套声明。接口抽象方法 SHALL 保持无 Code 的分号声明；抽象类构造器 SHALL 使用成员简单名。根/child 物理报告及两个声明的派生来源 SHALL 保留。任何 child、预算或停止证明不完整时，系统 SHALL 不发布半组源码。

#### Scenario: Two direct declaration-only members

- **WHEN** 根类仅有无成员使用的默认构造器及准确双向关系的 `public interface I` 与 `public static abstract class A`，`I` 只有两个无 Code 的 public abstract 方法，`A` 只有默认构造器和一个无 Code 抽象方法，二者没有字段或 Signature
- **THEN** 根源码同时包含合法的 `I` 与 `A`，和同一外部 Runner 一起 Java 8 重编、验证运行及反射结果与原 class 一致

#### Scenario: One member is not proved

- **WHEN** 任一 child 缺失或定义歧义、双方关系冲突、接口方法不合法、第三直接 child 出现、额外字段/Signature/使用点出现，或恢复因预算/取消停止
- **THEN** 系统 SHALL 保留可查询的物理事实及明确拒绝/停止结果，且根源码不出现仅一个被投影的成员

#### Scenario: Existing family slices remain separate

- **WHEN** 输入属于既有唯一静态抽象成员、构造型静态成员或泛型 `Generic.A` 形态
- **THEN** 本双成员证书 SHALL 不改变前两者的已证源码和来源，也不放行 `Generic.A` 的未证 Signature/bridge
