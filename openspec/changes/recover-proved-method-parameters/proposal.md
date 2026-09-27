## Why

EM-03 的冻结 Java 8 类以 `-g:none -parameters` 编译后，原 class 与固定 JADX 的完整源码重编均保留 `paramStr`、`final number`，Jarde 虽能重编却写成 `arg1/arg2` 且丢失 `final`，运行时参数反射因此改变。成员级 `MethodParameters` 已在 class 中，需在不让方法声明与正文命名分叉的前提下使用这份物理证据。

## What Changes

- 对完整、计数与 descriptor 匹配且名字可作 Java 标识符的 `MethodParameters` 读取参数名及 `ACC_FINAL`，在同一方法的声明和正文中一致呈现；保留原有 `-g:none` 且无该属性时的确定性槽位名。
- 属性缺损、重复、无效位置/flag、与本次方法事实冲突时保守拒绝该元数据投影，不从参数索引、源码猜测或单独 header 文本拼出名字；预算/取消保留现有停止语义。
- 以原/JADX/Jarde 完整 Java 8 源码重编、`-Xverify:all` 与参数反射验收；`throws` 的既有正确行为作为相邻回归。
- 本变更不处理 Smali 畸形 `Exceptions`/泛型签名、隐式/合成参数重排、LVT 与 `MethodParameters` 冲突的通用协调、参数注解重索引或跨方法推断。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：以准确成员级 `MethodParameters` 在无 debug 情形中恢复与正文一致的源码参数名和 `final`，并对不完整证据保守回退。

## Impact

涉及 `jarde-reader` 成员属性类型化解析、核心 class-source 方法事实到恢复层命名的交接及现有参数发射；不新增后端、通用重命名 pass、AST 种类或外部依赖。物理属性、来源和预算仍由现有 reader/恢复请求承担。冻结证据见 [EM-03 对照](../../evidence/java-syntax-2026-09-27/em03-method-signatures/report.md)。
