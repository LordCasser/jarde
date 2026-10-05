# package-info 注解载体巡查（2026-10-05 root）——呈现质量缺口（SAFE）：声明层标识符合法性家族

## 发现（两层）

1. **投影拒绝**：`package-info projection refused: only one complete RuntimeVisibleAnnotations attribute is supported`——但本 fixture **只有一个**注解（`@Deprecated`）；拒绝后注解行仍呈现（`@java.lang.Deprecated`），包声明 `package com.example;` 呈现正确；
2. **幸存声明损坏**：包声明类渲染为 `interface package-info { }`——`package-info` 含连字符**不是合法 Java 标识符**，剥离注释后语法错误 → **不可编译 = SAFE**（第一不变量不违反）；jadx 完美解出（正确文件名 + `@Deprecated package com.example;`）。

## 归因

**声明行名字拼接路径缺合法性检查**——与 #126 unicode 片同族：字段/方法声明走 `is_java_identifier`（CJK 被 `__` 替换），但**类/包声明行的名字原样拼接**（hyphen 直通、成为语法错误）。recover-unicode-identifiers 片实现时应核对类名/包名路径是否同源修复（本条不另立项：SAFE + 极低频——package-info 在真实库中存在但稀少，且拒绝注记已在）。

## 附带判别

Java 8 中 `@SuppressWarnings`/`@Target` 均**无 PACKAGE target**（javac 报"批注接口不适用"）——包级注解封集极小（`@Deprecated` 及自定义 PACKAGE-target 注解）；探针两次修正后以合法最小形定格。
