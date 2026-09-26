## ADDED Requirements

### Requirement: 准确证明的 Java 8 包注解源文件

系统 SHALL 将完整证明为 Java 8 标准 `package-info` 物理类的源级结果写成包注解与包声明，且 SHALL 保留原物理类的身份、事实和来源。系统 MUST NOT 为该源文件写出 `interface package-info` 或其他虚构类型声明。

#### Scenario: 运行时可见包注解
- **WHEN** `p/package-info.class` 的 Java 8.0 类名、合法非空包名、标准标志、无接口/字段/方法、唯一 `RuntimeVisibleAnnotations` 中的空 `java.lang.Deprecated` 注解及其来源均获准确证明
- **THEN** 完整源码中的 `p/package-info.java` 在 `package p;` 之前写出原包注解，不含类型声明，并与同组源码一起通过 Java 8 重编；运行时读取包注解所得值与原 class 一致

#### Scenario: 证据不完整或非标准形状
- **WHEN** 类名、包名、类头、成员表、包注解、来源或执行预算中的任一必需证据不完整，或物理类有不支持的成员、接口、标志或类版本
- **THEN** 系统 SHALL 拒绝包声明投影，不得以简单名或空成员猜测，并 SHALL 在物理结果中保留可见事实及拒绝原因

首切片仅支持唯一空 `java.lang.Deprecated` 运行时可见注解；`RuntimeInvisibleAnnotations`、其他注解、多个注解和包括 `SourceFile` 在内的其他类属性均 SHALL 拒绝。该边界避免把未知 `@Target` 或 retention 信息猜作合法包注解，也避免把不可见注解改成运行时可见。

#### Scenario: 整体输出与来源
- **WHEN** 同一容器中同时包含包信息类和使用包注解的普通类
- **THEN** 系统 SHALL 对包信息类原子发布完整源文件，包注解与包声明的来源 SHALL 可追溯至原类及其注解，且同组普通类不因包信息投影改变语义
