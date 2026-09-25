## Why

普通 instanceof 测试目前缺少可呈现事实和表达式，Object 参数样例的80行三方审计中 jarde 整类编译失败。扩展合法源码又证明 Object 加宽会在字节码中消失，盲目恢复成 String instanceof Integer 等文本也无法编译，jadx 同样出现该问题。

## What Changes

- 在已有表达式恢复路径中忠实呈现类型测试及 boolean 结果，覆盖普通类、接口、数组、局部、返回和已有条件。
- 保留可成立的 Java 左操作数静态类型上下文，复用现有安全 Object 加宽及类型拼写，不引入继承解析器。
- 复用最终消费点、唯一消费、失败生产者、来源和预算协议，验证调用次数、生产者自身抛错、null 与不相容静态类型。
- 前置为已有引用 cast、boolean 类型判据和区域恢复。0/1 汇合、pattern matching、额外 loop/guard 模式、类型层级解析和常量折叠不属于本项。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`: 恢复真实 instanceof 的 boolean 表达式及合法引用静态上下文，保持操作数效果与拒绝来源。

## Impact

涉及 jarde-java 的 facts/decode、ast/emit、build 中共享 boolean 判据和现有消费边界，以及专项测试。新增一个忠实指令事实和一个类型测试表达式即可，不增加 pass、resolver、缓存或第三方库。生产文件等待 numeric/throw/final 字段的所有权交接，不并行抢写。验证/编译平面的承诺与宿主接口不变。
