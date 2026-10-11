# CF11 结构化 void 泛型后续门：静态架构事实

已有实际 Signature diagnostic-v1/独立accept-v1只证实首拒在report普通generic_return_candidate：单block/noPhi直线VoidBody未覆盖循环，随后首句必须Return(value)失败；它没有执行后续declaration门。本记录只读源码，不冒称新诊断、运行或产品。

`crates/jarde-java/src/report.rs:6418` 既有GenericReturnValue::VoidBody仅记录unread_parameter_slots。`report.rs:8803` helper用rawCode/operands/SSA/effects全同位核验参数读取/写入，再拒所有parameter writes；目前还强制单block/noPhi/nohandler。故不能直接移除这些完整性条件；多block须按物理BCI一一绑定，不以顶层AST数量冒充全Code覆盖。

根facade `src/class_source.rs:7418` 的void_body_parameters_proved又要求每个parsed参数是type variable、参数化后的source spelling与physical相同，或其slot在已证unread集合。当前两个List<String>都是已读参数，source spelling又区别于raw List；所以即使单纯生成现有VoidBody candidate，这个下游门也不会接纳它。这是静态推论，不是该方法第二道门实际拒绝日志；必须以下一次最小same-run观测或实现测试确认。

`src/class_source.rs:8134` 的generic_void_body_declaration是另一条窄 `<T extends B> void(B,boolean)` 路，不用于普通两个List<String>的签名，不能把它当当前loop唯一入口。普通投影在上述parameter source proof处做决定。

下一步需要在既有Program/SSA/candidate中提供“参数化类型下实际读位置成立”的有限事实，不能把已读slot虚构为unread。当前fixed foreach body用Object元素、原String cast保留；List<String>迭代到Object在源码层可赋值，但必须逐实际parameter读取与Iterable来源证明，不能由Signature存在或唯一调用名字替代。可以先考虑只读iterable/Iterator初始化这类已证明位置，不预建通用generic type solver。

对抗控制：List参数参与原add(Object)等写操作、raw cast后传入方法、parameter重赋值、List<? extends/super T>及迭代元素转换、同名局部遮蔽、handler/Stop、多block缺失Code/SSA/effects项；不能借“同erasure”发布不可编译的List<String>正文。任何超出这个固定用法的泛型签名能力独立计账。

此研究属于同一固定CF11账本候选，局部引用写兼容性OpenSpec保持独立；无源码修改、无新增机制选择、无完成数增加。source-pins.json绑定此次静态输入，原实际诊断接受目录不改写。
