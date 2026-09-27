## Why

固定 JADX 的 EM-02 覆写测试中，同包子类真正覆写父类包私有方法时，JADX 输出 `@Override`，Jarde 不输出；私有和跨包包私有同名方法则都正确地没有该注解。完整 Java 8 三方重编运行虽一致，Jarde 仍缺少一个可证明的源码级覆写提示。

## What Changes

- 仅在本次类源码请求选定同一 jar 的准确直接父类，且同名同描述符的父方法可见、可覆写时，为子类实例方法投影 `@Override`。
- 原方法事实和本类声明保持不变；缺失/歧义父类、private/static/final、跨包包私有、桥/合成、泛型/协变及预算停止均不猜测注解。
- 用固定 EM-02 三方完整 Java 8 对照与负例验收同包正例及两个同名非覆写反例。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`: 为获证的直接父类实例方法覆写增加可编译的源码级 `@Override` 投影。

## Impact

影响 `jarde` 类源码的跨类定义读取、成员证明和最终文本组装，以及定向测试；不新增 JVM IR、crate、CLI 参数或依赖。[固定证据](../../evidence/java-syntax-2026-09-27/em02-modifiers/report.md)明确将 Smali 非法访问标志与更广义继承关系排除在首片之外。
