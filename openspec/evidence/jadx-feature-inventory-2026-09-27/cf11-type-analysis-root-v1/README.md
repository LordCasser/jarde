# CF-11 类型路径诊断

本目录保存三种不同证据：普通 LVT 元数据传播的只读核查、固定 TestNestedLoops 方法泛型签名的输出对照，以及局部引用类型的真实诊断运行。各文件原字节由 `archive-pins.json` 绑定。

## 局部类型实际运行

root 在产品 73adae72 的源码上临时应用 observation-only patch，诊断真实 baseline-v1 的 `noCastAssign(Ljava/util/List;)Ljava/lang/Object;`。精确测试实际 1 passed / 0 failed，日志完整；但测试结束后的 free 为 5352644608 bytes，低于 5 GiB（5368709120）。守卫记录 `observed_after_exit: true`，runner 和外层 invocation 均 exit1。本轮 status 为 failed，不能算正式诊断验收成功，更不能算类型修复。

finally 固定 `cargo clean` 实际 exit0；两份文件恢复原字节，源码闭包 4744→4745→4744，target 不存在。没有降低门槛或把测试成功拼接成 runner 成功。

独立失败审计 v2 实际 exit1，解析 `event=end` 时出现 IndexError，facts 未产出；原脚本、收据与 raw 均保留。v3 仅修正事件 token 分组，root 实际执行 exit1，完整日志、source pins、恢复与 cleanup 事实核验完成；三条正式拒绝分别是资源守卫、runner 非 observed_same_ir、invocation 非 exit0。v3 仍 status=failed，不能称为诊断通过。该审计固定这一次失败输入，不适用于未来成功运行。

完整 raw 日志显示同一个 `LocalVariable { slot: 1, index: 0 }`：BCI2 写初值 String 常量（stored Value4 / written Value5，SSA Ref Unknown）；BCI25 写 Iterator.next 的 Object（stored Value12 / written Value13，SSA Named Object）；BCI10 的 Phi Value2 是 Ref Unknown，BCI29 读该 Phi。`decide_types` 最终却选 String，公开正文仍出现 String 局部接收 Object next 的非法赋值。这是此次失败运行里的原始观察，尚待资源条件满足后正式独立验收。

root 源码核对确认，现有决策以最早写入的 `written_type` 为起点，后续循环只识别 reference/primitive 类别冲突。已有 `direct_reference_write_type` 和 null-leading 的逐写证据可供后续设计参考；目前不新增 LUB 或变量身份服务，不直接按调试表决定声明类型。成员调用、参数约束、数组、未知引用、复用与异常路径需要主动反证。

## 元数据与方法泛型

reader 保存普通 LVT descriptor；facade 的普通映射只传 slot、name、range，只有唯一匹配 LVTT 才调用 `with_type_metadata`。因此上述同次日志的 descriptor=None 符合当前实际输入，不代表 class 中没有 LVT Object。既有 DebugLocal 有字段可承载 descriptor，是否需要扩大传播仍取决于源码类型证明，不能把调试事实直接当证明。

固定 TestNestedLoops 的 method Signature 确实带两个 List<String> 参数。当前候选已恢复两个 Object foreach，保留 String cast/赋值，但方法头仍为 raw List，且 `ordinary_generic_source_unproved` 仍在。foreach 恢复与泛型方法契约投影分别核验；旧同 IR foreach 拒绝日志不能当当前泛型拒绝原因。后续只诊断该方法泛型投影的准确失败子门。

当前 foreach 产品自身 CI 与干净主线验收仍独立进行。以上分析不增加固定 71 单元 / 612 文件的完成数，不关闭 CF-11。
