## Context

见 proposal.md。main 564e22c1 的 SameClassFieldUse 只带 RHS write_source 和 source_complete。project_field_signature 对每一写入统一使用 parsed.ty；raw receiver 下合法的 Object→Object 选择因此被误作 Object→T。same_class_published_method_parameters 已在方法投影完成后、字段提交之前构造，未发布 Signature 只提供 descriptor 参数。这条阶段顺序可直接沿用。

JLS8 §4.8 明确：raw 类型自己声明的非静态字段选择擦除类型；静态成员保持泛型声明中的类型。§4.6 保留数组维数和最左上界。原六族取证见 openspec/evidence/raw-receiver-source-selection-patrol/summary.md，新冻结矩阵见 openspec/evidence/raw-receiver-field-selection-2026-10-09/。InstanceRawLocal 的反事实证明，不能从 aload/SSA 起源推断最终源 receiver。

## Goals / Non-Goals

**Goals:** 在现有 class-source 字段提交路径支持实际 emitted raw formal 与保留 raw local；保留 T、T[]、bound 字段声明和行为；每个字段使用、receiver 和 RHS 均按物理身份绑定；实际 method header 是参数类型唯一依据。

**Non-Goals:** 通用参数化 receiver 代换、方法 Signature 扩容、this 委派、跨类/继承字段解析、复杂 alias/cast/phi 源级推断、任意调用结果适配、隐藏编译失败的猜 cast、改变反序列化输出 API。上述问题分别登记，不把本片写成整个 generic 单元完成。

## Decisions

### 1. 复用同次 Program，交付最小 receiver 事实

在 jarde-java 已构建 Program、实际发射前的既有 class-source sidecar 路径提取按物理 putfield BCI 唯一匹配的 receiver 事实，不新增 AST 节点或第二次解析。优先交付最小 site 事实，避免为了局部证明保留每个完整 AST。若现有 retained AST 查询能完整覆盖冻结 raw-local 族且预算更简单，也可复用它；不得因为 AST 默认不保留就静默缩到 direct-only。

实际 ExprKind::Local("this") 标为 this，不沿 SSA 别名还原原 source raw 类型。直接参数名须与同次 Names/parameter_positions 一一绑定物理 slot；最终是否 raw 在 method commit 后读取该方法实际发布的参数类型。raw local 须在实际 AST 中有唯一可见声明、Type::Reference 的精确 raw own-class 拼写，且字段 receiver 表达式确为该 local；Expr::presented 的物理类型本身不能替代参数的最终泛型头或 local 声明。重绑定、歧义、闭路径外的作用域/phi 均不进入新证明。

字段 BCI、FieldPlan 和 CP owner/name/descriptor 要一一对应。合成 accessor/inlined origin、同名字段或没有实际发射的 putfield 不能生成同类字段 raw 许可。source_complete 仍表示正文完整，不代替 receiver AST 事实。后续 class-source 重发射若能改 receiver 或声明，须核对事实仍适用，不能沿用旧 AST 许可。

### 2. 保持 publication 有向顺序

既有 SSA 扫描继续负责物理 use 清单和封闭 RHS；把同次 receiver 事实关联到相同 PhysicalMethodId/BCI。已发布参数表同时收集 RHS 和 receiver 参数所需方法，含 null RHS 的 raw 参数写者；绝不读取被拒绝 writer Signature、其他方法头或字段反向证明方法头。static slot0 与 instance this 由实际 access flag/physical slot 区分。

### 3. 显式分开声明类型与访问选择类型

字段声明仍是已通过 erasure/class-scope/name/flags/type-annotation 检查的 parsed.ty。只在 opcode b5、自己声明且物理身份准确、receiver 明确 raw、正文完整时，以已经校验的字段 descriptor 擦除类型证明 RHS 可赋值；其余路径保留旧泛型 RHS 证明。不要让现有 generic-only raw_parameter_assignable 的约束全局变宽。

擦除目标赋值须使用现有 descriptor/reference assignability 的封闭边界：null 合法；参数只读实际发布类型或 descriptor、匹配物理槽位，已发布源变量可按其已验证上界/擦除向擦除目标赋值；数组维数/组件和 bound 保持准确；Object 不能赋给 Number/String。RawAllocation 仅在既有完整初始化 proof 能证实赋值时使用，不扩展调用/别名来源。预算遍历和取消传播使用既有 Budget，不吞掉停止为普通拒绝。

### 4. 参考 JADX 算法，避免移植缺口

本地 JADX 的 TypeBoundFieldGetAssign.getType 使用 instance type 调 TypeUtils.replaceClassGenerics，TestGenericFields 验证泛型字段链；raw TypeUtils.getTypeVariablesMapping 返回空映射，FieldGet 动态 bound 的未替换 fallback 仍返回 initType 并有 scope TODO。FixTypesVisitor 再以 cast 与类型传播尝试修复。借鉴“访问点 receiver 决定成员类型”的原则；本片用显式 raw erasure 和实际 emitted receiver 证明，不移植 unresolved T fallback、全局 fixpoint 或未证明 cast。

没有适合替代本地物理来源与已发布类型事实的外部库：已有签名解析、erasure 与 AST 足够，不新增依赖。仅参考本地代码算法，不复制 Java 实现；保留测试与源路径出处，无新许可内容引入。

## Risks / Trade-offs

- AST 参数类型早于最终 method header → 保留参数身份，延后以已发布参数表分类，绝不仅看 presented raw descriptor。
- this alias 重写使 raw 形消失 → 只认证实际 receiver Expr；InstanceRawLocal 必须维持安全擦除，JADX 同例失败照实保留。
- 仅一个写位合法 → 使用完整字段清单逐写者验证，并保留现有 read consumer/source identity guards。
- 确定 raw 不代表 RHS 合法 → 单独验证擦除目标赋值，不用“descriptor 相同”代替泛型或值来源证明。
- 事实扫描无界或旧 AST 重发射 → 计费/poll，并对实际仍发射的物理 member/site 验证；未知保守，不猜源类型。

## Migration Plan

在 main 原位实现，无新增分支/worktree。root 独占共享 Cargo target。Luna 分别负责确定性生产修改和冻结对照，root 审查边界与独立回归；完整门禁/最新 HEAD CI 验证后提交推送，更新 handoff 并清理 Cargo。回退保持字段 descriptor 声明与原拒绝，不提供兼容新旧事实双轨。
