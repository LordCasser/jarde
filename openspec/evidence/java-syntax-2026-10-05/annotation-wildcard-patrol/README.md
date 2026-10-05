# 注解与通配符消费巡查（2026-10-05 root，负结果）

## 探针

[fixture/AT.java](fixture/AT.java)（`--release 8`）：注解**类型声明**（`@interface Mark` 含 `@Retention(RUNTIME)` 元注解与 `default` 值成员）、`@Deprecated` 内建、字段注解 + 显式值（`@Mark(v = "x")`）、`Class<?>` 返回 + 类字面量、通配符参数消费（`List<? extends Number>`、`List<? super Integer>`）。

## 结果：**健康，无缺口**（宿主 + 注解伴生 quotes=0）

- **注解类型声明**：`@interface AT$Mark` 带 `@Retention(value = RetentionPolicy.RUNTIME)` 元注解与 `public abstract String v() default "d";` 成员（含 **default 值还原**）——完整；
- **使用点**：`@AT$Mark(v = "x")`（字段，显式值对）、`@java.lang.Deprecated`（方法）——全恢复；
- **`Class<?>`/类字面量**：`AT.class` 恢复（Signature 投影拒绝注释属既有 Signature 域，raw `Class` 呈现）；
- **通配符参数**：`List<? extends Number>`/`List<? super Integer>` 呈现为 raw `List`（既有签名投影域的退化形——擦除后一致、行为等价）；
- 拼接两类型 `javac` exit 0、`java -Xverify:all` 输出 `0ATString25` 与原 class **逐行一致**。

## 处置

负结果归档，不立 spec。注解声明/元注解/default 值/使用点、Class 字面量、通配符参数位确认覆盖（通配符的**呈现保真**归既有 ordinary-parameterized-signatures 扩验域，非缺口）。
