## Why

Java 8 类在字段类型、方法返回类型和形参类型上的 `TYPE_USE` 注解真实存在，但 Jarde 的类源码静默省略；独立编译的 subject 使三处运行时反射从 `field / return / parameter` 全变为 `null`。普通声明注解的前缀位置在双目标注解上会同时生成声明和类型属性，不能直接拿现有成员注解拼写补这个缺口。

## What Changes

- 按需读取字段、方法自身的运行时可见/不可见类型注解属性，保留物理属性壳、目标、路径和完整注解值，区分声明注解与类型注解。
- 首轮只恢复目标分别为 `FIELD`、`METHOD_RETURN`、`METHOD_FORMAL_PARAMETER`，路径为空、且 Java 源能在限定引用类型名内部独立拼写的使用；形参按 descriptor 位置定位。重编译后类型反射须与原 class 相同，且不得凭空产生声明注解。
- 对基本类型、默认包单段名、数组路径、同位置双目标冲突和受损或不可拼写事实明确拒绝；保留原始归属及停止状态，不猜注解类型的外部 `@Target` 元数据。
- 冻结原/JADX/Jarde 完整类及隔离 subject 的对照，并增加双目标源码位置、受控 primitive type-only 等边界。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：类源码在已证明可忠实表示的位置恢复 Java 8 类型使用注解，并公开不支持位置的物理事实和拒绝。

## Impact

前置条件是 `recover-member-annotation-uses` 的共享 reader、成员归属和声明拼写经 root 验收。实现只触及 `jarde-reader` 的类型注解内容读取、核心类源码/JSON 的类型位置拼写及 facade 的同次成员交接；不改方法体 IR、CLI 协议或依赖。类继承/泛型界限、接收者、throws、局部变量、cast、new 等其余 type-use 目标，以及数组层级路径、外部注解类型解析和声明注解独有的双目标漂移，分别留作后续审计。
