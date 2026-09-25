## Why

完整 Java 8 类源码目前只用方法 descriptor 拼声明，忽略同一物理成员的泛型 `Signature`。已冻结的 `<T extends Number> T choose(T,T,boolean)` 因而被输出为擦除后的 `Number choose(Number,Number,boolean)`；三方源码都能重编，但原类与 JADX 的反射类型参数数量为 1，Jarde 为 0。

## What Changes

- 有界读取方法自己的 `Signature`，复用现有签名语法解析思路，证明类型变量作用域、参数/返回/throws 的完整拼写及擦除 descriptor 一致；再证明已恢复方法体在泛型声明下仍可重编且不会改变已知调用绑定，才在完整类声明中写回泛型方法类型。
- 类级投影保持物理方法身份、原 descriptor、原方法体恢复报告和属性来源；缺失、不一致、未支持、方法体类型关系无法证明或受预算/取消影响时不猜泛型声明，保留当前擦除呈现与可查拒绝原因。
- 以普通调用及反射泛型元数据共同验收原/JADX/Jarde 重编结果；另外覆盖类型变量越界、擦除不符、数组/通配符/内类/varargs 与 class 类型变量边界。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：完整类源码对已证明的方法泛型 `Signature` 保留源级类型参数与类型使用，未知形状明确拒绝投影。

## Impact

涉及方法属性读取、已有通用签名语法、`class_source` 方法声明拼写、类级报告与预算/来源测试。方法体继续以 JVM descriptor/SSA 为事实；此 change 不承担独立的调用实参 reference 转换、类泛型头整体恢复、字段泛型声明或反射注解复制问题。基线见 `../../evidence/java-syntax-2026-09-24/generic-method-signatures/`。
