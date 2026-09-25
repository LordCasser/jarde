## Why

合法 Java 8 `@interface Nested { Inner child() default @Inner(count = 6); Inner[] children() default {@Inner(count = 2)}; }` 的 class 文件在 reader 中完整保留两个 `AnnotationDefault` 值树。Jarde 目前的 `resolve_default` 遇到 `ElementValueFacts::Annotation` 返回 `None`，使两个默认值在生成声明中静默消失；该行为在 `../../evidence/java-syntax-2026-09-22/annotation-default-boundaries/header-minimal/nested/` 的原 class/JADX 反射 `6/2` 与 Jarde 文本中可直接核对。当前 `@interface` 头部错误另有前置 change `spell-annotation-type-headers`。

## What Changes

- 在已有成员默认值词汇中增加嵌套注解值，沿 reader 已解析的 descriptor、按序成员名和值递归解析并拼写 `@Type(name = value, …)`。
- 数组仍采用现有全有或全无规则：一个子值不能忠实拼写时不输出错误的部分默认值；reader 已解析的事实与原始 class 字节留在读取层，不声称 class-source JSON 已发布该值树。
- 头部前置 change 完成后，用完整 Nested/Inner/runner 的 Java 8 编译和反射结果验收；F/D 注解常量、enum 类型发射及任意注解使用位置另案，不在本切片。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`: 已解析且可忠实拼写的嵌套注解及其数组默认值 SHALL 出现在注解成员的 `default` 声明中，并与原 class 的反射默认值相同。

## Impact

仅 `src/class_source.rs` 的私有默认值解析/拼写和相应测试/fixture；reader 已有递归事实，无新 crate、依赖、公共 API、解析器或一般 annotation-use pass。该 change 必须排在 `spell-annotation-type-headers` 后验收。
