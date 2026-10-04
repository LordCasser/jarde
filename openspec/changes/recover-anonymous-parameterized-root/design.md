# Design — `recover-anonymous-parameterized-root`

见 [proposal.md](proposal.md)。本片是 5.3 链的**环 3**，机制判定见 [anonymous-chain-rings-2-3](../../evidence/java-syntax-2026-10-04/anonymous-chain-rings-2-3/README.md)（结论：**不需要新机制**，移植同文件既有先例）。

## Context（root 实测事实）

锚 `anonymous-super-dispatch` 的形态（javap 逐项实测，见 proposal 的字节码转录）：

- 根方法 `private static Base create(java.lang.String)`，descriptor `(Ljava/lang/String;)LBase;`，flags `ACC_PRIVATE|ACC_STATIC`。
- 分配点在 **BCI 0**，即**直返形**（`0: new / 3: dup / 4: aload_0 / 5: invokespecial $1.<init>:(Ljava/lang/String;)V / 8: areturn`）；实参 `aload_0` 就是根方法的参数槽 0。
- child `AnonymousSuperDispatch$1`：`final java.lang.String val$captured;`，ctor `(Ljava/lang/String;)V` 为 `putfield val$captured` → `invokespecial Base."<init>":()V`（**super 无实参**）。
- `Base` 是顶层类（binary 名不含 `$`），故不受 `anonymous_super_source_type_unproved` 阻挡。
- 当前拒绝码：`anonymous_super_return_type_unproved`，文本 "the root method return descriptor is not the exact superclass type"（root 实测渲染，非推断）。

## 可复用的既有先例（本片的实现依据）

**接口匿名路径已实现"根方法带一个捕获参数"**，`src/facade.rs` 约 3490–3530：

```text
3490  let captured_root_parameter = if capture_descriptor == Some(b"D") { … }
3491      expected_descriptor = [b"(D)L", interface_name, b";"]
3522      let parameter_name = class_source_single_parameter_name(root_ast, 0);
3523      if root_method.descriptor.0 != expected_descriptor
3524          || method_record.is_none_or(|m| m.item.access_flags & 0x0008 == 0)   // ACC_STATIC
3525          || matching_scan.is_none_or(|scan| !scan.complete)
3526          || !site.verified
3527          || site.argument_bcis.len() != 1
3528          || site.argument_parameter_slots.as_slice() != [Some(0)]
3529          || allocation_argument_bcis != Some(site.argument_bcis.as_slice())
3530          || parameter_name.is_none()
3545      let return_matches = if captured_root_parameter.is_some() { …(D)L…; } else { …()L…; };
```

**父类路径没有任何等价分支**：`facade.rs:4698-4699` 只有 `expected_return = "()Lparent;"` 一条恰等检查。

## Decisions

1. **只放宽参数表，返回部分仍须恰为 `Lparent;`。** 判据形如：根方法描述符 == `(P)Lparent;`，其中 `P` 是被证明的捕获参数描述符（单个；多参数形本片拒绝并登记）。**不得**放宽返回类型——那是环 2（`recover-anonymous-super-return-widening`，未立项）的范围，且环 2 需要一条新的类级可赋值性证明能力。二者混淆会让验收无法定位失败原因。
2. **捕获实参来源扩为"根方法参数"**，判据沿用先例的 3524–3530 全套（ACC_STATIC、`scan.complete`、`site.verified`、恰一个分配实参、该实参就是参数槽 `Some(0)`、BCI 与 AST 对齐、参数名可从 AST 取得）。**参数名必须来自 AST 而非 `LocalVariableTable`**：锚以 `-g` 冻结（有 LVT），故必须另冻 `-g:none` 同形对照，证明实现在无 LVT 时同样工作——否则实现会无意中依赖调试信息，而真实 class 常无 LVT。
3. **不硬编码 `b"D"`。** 捕获描述符按 child 字段实际描述符取（本片锚是 `Ljava/lang/String;`），呈现类型走 `ProvedCapturedParameterRead.parameter_presented`（`recover-anonymous-mixed-super-capture` 已引入，替换了原 `Some(Type::Double)` 硬编码）。接口路径残留的五处 `b"D"` 特化（约 3455/3490/3771/3800/3896）**不在本片范围**——它们有 `recover-proved-anonymous-local-capture`(6/6) 的已验收测试守着，泛化须另立一片。
4. **不改站点扫描，故不继承环 1 的遏制义务。** root 读码确认委派顺序：`project_class_source_anonymous_interface`（`facade.rs:3279`）在 **3367** 就把"child 有非 Object 父类且无接口"的情形**委派**给 `project_class_source_anonymous_super`，这**早于**接口自己的返回门（3544）。本片只改父类路径的返回门，且锚已是直返形（不动 `class_source_direct_return_new` 的语句形判据），故**不会**改变 `_anonymous_return_sites` 的产生条件，接口路径的激活面不变。环 1 之所以必须显式遏制（其 design 判据 5、证据 [interface-path-activation-probe](../../evidence/java-syntax-2026-10-04/interface-path-activation-probe/README.md)），是因为它放宽**站点形**；本片不放宽站点形，故无需站点判别位。**但若取证发现实现必须动站点形，停手报告**，那说明本片范围判断有误。
5. **划分退化形须显式覆盖**：本片锚的 super 实参集为**空**、全部构造器参数都是捕获角色。`partition_anonymous_val_constructor` 须在该退化形上仍正确（不得因"super 实参数为 0"而误判为无角色或拒绝）。这是本片与环 1 锚（两 super 实参 + 一捕获）的关键差异，验收须两形都在。

## Goals / Non-Goals

**Goals**：`anonymous-super-dispatch` 形（根方法带一个捕获参数、直返分配、super 无实参）内联为 `new Base() { … }`，捕获读取重拼为根方法参数名；完整源集可编译且行为与原 class 一致。

**Non-Goals**：根方法带**多个**参数；根方法返回父类的**超类型**（环 2）；分配点在局部声明初始化位（环 1）；接口路径 `b"D"` 泛化；`this$0` 三者并存；嵌套匿名；多分配点。

## Risks / Trade-offs

- **风险：放宽根方法门会扩大投影候选面。** 缓解：判据 2 沿用先例的七项合取（ACC_STATIC、scan.complete、site.verified、单实参、槽 0、BCI 对齐、参数名可得），且"已证分配点唯一"不变量保持；corpus 双腿扫描差异应仅本形。
- **风险：与环 1 改同一道门 → rebase 冲突与验收混淆。** 缓解：**强制顺序**——环 1 合入主线后才派发本片；本片 tasks 不得先行修改该门。
- **风险：`-g` 锚掩盖对 LVT 的隐性依赖。** 缓解：判据 2 要求 `-g:none` 同形对照腿。

## Migration Plan

无数据迁移。落地顺序：先冻 `-g:none` 对照 → 放宽参数表判据（保持返回部分恰等）→ 捕获实参来源扩为根方法参数 → 词法替换目标改为参数名。每步失败原因可定位。

## Open Questions

- 根方法为**实例方法**（非 `ACC_STATIC`）且捕获参数来自其形参的形，先例是拒绝的（3524 要求 ACC_STATIC）。本片是否同样拒绝？**默认拒绝并登记**，除非取证发现该形在真实语料中高频且判据可安全放宽——届时停手报告由 root 裁决，不得自行放宽。
- 捕获参数在根方法内**除分配实参外还被别处消费**（例如同时打印）的形：先例的判据只核对分配实参侧。本片须确认该形是否仍可证（词法替换会不会漏掉另一处消费），若不可证则拒绝并登记为负例。
