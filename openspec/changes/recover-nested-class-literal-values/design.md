## Context

[巡查证据](../../evidence/java-syntax-2026-10-04/nested-class-literal-patrol/README.md)：A12 `nestedLit` 的 BCI 0 `ldc #15 // class A12$Nested` 被引注 "not part of the provable subset"，级联两条（依赖链不有界、saved declaration 未提交）。对照 A10 五形（`A10.class` 全部健康）——差异仅在 CP 名是否含 `$` 嵌套段。

### 与既有 change 的关系（立项查重，2026-10-04）

`recover-class-literals`（**8/8 已完成**，2026-09-26 合入）已恢复引用/数组/本类 `T.class`，其 spec 明确把"Class 池名无法按合法 Java 类型呈现"列为拒绝场景——**本片是该边界的扩展，不重复其机制**（呈现通道 `ExprKind::ClassLiteral` 与 `ConstantValue::Class` 均已存在，本片只动准入）。

### 精确落点（root 已定位，省去实现者取证）

- **门在 `crates/jarde-java/src/decode.rs:347` 的 `source_internal_name`**：`!name.is_empty() && !name.contains('$') && …all(is_java_identifier)`；被同文件 `class_literal_type`（311 行）两处调用（数组元素名 322 行、对象名 337 行），失败返回 `None` → `Operation::Other` → 上游 "not part of the provable subset"。
- **该拒绝在 09-26 是正确的保守决策**（其文档注释）：单类读取无法区分 `Outer$Inner`（嵌套名，需源码拼写）与字面名含 `$` 的顶层类（`$` 是合法 Java 标识符字符）。**但 `recover-nested-type-source-spelling`（2026-10-02 合入，晚于 class-literals）建立的呈现缝已消解该歧义**：拼写不再靠 `$` 猜测，而靠 `InnerClasses` 行集确证（见下条）——在行集内的名拼为源码形，不在行集内的名（含字面 `$` 顶层类）保持池形，两种结果都是合法源码。故歧义不再是拒绝理由。
- **拼写零新增**（root 2026-10-04 三次核实后的最终版，修正先前两版错误依据）：`emit.rs:1497` 的 `put_type` 对含 `$` 的名调用 `names.rs::nested_member_reference_spelling`（226 行），声明侧 `src/class_source.rs:5687/5741/6020/7113` 调用同一函数——本片只让类字面量走这条既有路径，不新增证据通道、不加拼写规则。
  - **两级机制（勿简化为"在行集内即拼源码形"）**：(1) `InnerClasses` 行集是**必要条件**（JVMS §4.7.6——`$` 单独不足以断言嵌套关系，`Named$Top` 可能是顶层类自身名），名不在行集内 → 保持池形；(2) 但即使在行集内，`nested_reference_spelling`（names.rs:168）对**自嵌套名**（`head == owner`，即名的顶层 owner 就是正在书写其文本的类）仍**返回池形**，其注释明说只有**折叠投影**才可拼成源码嵌套形（`only a fold projection may spell it as source nesting`，names.rs:202-207）。
  - **故池形的真实成因是折叠深度，而非行集缺失**（javap 实证：`N2.class` 的 InnerClasses 表确实含 `Leaf=class N2$Outer$Mid$Leaf of class N2$Outer$Mid` 行，`FP.class` 同理含 `Leaf=class FP$Mid$Leaf of class FP$Mid`，但两者的深层名都保持池形）。成员折叠是**单层**的：深度 1 的名（`A12$Nested`）被折叠覆盖 → 拼 `Nested` → 编译后携带 InnerClasses → `getSimpleName()` 正确；深度 ≥2 的名（`N2$Outer$Mid$Leaf`）覆盖不到 → 保持池形 → 源码中是顶层类 → `getSimpleName()` 返回池名、`getEnclosingClass()` 返回 null。
  - **先前"声明与引用不可能不一致"的论断已被 root 的 FP probe 推翻**：家族口径下 `FP`（不含任何类字面量、主仓 HEAD 二进制）的方法体呈现 `new FP$Mid$Leaf()`（池形）而声明侧呈现 `static class Mid`（源码形，深度 1 被折叠），javac 报"找不到符号"——即单层折叠 + 深层名池形会产生**声明与引用不一致**。该缺陷是既有平铺/折叠域的，与本片准入判据正交，已登记（见 handoff.md 的"池形类型名的结构反射陷阱"）。
  - **本片的验收后果（root 裁决）**：类字面量准入放宽后，若其呈现仍为池形且被 `Class` 的结构反射方法消费（`getSimpleName`/`getEnclosingClass`/`getCanonicalName`/`getDeclaringClass`/`isMemberClass`/`isLocalClass`/`isAnonymousClass`/`getNestHost`/`getNestMembers`/`getEnclosingConstructor`/`getEnclosingMethod`），会把基线的**响亮失败**变成**可编译且行为不同/NPE**，违反既有不变量——必须保持拒绝。判据取"最终呈现文本是否仍含 `$`"，**不得**取"名是否在行集内"（后者与偏离无因果关系）。
