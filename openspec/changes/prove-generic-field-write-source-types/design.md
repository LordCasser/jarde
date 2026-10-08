## Context

见 proposal 及 generic-holder-patrol。scan_member_uses 目前只给getfield/getstatic分类；putfield/putstatic的read_expressible=None不阻止字段投影。project_method_signature 的ctor分流另有覆盖缺口，不在本片修复。字段和方法候选已在同一class_source运行内延期提交；现有提交顺序先fields后deferred methods，使用“实际发布参数类型”的证明须处理这一依赖。

## Goals / Non-Goals

**Goals:** 修复字段发布健全性，复用SSA、Signature解析和同类使用清单；TypedSetter及已支持参数化字段正例保留。

**Non-Goals:** 不能为Hold(T)新增ctor形、不能补源码implicit unchecked cast、不能新增泛型推理引擎或外部class读取。不能以编译通过宣称保守回退的Signature反射也一致。

## Decisions

1. **写位与读位分开证明。** 在既有SameClassFieldUse事实中补最小写入值来源（从同轮SSA提取），不以read_expressible=None代表成功。优先直接未改写参数与null；无法证明的写位保持未证明，不凭名字猜变量。物理字段identity/完整清单门不变。
2. **以发布事实确定源类型。** 参数槽绑定其物理方法identity和已成功发布的Signature，或保留的descriptor类型。类型变量比较须带class/method scope身份；T/U或shadow不能只比较擦除或文字名。Signature解析复用reader现有结构，不反向解析rendered declaration字符串。
3. **消除发布顺序依赖。** 在既有同类commit阶段先结清相关method候选，再证明/发布field候选，或利用同一阶段的已结清投影结果；不得把Pending或Refused Signature当作Projected。若现有候选间存在无法闭合的依赖，保守拒绝相关字段，而非增加推测性fixpoint系统。实现前用门控确认Deferred方法正面形和预算传播。
4. **赋值关系保持有界。** 同一scope的同型参数/null是明确正例；参数化字段的合法raw赋值需按已有源码类型事实验证，不把raw擦除与T变量混为一谈。方法已发布Signature中明确raw的Class参数沿相同判据处理。root补充确认既有阳性 `RawBoundWriter` 的 `<R extends raw HashMap> put(R,boolean)`：参数仍是方法变量R，但已发布的唯一显式raw Class上界允许对Map<String,String>作unchecked赋值。保留reader现成TypeParameter原始bounds，只对无interface bounds的显式raw Class上界复用现有有界平台关系；目标须是参数化Class，不能以此放行变量T。参数化bounds、class-scope bound推断和泛型替换均不在该窄关系内。更复杂关系不能由物理descriptor推出，保守拒绝。已有字段族须先冻结，再用实测决定最小受支持关系，不新增JDK owner表。
5. **单一证据通道。** reader解析/擦除、SSA验证值、facade扫描与source候选提交各守原层；拒绝必须带field/method/BCI。无需新库、crate、IR或pass；JADX只能提供语法对照，不能替代写值类型证明，未复制外部代码。

## Risks / Trade-offs

- 字段变成Object后某些依赖声明仍不合法 → 对冻结类完整重编译检查；需要联动拒绝时仅沿该字段既有依赖传播，不扩为新的语法恢复机制。
- 不加区分地拒绝writes会消掉已有泛型收益 → TypedSetter、null、参数化/raw字段与同类延期方法都是强制正例。
- 整段AST重复留存会增加内存/预算 → 扫描期间仅保留必要的SSA来源事实，保持原预算/取消传播；不复制整套Program。
- 泛型ctor恢复与字段soundness相互遮蔽 → Hold在本片只能安全回退，不能勾选ctor覆盖；后续片独立恢复并验收reflection。

## Migration Plan

先冻结/门控，最小实现与双腿整类验证，再root独立验收。无API兼容迁移与新依赖；撤销本片即可回滚。
