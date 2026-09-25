## Context

`src/class_source.rs::class_declaration` 从 `ACC_ANNOTATION` 选出 `@interface`，随后仍按 `ACC_INTERFACE` 处理 `ClassDeclarationFacts::interfaces`，把 JVM 规定的 `java/lang/annotation/Annotation` 拼为 Java 源码的显式继承。`@interface` 的这种继承是隐式的，javac 拒绝显式语法。`header-minimal/basic/` 的 Jarde 完整类只有这一个编译错误；`nested/` 的两类同理，另有独立默认值省略问题。`ClassSourceDeclaration.item` 已保留物理接口表，不需要改变 reader 或公开报告结构。

## Goals / Non-Goals

**Goals:** 恢复 canonical Java 8 注解类型的合法声明头，保留原始接口事实，让基础默认值完整类可编译、反射行为相同，并保护普通接口/类继承发射。

**Non-Goals:** 嵌套/F/D 默认值、注解使用位置、enum 常量、异常 class 文件附加接口的 Java 表达、注解的继承解析。后几项不得因这个头部修复被冒称可恢复。

## Decisions

1. **在既有类头拼写处分支。** `ACC_ANNOTATION | ACC_INTERFACE` 且接口表正好是 `java/lang/annotation/Annotation` 的类输出 `@interface Name` 后直接闭合头部；其余仍由现有接口/类分支处理。对非 canonical 接口列表不静默丢掉额外接口，避免把非 Java 注解的 class 文件改述为合法注解。实现不新增 AST、pass、解析层或公共字段。
2. **结构化事实不变。** 只改变 `ClassSourceDeclaration.declaration` 和组装 text，`item.declaration.interfaces` 仍是 class 文件原始列表。已有预算/取消、成员默认值、source-map 请求不增新遍历。
3. **整类为验收单位。** `basic/` 原 class、JADX、修后 Jarde 的完整类与未改 runner 分别用 `javac --release 8 -g:none`、`java -Xverify:all` 运行反射默认值；普通接口继承和普通类实现做相邻回归。`nested/` 仅检查头部已合法且默认值仍未恢复，归下一 change，不能拿它的源码可编译代替语义验收。

## Risks / Trade-offs

- 只检查 annotation flag 就丢掉接口表，会掩盖异常 class 文件的额外接口；限定 canonical 列表，非 canonical 保留现有弱表示和原始事实。
- 修复头部后，原本被 javac 拦住的缺失默认值会变成可编译但反射不等价的结果；独立 nested fixture 明确标红并交给下一 change，不能把本 change 的 Basic 通过说成所有注解已恢复。
