## Context

已完成的 `spell-annotation-type-headers` 使 canonical 注解头合法。`src/class_source.rs::declared_annotation_default` 从成员属性读到 reader 的 `ElementValueFacts`，`resolve_default` 再按 tag 和常量池项构造私有 `MemberDefault`；F/D 分支目前明确返回 `None`，所以报告保留原始属性事实但源码失去默认值。`MemberDefault::Array` 已用 `Option<Vec<_>>` 保证一个子值失败时不输出半个数组。`CpEntryKind` 保存 u32/u64 原始 bits，不需 reader 改动。

## Goals / Non-Goals

**Goals:** 对有忠实 Java 8 注解常量表达式的 F/D 属性值恢复默认值；完整注解类重新编译后反射 raw bits 与原 class 相同，来源事实、预算/取消及数组原子性保持不变。

**Non-Goals:** 非标准 NaN payload/sign 的源码猜测、运行时 `Float.intBitsToFloat` 调用、字段 `ConstantValue`、Code 中的浮点常量、任意常量折叠或全局数值求值器。不能把已编译但默认值缺失的类计为恢复成功。

## Decisions

1. **原始 bits 是唯一输入。** 在现有私有 `MemberDefault` 加 `Float(u32)`/`Double(u64)`，仅当 `ElementConstantTag::Float` 配 `CpEntryKind::Float`、`Double` 配 `Double` 时构造。`resolve_constant_value` 保持原支持范围，不因共享 enum 增加字段初始化。成员类型与 tag 不匹配、池项错误仍拒绝。
2. **有限值用精确十六进制 Java 字面值。** 对符号位、偏置指数、尾数作整数分解，明确写 `f`/`d` 后缀；负零、最大/最小有限值和次正规值都要按 raw bits 重编译回同值。复用已存在的浮点拼写规则或在既有 `jarde-java` emitter 内放置一个纯格式化辅助函数供本项和 `recover-floating-point-constants` 使用；不新增 crate、AST、pass 或数值抽象层。函数只负责字面值，不决定 NaN 恢复资格。
3. **特殊值有明确准入。** `+∞`/`-∞` 使用 `Float`/`Double` 的对应 `POSITIVE_INFINITY`/`NEGATIVE_INFINITY` 常量，标准正 quiet NaN 使用 `Float.NaN`/`Double.NaN`；当前 Java 8 `javac` 已验证它们可用于注解默认值且反射 raw bits 与 fixture 相同。非标准 NaN 的 sign/payload 或 signaling 模式返回 `None`，不经宿主浮点转换归一化，不尝试注解默认值不允许的运行时方法调用。负 NaN/payload 的 JVM 合法受控样本是拒绝边界，不能以 JADX 的显示文本代替 raw-bit 运行结果。
4. **沿用属性树与整段提交。** `MemberDefault::spelling` 只在 `resolve_default` 完整成功后输出，嵌套数组中任一 F/D 不能忠实拼写时整段 `default` 省略；reader 的原始 `AnnotationDefault` facts、class-source 的 member item/现有 JSON 字段、类源码输出预算与取消路径不变。class-source JSON 不新增解析后的默认值树。默认和完整 evidence 请求文本一致。
5. **整类反射验收。** 固定 293B 注解类与 runner，原/JADX/Jarde 各自完整编译并在 `-Xverify:all` 下执行，逐行比较 raw bits。另用 patched class 的负 NaN/payload 运行验证不能误接纳，数组和普通 B/C/I/J/S/Z、String、class、enum、嵌套注解默认值做相邻回归。

## Risks / Trade-offs

- 宿主 `f32`/`f64` 格式化或十进制最短表示丢失负零、次正规或边界舍入 → 只按整数位分解拼精确 hex，并做实际 javac/反射 raw-bit 对照。
- `Float.NaN` 抹掉异常 payload 或符号 → 只接受指定 canonical bits，其余保守拒绝；不把可编译的归一化结果当成功。
- 共享 `MemberDefault` 无意扩了字段 ConstantValue → 单独断言 field 路径不变，构造 F/D 仅在 `resolve_default`。
- 数组一处失败只输出前缀 → 复用现有 `collect::<Option<Vec<_>>>()`，测试整段缺省。
