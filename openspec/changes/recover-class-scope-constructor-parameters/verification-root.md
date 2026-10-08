# 类作用域构造器参数 — root 验收

基线是 `790579e2`，既有字段源码赋值证明已在主线；本片恢复此前仍擦除的构造器参数。root 先核对最新 handoff 和干净主线，再派 Luna 实现，独立审查与重放，没有扩散到另一个 generic 子系统。

## 恢复路径与架构边界

`report::generic_constructor_candidate` 在同次 Program/Code/SSA/InitRecord 上证明完整 Object()、直接 this 字段写和 return。每个原参数 load 必须来自正确物理槽的 Entry 定义；初始化后的 this、putfield owner/name/descriptor、AST origin、唯一消费、operation/effect 与全部指令逐位一致。原参数可重复加载给多个字段；long/double 槽由 descriptor 的现有位置函数计算，不按参数序号猜。重复 BCI、额外指令、改写局部、phi/EH/call/cast 保守拒绝。物理字段必须是同次读取的唯一实例声明，不靠泛型 Signature 确认字段身份。

`class_source::project_method_signature` 在已发布类 scope、reader 作用域/擦除与完整候选均成功后，沿原子发布路径恢复 T、T[]、bound 和多变量参数，不新增构造器形式参数。facade 仍先结清方法，再让字段全写位证明消费实际发布参数；CrossHold 恢复 U，不能把 U 和字段 T 当成一个 binder。旧方法级字段赋值构造器保持拒绝，匿名 superclass 转发和静态成员空构造器消费者显式排除字段写候选。没有新 pass、parser、IR、fixpoint 或源 cast 修补。

## 验收中发现并修复的回退

首次 18 族四腿重放，8 个正例恢复完整泛型反射，却让 ThisDelegateHold 四腿从可编译变成不可编译。被委派目标发布 T，caller 仍发布 Object；既有物理 descriptor/arity 只证明目标身份，不能证明源码参数适配。原始失败保存在 `results/initial-root`，没有删掉边界或改成“允许失败”。

最终在已有 SameClassInvokeUse 中携带该次 SSA 构造接收者事实，最终 deferred commit 按精确目标检查。实际 this 或未知接收者给出 `unproved_this_delegate_call`，只限制新增 class-scope 投影；method-formal 旧路径不变，也不污染无关清单。raw new 保持可恢复：root 与独立 audit 核对普通 New AST/emitter、本片无 `$`/Object/no-interface 门以及两个受限 diamond 来源，确认目标调用实际输出 raw `new C(...)`。不能把这个放行理由说成“SSA 证明了原始源码的泛型类型”。

新增 RawNewHold 又抓到 scanner 把接收者固定为 Stack(0) 的误拒。真实 `new; dup; aload; invokespecial` 保留副本占据深度 0，接收者位于深度 1；frame Invoke effect 先逆序读取参数、再读取 receiver。现取 reads.last 的实际 Stack 槽和 Value 分类，this 仍拒绝，new-site 放行。两组真实独立分配和 ThisDelegate 四腿测试闭环。

## 三方完整类对照

真实 Corretto 8u432 与 OpenJDK 23.0.1，各含 `-g`/`-g:none`，输入 major52。20 族共 80 输入：9 个全恢复正例、PeerNewHold 部分恢复、10 个独立边界。root 单独编译外部 Driver，生成文本重编后仅把生成类与 Driver 放在 classpath，以 `-Xverify:all` 执行；原 jar 不掩盖漏类。每个 CLI 输出要求非空完整类及自述头，CLI 0/4 不作为成功判据。

| 产物 | 完整编译/行为一致 | 全声明泛型反射一致 | 所有构造器泛型反射一致 |
| --- | --- | --- | --- |
| 基线 Jarde | 72/80 | 0/80 | 8/80 |
| 候选 Jarde | 72/80 | 36/80 | 48/80 |
| JADX 1.5.6 | 64/80 | 64/80 | 64/80 |