- **`spell_reference`（build.rs:26715）保持池形**（`internal.replace('/', ".")`，不动 `$`）——即 ty 在**证明层**保持池形，重拼发生在**呈现层**（`put_type`）。这与其它携带 `$` 的 ty 路径（如 `new M4$1Doubler()`）一致，故 decode 放宽后不破坏身份缝（`spell_reference`/`type_of_base`/`class_name` 等身份缝产出仍为池形，证明比较不受影响）。

**因此本片是"准入放宽 + 既有缝拼写"**：拼写复用 nested-spelling 已建立的 `InnerClasses` 行机制（**本片不新增证据通道**，但实现者须知呈现结果取决于该行集——行集缺失时两侧同保持池形，仍一致）；无需把判据搬进有快照访问的层、无新证明机制。

## Goals / Non-Goals

**Goals:** 嵌套类名（`Outer$Nested`、多段 `A$B$C`）的类字面量准入；A12/A11/N2 恢复且行为一致。**Non-Goals:** 呈现拼写规则变更（nested-spelling 既有口径不动）；`spell_reference`/身份缝改动；MethodType/MethodHandle 常量（decode 既有拒绝不动）；泛型参数化字面量（Java 8 无此形态）。数组形（`Nested[].class`，同门 322 行）**一并准入**——同一判据、同一缝，分别处理会留下不一致。

## Decisions

1. **准入判据放宽为"每段是合法 Java 标识符"**：`source_internal_name` 改为按 `$` 分段后每段均为合法标识符即准入（`$` 不再一票否决），同时保持排除空段与非法字符。**本地类（`Outer$1Local`）与匿名类（`Outer$1`）的尾段以数字开头、非合法标识符起始，被同一判据自然排除**——无需专门规则，但实现须以负例钉死该推论。
2. **呈现零新增**：值走既有 `ExprKind::ClassLiteral` → `put_type` 缝（Context 已证）。折叠/分离两口径都按 nested-spelling 既有规则，本片不加规则。
3. **验收锚定（root 验收后修正）**：主锚 A12 两形（`nestedLit`/`nestedRecv`——三指令最小形，深度 1 折叠成功）、A11 三形（反射读注解全链）、WC1（字面 `$` 顶层类）、WV1（数组形）与 A9.main（复合级联形，proposal 的"17 处引注"清零）；A10 五形逐字不变；LC 负例（`Outer$1`/`Outer$1Local`/`Outer$`/`Outer$$Inner`）全拒。**守卫的四向必须齐备**（判据是"池形 ∧ 结构反射消费"，非按深度硬编码）：A12-jar（折叠成功 + `getSimpleName`）→ **恢复**；A12-single（standalone + `getSimpleName`）→ **拒绝**；RF-jar（family 折叠失败 + `getSimpleName`）→ **拒绝**；N2-jar `midLevel`（`getName`，非结构反射）→ **恢复且逐字一致**。另 N2 的 `multiLevel`/`recvChain`（深层名 + `getSimpleName`/`getEnclosingClass`）保持拒绝，其池形呈现被结构反射消费会静默偏离/NPE。

