## ADDED Requirements

### Requirement: Complete class source projects proved enum switch labels

当 Java 8 完整类源码请求的同一运行环境证明枚举分派表的真实定义、初始化写入、枚举常量对象的唯一身份、连续 ordinal、`values()` 数组来源与使用点一致时，系统 SHALL 将对应整数分派呈现为枚举 selector 与常量 `case`。投影 MUST 依据真实表写入而非合成字段名、枚举声明顺序或整数 case 顺序；物理类、方法、表字段和原恢复报告 MUST 保持可审计。

#### Scenario: Ordinary compiler map

- **WHEN** 分派方法读取已解析的表，表初始化把 `RED.ordinal()` 写为 1、把 `BLUE.ordinal()` 写为 2，且两项正是该 switch 使用的 case 键
- **THEN** 完整类源码 SHALL 写出 `switch` 的 `RED` 和 `BLUE` 标签，重新编译的返回值、调用次数与 trace SHALL 与原 class 相同

#### Scenario: Valid map values are swapped

- **WHEN** 同一合法 class 仅把表的两条整数写入交换，原 switch 方法及字段名不变，JVM 验证执行将 RED/BLUE 的结果交换
- **THEN** 输出标签 SHALL 随真实写入交换，重新编译后也 SHALL 交换结果；MUST NOT 保持原常量名顺序而产生错误程序

#### Scenario: Aliased enum fields

- **WHEN** 一个 JVM verifier-valid enum 的两个 `ACC_ENUM` 字段指向同一对象，导致 helper 对同一 runtime ordinal 的先后写入相互覆盖
- **THEN** 系统 MUST 拒绝标签投影并保留整数分派；常量字段名、flags、`values()` 签名或 helper 映射本身都不能替代对 enum `<clinit>` 对象身份与 `(String,int)` 构造器 ordinal 转发的证明

#### Scenario: Irregular enum values array

- **WHEN** 一个 verifier-valid enum 保留正常常量字段与构造器，但其数组工厂返回 null、长度不足的数组、元素顺序改变，或公开 `values()` 有额外调用/返回另一数组
- **THEN** 系统 MUST 拒绝标签投影并保留整数分派；必须证明 `<clinit>` 实际调用的数组工厂、所发布数组与公开 `values()` 的完整路径，不能仅凭其签名、字段名或传入但未使用的方法 IR 放行

#### Scenario: Null selector and selected-arm effects

- **WHEN** selector 为 null，或各 case 调用有不同的副作用
- **THEN** 投影源码 SHALL 保持 null 路径的异常类别和发生前的副作用，并只执行命中 case 的调用一次

### Requirement: Enum label projection is bounded and refuses unknown maps

跨类映射和 enum 常量身份的查找、证明和源码发布 SHALL 遵守现有环境选择、预算、取消与证据契约。无法证明的映射或对象身份 MUST NOT 被猜成枚举标签；已有整数分派可独立证明时 SHALL 保留它和真实来源，不能因标签投影失败而丢失已恢复的 switch。

#### Scenario: Missing or ambiguous dependencies

- **WHEN** 表定义、枚举类或任一已用 case 对应的常量在所选环境中缺失、歧义或无法证明唯一
- **THEN** 系统 MUST 拒绝枚举标签投影，并报告缺少哪一项证据；MUST NOT 依据 `$SwitchMap` 名称或未选定的外部类猜测

#### Scenario: Mutable or irregular table

- **WHEN** 表的初始化包含额外效果、重复/遗漏写入、非唯一键、未覆盖的控制路径，或该表还被当前可见代码改写
- **THEN** 系统 MUST 拒绝将该表当作固定 enum-to-int 映射；未证明的处理器边也 MUST 保留在拒绝范围内

#### Scenario: Budget or cancellation during cross-class proof

- **WHEN** 依赖读取、证明、来源记录或输出在预算耗尽或取消时停止
- **THEN** 系统 SHALL 按现有停止契约响应，MUST NOT 发布一部分已换成枚举标签、另一部分仍依赖未证明表的半成品完整类；essential 与 all 的成功正文 SHALL 相同

#### Scenario: Independent method recovery

- **WHEN** 调用方只请求单方法恢复，而不经过本项完整类的跨类投影阶段
- **THEN** 该方法 SHALL 继续呈现原有已证明的整数表读及整数 case，MUST NOT 声称已经恢复常量标签