候选没有新增编译或行为回退。9 个正例的 36 条腿完整编译、行为和 class/ctor/field 反射一致；GenericDeclaration 身份区分类变量与构造器变量。JADX 在 ObjectHold、ErasedCastHold、CrossHold、ShadowHold 上各四腿无法重编，不能继承其字段/构造参数同擦除即兼容的假设。Jarde 的 CallHold/ExceptionHold 各四腿仍有原有 Object 实参不能传给 T 方法的整类编译失败，正文已恢复，不能写成“正文拒绝”。这是普通泛型调用参数的独立未处理债务。

CrossHold 四腿恢复 U 构造参数，字段 T 擦除为 Object并行为一致，不宣称字段反射保持。PeerNewHold 一参数构造器包含额外 new，仍发布 Object；二参数目标恢复原 class T，字段由第一构造器实际 Object 写保持擦除。完整类四腿编译/行为正确，不能把这个部分恢复计为全反射正例。ThisDelegateHold 两个构造器继续保持 Object，四腿完整编译和行为一致。

前片泛型字段写的 23 族双 JDK 独立重放，基线与候选均为每腿 22/23 完整编译，全部 22 行为一致，六族完整反射仍保持。SCGB 是原有正文拒绝。输出保存在本片 `results/field-regression/root`，没有覆盖前片的已验收取证。

## 门禁与可重放产物

构造器专项 6 项及旧方法级构造器 4 项通过；lib 186 项在 incoming 门初版通过，后续同类 new 扫描修复由完整工作区门禁验证。最终 fmt、CI 同口径 workspace/all-targets/all-features clippy 通过；两固定 seed 各 3,243 passed、0 failed、93 ignored；显式 ignored P3 3 项、functional-constructor 整类 1 项、bound-receiver 整类 1 项通过；strict OpenSpec 321/321 通过。记录在 `results/local-gates`。JDK25 instruction-boundary oracle 由远端实际 JDK25 验收，最终远端结论按最新 main HEAD 查询。第一轮曾因旧 Hold 的 Object 断言停止，保留日志；修正为 T 参数/字段和完整原类反射一致，并额外验证 class GenericDeclaration 身份，再完整重跑两 seed，未放宽其他边界。

完整源码/原 class/jar/javap、JADX 与基线日志在 `evidence`；最终 root 对照在 `results/root`，初版回退在 `results/initial-root`。CLI 与生产源码 SHA 在 `results/acceptance-manifest.json`。最终生产源码只比完整 Java 重放 CLI 多一项等价 lint 修正：map_or(true, predicate) 换成 is_none_or(predicate)，工作区门禁在修正后的源码运行，不把重放 CLI hash冒充修正后新二进制 hash。

下一片 raw receiver 的实测证据保存到 `openspec/evidence/raw-receiver-source-selection-patrol`，没有混入本片生产代码。只读巡查确认 static raw 参数/保留 raw alias 与被 renderer 折成 this 的 alias 必须区分；field T 的反事实编译不是当前反编译输出，不计为本片恢复。

## 主线交接与远端复验入口

实现、冻结输入和独立验收已合入并推送 main；仅剩 main 分支。15 个工作树的辅助树均为干净 detached 副本，HEAD 全为 main 祖先，未留下实现分支占用。受固定任务保护的副本被 Codex 归档工具拒绝删除，保留保护，不绕过。共享 target 已清空：先释放 17.1 GiB，再清理 CI 修正复验的 979.5 MiB。

首次远端 CI 第二 seed 的既有 bulk 并发断言失败已单独取证和修正，见 [CI 验收说明](ci-delivery-verification-root.md)。修正提交 `d6598866` 的远端 [run 37808909595](https://github.com/LordCasser/jarde/actions/runs/37808909595) 执行全工作区两 seed、实际 JDK25 oracle、P3、构造实参、依赖边界、strict OpenSpec 及 MSRV/fuzz/supply chain。最后的交接文档提交不再改动生产或测试；接续时使用根 handoff 的命令核对最新 main HEAD 的实际 CI，不从历史基线推断状态。
