## Context

动机见proposal。产品基底是 `597010a8cb9992eead8276b2f2a67186ada393d9`；两前置change的最终自身CI/clean-main交付尚待完成，本项先固定规划与基线，不修改编译来源。

实际基线是两份完整捕获：debug receipt `f9b25f4f93c98c795618590ec75041dd827bd058f344e1c1e8a16dfc92b271e3`，none receipt `b94ca14e7940a4eb4a964172337393e496ed061195e26af2a8eceb79c53d258b`。每份27命令/54raw，双JDK原类编译运行、4条Jarde完整类编译运行均成功，44行输出逐字相同；4条JADX完整类编译失败，无runtime。root实际重放独立审计v2，接受文件SHA `6cf0567b6d31cf3b63d95c0d8ad7b03c7d85a4db29fec5bb72a60db87d211048`，状态audited/errors空；17物理方法按class Code独立解析，report与实际render stdout一致。基线归档进入本change results，不能把这些观察称为新能力完成。

root已读当前 `report.rs::generic_return_candidate`、`generic_void_unread_parameter_slots`、`recover_inner`，`region.rs::named_catch_rows_for_shape`/owner收集与 `Region::Try`，以及 `src/facade.rs::prove_generic_list_parameter_body` 和 `src/class_source.rs::ordinary_parameterized_declaration`。void候选入口和census各自拒绝handler，effect扫描也要求handlers为空；这三个门共同挡住本项。已经保留的完整Program、物理库存和同次Recovered仍在候选产生处可用。现有局部声明/赋值/foreach及调用AST helper已遍历Try body和catch，不应再建第二AST或文本分析。

## Goals / Non-Goals

**Goals:** 给已恢复普通Try前缀+foreach的完整void方法补齐精确泛型List源级声明与调用闭环。结构证明、物理用途证明、签名擦除、可赋值性和真实行为分别验证；至少String/Integer，当前可证明nested body也必须覆盖，不能只修handled一例。

**Non-Goals:** 不修改循环/异常识别算法，不扩大现有Region拓扑，不把任意成功Try当泛型证书。保留no-handler候选及全部旧控制；不改变SSA/Phi类型，不恢复wildcard capture，不加公共carrier、solver/pass、依赖或隐式JDK/classpath访问。纯goto缺少文本映射独立记录，不借本项扩大修改。

## Decisions

1. **先绑定结构，再打开census。** 候选函数从 `recover_inner` 借用同次Recovered/canonical/code；复用现有ordinary named row、prefix+Loop形状和有界owner收集，只派生当前调用内的私有准入事实。必须一条真实typed row，ordinal/type/handler/start/end与现有Catches及实际Try clause匹配；body前缀与loop（含已支持if/nestedloop）保持原owner，catch和纯range-end continuation不得被body吞掉。完整方法没有缺失/重复/额外owner或fallback，才可调用带此事实的物理扫描。不能只删除 `exception_handler_count != 0`，也不能依靠Program非ragged或AST中的Try兜底。证书不跨方法缓存，不写入新的公共请求字段；函数返回的私有有界事实只在本次候选内使用。

2. **一个完整方法扫描，两个准入来源。** 保留当前no-handler路径，ordinary Try仅作为另一种已证明scope进入同一 `generic_void_unread_parameter_slots` 核验。所有Code/operands/SSA/effects/Operations按真实BCI唯一配对，操作库存长度只是第一道门，仍核opcode、wide/短local、block与全部读写。对于新增scope，逐effect核真实handler row归属和protected范围，而非任意接受非空handlers；handler-owned指令也走相同参数读取/写入扫描，任何参数写都拒绝。参数read→单stack输出→唯一invoke receiver仍由实际SSA定义/uses证明，不按BCI相邻或变量名推断。新遍历、匹配、集合与clone均先计费并poll，失败返回None，Stop原样传播；不能生成半个候选。

