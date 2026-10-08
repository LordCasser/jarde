## ADDED Requirements

### Requirement: raw receiver 的字段选择类型

系统 SHALL 在同类实例字段泛型声明投影时，按实际发射 receiver 的源类型验证每个写入访问点：已证明 raw receiver 的非继承实例字段选择字段声明的擦除类型，this 或参数化 receiver 保持其泛型写入规则。系统 MUST NOT 以值的 SSA 起源、物理描述符相同、被拒绝的 Signature 或另一个方法的泛型头替代实际 receiver 源类型证明。无法证明新路径时 SHALL 保留既有可靠结果与明确拒绝，而非添加推测 cast 或改写方法泛型 API。

#### Scenario: raw formal 的实例字段写入
- **WHEN** 泛型类字段声明为 T、T[] 或带上界 T，同类完整方法通过实际发布为 raw 类类型的参数写入该字段，且每个写入值可赋给该访问点的字段擦除类型
- **THEN** 字段 Signature SHALL 发布，完整类 SHALL 重编并保持 JVM 写读行为与字段泛型反射；static 方法 slot 0 SHALL 按参数处理，宽参数槽位 SHALL 按物理参数顺序处理

#### Scenario: 输出中保留的 raw local
- **WHEN** 完整正文实际声明 raw local 并通过该 local 写入同类实例字段，局部声明与使用点源类型可唯一证明且所有写入值合法
- **THEN** 字段 Signature SHALL 发布；SHALL 覆盖从 raw formal 保留的 local，不因仅支持直接参数而丢弃此闭合场景

#### Scenario: 被折成 this 的别名
- **WHEN** 原始 raw local 别名被实际输出折叠成 this，且 Object 写入值不能赋给字段泛型 T
- **THEN** 字段 SHALL 保持擦除与拒绝；SHALL 保持整类可编译及行为，不得使用原始 raw local 或 SSA 值起源放行

#### Scenario: 实际拒绝的方法 Signature
- **WHEN** 写者物理 Signature 声明参数化 receiver 或 method T，但该方法头未发布该 Signature，实际输出 raw receiver，或者另一个 native 方法发布了相似参数类型
- **THEN** 写者 SHALL 仅按自己实际发布的参数类型判断 raw 选择；合法 raw 写入 SHALL 可保留字段 Signature，但 SHALL 不宣称该写者泛型 API 已恢复或借用 sibling 的泛型事实

#### Scenario: this 与不同 binder 保留原有边界
- **WHEN** receiver 是 this，或实际发布的参数化 receiver，而 RHS 是 Object、不同 class/method binder 或未证明值
- **THEN** 系统 SHALL 继续使用原有泛型写证明，不能因字段擦除相同而放行；已有合法 T 写入及已验收构造初始化 SHALL 保持恢复

#### Scenario: 所有访问点与字段身份
- **WHEN** 使用清单不完整、存在未证明写者、字段 owner/name/descriptor 不一致、同名物理字段歧义、继承字段或读消费者不能表达投影类型
- **THEN** 新 raw 路径 SHALL 不绕过既有字段整体拒绝；一个 raw 安全写位不能覆盖另一不安全写位，static 字段 SHALL 不被 raw 实例字段规则擦除

#### Scenario: 预算取消与原子发布
- **WHEN** receiver 事实扫描、源类型适配或字段提交遇到预算/取消
- **THEN** 系统 SHALL 传播既有停止状态并保持成员原子发布，不能以事实缺失或预算拒绝冒充安全成功

#### Scenario: 四腿独立对照
- **WHEN** 对冻结族在真实 JDK8/23 与 debug/no-debug 下验收
- **THEN** 原类、JADX、基线与候选 SHALL 分别完整重编与执行，失败全文 SHALL 保留；候选 classpath SHALL 不包含原 class/jar，泛型反射 SHALL 区分 GenericDeclaration 身份，CLI 返回码或非空文本 SHALL 不替代正确性验收