## Risks / Trade-offs

- **字面名含 `$` 的顶层类**（合法 Java，如 `Foo$Bar.class`）→ 该名的 `$` 不构成嵌套关系，故它**不会**出现在 `InnerClasses` 行集内，`nested_member_reference_spelling` 因此保持池形 `Foo$Bar`；而声明侧走同一函数同一行集，也保持池形——**这种情形下两侧一致**且都是合法源码（`Foo$Bar` 是合法标识符），WC1 fixture 钉死。注意"两侧一致"只在**同为池形**时成立：折叠域内声明被拼为源码形而深层引用仍池形时两侧会不一致（FP probe 实证的既有缺陷，属折叠深度域，非本片）。
- **本地/匿名类字面量误纳** → 尾段数字开头即非合法标识符起始，判据自然排除；负例钉死（`Outer$1.class`、`Outer$1Local.class`）。
- **池形 + 结构反射 = 静默偏离/NPE（root 验收发现的阻塞点，本片必须堵）**：类字面量准入放宽后，深层名（折叠覆盖不到）保持池形 → 编译产物是顶层类、无 InnerClasses → `getSimpleName()` 返回池名（N2 `multiLevel`：`Leaf`→`N2$Outer$Mid$Leaf`）、`getEnclosingClass()` 返回 null（N2 `recvChain` → NPE 崩溃）。基线该形是**响亮失败**（10 处引注、方法体空、javac 报 missing return），本片首版把它变成**可编译且行为不同**，违反 `recover-return-in-do-while-false` 与 handoff"池形类型名的结构反射陷阱"的不变量。守卫判据取"最终呈现文本仍含 `$` ∧ 值被结构反射方法消费"（两级机制见 Context，**不得**按行集判定或按深度硬编码）。
- **family 折叠失败 + 直属成员 + 结构反射 = 静默偏离（root 验收发现的第二个阻塞点，本片必须堵）**：build 层守卫靠 `pool_spelled_members` 旗标区分直属成员，但旗标只在 standalone 口径为 true——family 口径下 build 恢复方法时折叠**尚未尝试**，旗标恒 false，无法预知折叠成败。故 family 口径下"直属成员（如 `RF$Inner`）折叠被拒（`<clinit>`/Signature/enum-initializer 投影）→ 回退分离池形呈现 → 结构反射消费"这条路径**绕过守卫**，发布 `RF$Inner.class.getSimpleName()`（0 引注）→ 家族重编 exit 0、运行 `RF$Inner` ≠ 原类 `Inner`（[residual-boundary/](../../evidence/java-syntax-2026-10-04/nested-class-literal-patrol/residual-boundary/README.md) 实测）。这是 ncl 引入的净倒退（ncl 前 decode 拒绝 → 整方法引注 → 响亮失败）。**裁决：在 facade 折叠失败回退路径（facade.rs:1957/1970 的 `Ok(Err(_reason)) => {}`）对受影响方法以 `pool_spelled_members=true` 重跑恢复**，使 build 守卫产生真实 Refusal（复用 20519 的 `analyze_method_ir` re-recovery 先例）；拒绝文本用 run 自己的解释，不伪造引注。A12-jar（折叠成功）与 RF-jar（折叠失败）在 build 层旗标相同，区别只在 facade 折叠成败——故守卫必须在折叠决策之后，不能在 build 层用更细旗标区分。
- **放宽后级联拒绝残留**（"dependency chain not bounded"）→ A9.main 等复合级联形若准入后仍有链界拒绝，如实报告并单列，不在本片放宽链界。
- **corpus 面变化** → 双腿扫描，差异应仅类字面量家族；`recover-class-literals` 既有测试与全部呈现测试零回退。
