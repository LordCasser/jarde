# 构造器类作用域参数 — root 实施前审查

基线 main `790579e23e6e84e0b45fe3565d20df2295ab8459`；该提交远端 CI run [37786347619](https://github.com/LordCasser/jarde/actions/runs/37786347619) 的四 job 已成功。root 构建 baseline CLI SHA256 `34a5288badf6fb40020c5117ef749ae4705545b29ee35a9fefda48de24276b53`，临时路径不属于持久产物。

root 独立请求真实 Corretto8/no-debug 的 Hold class-source：完整 `super(); this.v=arg1; return;` 已恢复，构造器仍 `Hold(Object)`，标记 ordinary_generic_source_unproved；字段安全保留 Object，标记 field_generic_write_source_unproved@6。因此本片不是正文控制流缺口，也不能把擦除形整类可编译计为泛型恢复。

Luna 只读审计与 root 源码交叉核对：

- 唯一候选生产者为 report::recover_inner；generic_constructor_candidate 当前只有前导/空体两个 AST 语句。可在原位置加入初始化 this 后完整字段赋值序列，没有第二轮恢复的需要。
- class_source::project_method_signature 的 constructor 分支仅在 method formals 非空时进入；generic_constructor_declaration 同样要求它非空。新的类 scope 分支必须同时调整分流和声明门，不能只去掉名字拒绝。
- facade 的 method commit → same_class_published_method_parameters → field commit 单向顺序成立。published 参数只认可 generic_signature_projected，Settled 和候选 Signature 不是实际发布事实。
- facade::field_write_source 已证明 RHS 的 Entry 参数和唯一 put consumer，但未证明 receiver 是初始化后的 this，也未证明完整正文；新候选必须补这些事实。physical fields 从本次 read 传入 class_source 验证唯一实例声明，不读字段 Signature，不以字段类型发布反证构造器。
- 匿名 superclass fold 以 candidate 的全部 forwarded slots 和 initializer absence 决定隐藏 body；静态成员空构造器还独立要求 descriptor ()V 与恰好三条物理指令。新增 field-write 候选要显式排除两类旧消费者，不能只依赖当前参数非空的偶然保护。
- tests/generic_constructor_projection.rs 的 ConstructorUses 方法级字段赋值仍在既有拒绝族；本片保持它，新增字段初始化投影仅为已发布类 scope 且 method formals 为空的参数。旧 generic empty/forward 不受影响。
- 同类调用绑定门仅证明物理目标/重载闭合，不证明所有泛型源级实参适配；本片不放宽它，较广调用类型债务独立记录。

root 不采用审计建议中的“只支持一个引用参数”：它会缩小已定义的恢复里程碑。完整 direct field-write 序列使用同一个既有候选即可覆盖多字段、T[]、bound 和 long/double 槽前缀；必须按物理参数槽与 Entry/初始化定义验证，不能仅放宽长度门。

JADX 参考为本地 SignatureProcessor.parseMethodSignature / TypeUtils.expandTypeVariables（类与方法变量先定位、再逐位对照物理参数）和 TestGeneric8 的 TestNumber(T n) 正向断言。该测试本身为非静态成员类，不能把顶层 fixture 闭环写成完整 TestGeneric8 追平；TestConstructorGenerics 则是菱形局部变量测试，不能作为本片覆盖证明。

不新增 pass、IR、fixpoint、parser 或赋值转换表。不以“编译成功”替代 GenericDeclaration 身份反射；未知字段、改写参数、EH/phi/call、重复 BCI、预算/取消各自需要可靠负例。原源码中的 Object 擦除 unchecked cast 在 bytecode 里可能不存在，不猜 cast。
