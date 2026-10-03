## Context

[巡查证据](../../evidence/java-syntax-2026-10-04/nested-class-literal-patrol/README.md)：A12 `nestedLit` 的 BCI 0 `ldc #15 // class A12$Nested` 被引注 "not part of the provable subset"，级联两条（依赖链不有界、saved declaration 未提交）。对照 A10 五形（`A10.class` 全部健康）——差异仅在 CP 名是否含 `$` 嵌套段。

### 与既有 change 的关系（立项查重，2026-10-04）

`recover-class-literals`（**8/8 已完成**，2026-09-26 合入）已恢复引用/数组/本类 `T.class`，其 spec 明确把"Class 池名无法按合法 Java 类型呈现"列为拒绝场景——**本片是该边界的扩展，不重复其机制**（呈现通道 `ExprKind::ClassLiteral` 与 `ConstantValue::Class` 均已存在，本片只动准入）。

### 精确落点（root 已定位，省去实现者取证）

- **门在 `crates/jarde-java/src/decode.rs:347` 的 `source_internal_name`**：`!name.is_empty() && !name.contains('$') && …all(is_java_identifier)`；被同文件 `class_literal_type`（311 行）两处调用（数组元素名 322 行、对象名 337 行），失败返回 `None` → `Operation::Other` → 上游 "not part of the provable subset"。
- **该拒绝在 09-26 是正确的保守决策**（其文档注释）：单类读取无法区分 `Outer$Inner`（嵌套名，需 `Outer.Inner` 拼写）与字面名含 `$` 的顶层类（`$` 是合法 Java 标识符字符）。**但 `recover-nested-type-source-spelling`（2026-10-02 合入，晚于 class-literals）建立的呈现缝已消解该歧义**：折叠域内拼简单名、分离域保留池形——池形对两种情况都是合法源码（分离输出的类声明本身就是 `class A12$Nested`，引用同形即解析）。故歧义不再是拒绝理由。
- **拼写零新增**：`emit.rs:1087` 的 `ExprKind::ClassLiteral { ty }` 已走 `emitter.put_type` —— 正是 nested-spelling 呈现缝的唯一写入口。放宽准入后文本自动按既有规则拼（折叠域 `Nested.class`、分离域 `A12$Nested.class`）。
- **`spell_reference`（build.rs:26715）保持池形**（`internal.replace('/', ".")`，不动 `$`）——与其它携带 `$` 的 ty 路径（如 `new M4$1Doubler()`）一致，故 decode 放宽后 ty 形不破坏身份缝（`spell_reference`/`type_of_base`/`class_name` 等身份缝产出仍为池形，证明比较不受影响）。

**因此本片是"准入放宽 + 既有缝拼写"**：无需 InnerClasses 行证据（分离域池形即可编）、无需把判据搬进有快照访问的层、无新证明机制。

## Goals / Non-Goals

**Goals:** 嵌套类名（`Outer$Nested`、多段 `A$B$C`）的类字面量准入；A12/A11/N2 恢复且行为一致。**Non-Goals:** 呈现拼写规则变更（nested-spelling 既有口径不动）；`spell_reference`/身份缝改动；MethodType/MethodHandle 常量（decode 既有拒绝不动）；泛型参数化字面量（Java 8 无此形态）。数组形（`Nested[].class`，同门 322 行）**一并准入**——同一判据、同一缝，分别处理会留下不一致。

## Decisions

1. **准入判据放宽为"每段是合法 Java 标识符"**：`source_internal_name` 改为按 `$` 分段后每段均为合法标识符即准入（`$` 不再一票否决），同时保持排除空段与非法字符。**本地类（`Outer$1Local`）与匿名类（`Outer$1`）的尾段以数字开头、非合法标识符起始，被同一判据自然排除**——无需专门规则，但实现须以负例钉死该推论。
2. **呈现零新增**：值走既有 `ExprKind::ClassLiteral` → `put_type` 缝（Context 已证）。折叠/分离两口径都按 nested-spelling 既有规则，本片不加规则。
3. **验收锚定**：主锚 A12 两形（`nestedLit`/`nestedRecv`——三指令最小形）与 N2 三形（多段 `N2$Outer$Mid$Leaf`、中层、链式 receiver）；A11 三形（反射读注解全链）；A9.main 为**复合级联形**（根引注 BCI 0/2 即两个嵌套类字面量）——预期恢复，但若准入后残留独立拒绝，如实登记为遗留、不强凑清零；A10 五形逐字不变；负例（本地类/匿名类字面量、字面名含 `$` 的顶层类按池形呈现且可编）。

## Risks / Trade-offs

- **字面名含 `$` 的顶层类**（合法 Java，如 `Foo$Bar.class`）→ 池形呈现即正确（分离域声明同形）；折叠域若该名恰与折叠成员简单名冲突，按 nested-spelling 既有作用域规则处理（仅折叠投影声明的成员才重写），负例钉死。
- **本地/匿名类字面量误纳** → 尾段数字开头即非合法标识符起始，判据自然排除；负例钉死（`Outer$1.class`、`Outer$1Local.class`）。
- **放宽后级联拒绝残留**（"dependency chain not bounded"）→ A9.main/N2 链式 receiver 形若准入后仍有链界拒绝，如实报告并单列，不在本片放宽链界。
- **corpus 面变化** → 双腿扫描，差异应仅类字面量家族；`recover-class-literals` 既有测试与全部呈现测试零回退。