3. **泛型只沿现有List用途契约进入源级投影。** `prove_generic_list_parameter_body` 继续逐读连接physical parameter slot、formals、foreach site、exact receiver call及source declaration/assignment/next-checkcast链。元素E由当前解析Signature和实际局部类型一致性闭合，现有String/Integer使用同一路径。有限调用仍是 `iterator()`、`add(E)`、`remove(Object)`、`size()`、`clear()`，按完整owner/name/descriptor/opcode绑定；unknowncall、rawalias、wildcard、parameterreassignment和indexOverload仍拒绝。read列表不能伪装成unread。若已有helper支持Try，则不改它；只有实际边界阻碍此交集时才做最小补齐。

4. **复用现有overlay事务。** 只为已证明add实际参数收集合成Object wrapper移除项；真实checkcast、remove(Object) wrapper及overload门不变。继续通过 `ordinary_parameterized_declaration` 的擦除/源级证明和 `GenericCallProjectionState` 完整暂存、重发射与原report一致性门一次安装声明/正文/来源。无第二次recover、文本替换或先发布声明再补正文。所有Stop必须跨层保持BudgetExceeded/Cancelled原语义，拒绝保留raw正文及精确marker；default/all选择不能影响准入。

5. **参考JADX算法，保留本仓证明边界。** 已固定本地7个相关测试及35项source pins的静态分析见 `openspec/evidence/java-syntax-2026-10-13/try-list-signature-next-mvp-analysis-root-v1/`。借鉴SignatureProcessor的擦除对齐、LoopRegionVisitor的iterator唯一uses与元素cast、ProcessTryCatchRegions的loop body递归，以及调用泛型替换/重载检查顺序；不移植其全classpath可变类型推断。现有reader、runtime、AST和标准库已足够，无需新增库或许可维护。JADX raw alias省略造成完整类编译失败的实际结果说明它也可能错误，原class仍是oracle。静态旧报告中String-only的缩限已被root README纠正，本设计统一E。

## Risks / Trade-offs

- 相同handler地址但range不同，或一个clause实际来自多个typed row → 按物理ordinal/type/range精确绑定，新增错row/跨range/重复或缺失owner，以及同handler/range multi-catch变异，不能只测空证书拒绝。
- Try body完整但catch用途漏扫 → 同一整个Code扫描纳入handler-owned指令，构造catch读、写、unknowncall和rawalias控制。真实已支持直接用途按同一List契约判断。
- wrapper移除误吞cast或重载 → permanent tests核add前后origin、Iterator.next真实cast、remove(Object)与indexOverload/rawalias旧控制；完整类含全部旧方法，不裁剪失败方法。
- 新proof晚Stop却已填candidate/overlay → 测实际进入新路径后的低预算与取消，检查Stopped及无部分投影，不把pre-cancel单例当late-stop验证。
- source map只证明现有anchors，未覆盖所有指令 → 基线缺口均为纯goto：handled@44，String/IntegerBranch@40/60/79，removePrefix@55，nested@90，unknownCall@40。现有anchors经独立decoder核为真实指令，且物理owner正确；不宣称全覆盖。此次确保语义操作来源不退化，纯transfer补图另拆。

## Migration Plan

先完成前置产品自身CI与真实clean-main交付，再按本change apply派发Luna私有候选；root审读、串行应用并独占Git/Cargo/JDK/JADX。永久正反/Stop门与完整类都按实际最终代码重新跑，双JDK分别用自身编译的class作JADX输入，保留default/rename-none真实源码及失败raw。完整Runner保留原44行（含旧17行oracle和27行新场景），再加可变output的handled正常完成行，候选至少45行；String/Integer的iterator/hasNext/next/output.add分别注入RuntimeException/Error。基线handled原行仅通过不可变output触发catch，不能代替正常循环完成。另以合格try方法改成员及formal/local名字验证名称无关，不借无try的renamed旧例充数。

每次Rust运行守住可用5GiB、target1GiB上限，1秒进程组检查、默认stack、finally exact cargo clean；共享编译来源由root管理，准备工具不得冒充执行。最终fmt/当前CI Clippy/global strict/必要focused/fresh双seedworkspace、自身产品CI、提交推送与实际HEAD clean-main逐项接受后更新handoff/tasks/账本。产品验收只支持本项范围，71/612或整CF11必须另核账本；当前持续授权已包含spec后派发实现，按apply推进。
