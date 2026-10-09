## Why

上一片恢复了嵌套int数组的语句位置`+=`，但`return a[i][j] += x`仍使完整类无法编译。一维、二维、三维、带副作用下标/RHS和RHS换行对象的完整11成员类已经实际复测：原程序两真实JDK2/2，JADX1.5.6 default/none四腿4/4，当前冻结Jarde两腿均compile1。root独立156checks确认完整源、固定Runner、空CP/SP、新classes与raw双流，没有通过删失败成员缩小范围。

差距是更新后的值也被返回。现有IndexAssign是语句，PostfixUpdate返回旧值，不能直接表达本例的新值语义。应在现有证明和表达树内补齐这一种值消费，不做新的恢复pipeline或全局copy推断。

## What Changes

- 补一个具有结果值的数组赋值表达方式，复用既有AssignOp、数组/index/RHS表达和assignment precedence；只由准确int加法+紧邻ireturn证明构造，不把返回新值塞进postfix旧值语义。
- 在既有左值复制、类型、唯一消费和时序/依赖证明上核对额外dup_x2四输出及store/return两个准确消费者，整组闭合后一次发布。
- 保持语句compound、postfix、不同左值和row-Phi边界。更新上一片仅因statement范围而拒绝returned的测试预期，不删除merged拒绝或真实来源断言。
- 固定完整新控制类及Runner，真实双JDK完整对照；继续回放上一片4腿和旧24腿，新增显式CI Java执行。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`: 已证明int数组复合更新可作为紧邻return的新值表达式，保留一次求值、异常顺序和来源闭包。

## Impact

主要在jarde-java AST/emitter/既有写回和return证明，以及相应表达遍历/类型与来源维护点；不会新增pass、类型服务、合成局部变量身份层，或修改JVM frame/decode。增加夹具/测试和工作流显式执行步骤，reader census/指纹由root实测维护。完整基线见上一片results/returned-next-baseline-v1与returned-next-root-verification-v1.json；当前CLI SHA8745071312981d4eafb8fbb738e6a1fa1edbb55753062bfae057e879c494bb7a。此规划不代表产品实现或DT26/71单元完成。
