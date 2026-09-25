# root 独立 switch 返回下推反例

root 从 sibling `switch/` 的源级输入删除无关 keepPool 方法，再重新 javac、应用相同结构的36-byte合法Code，并以major49旧验证器形式执行。删除发生在重新生成输入之前，没有修改反编译结果。原输入 SHA `25f1a74ac01c1bea3563e679958e0ed6395219ade042cd87a59eb496597457ef`，patched class SHA `e2373834340108fdb7caecdfcb06cecdc9e1f8fdb43e8ab14028401a6faf6b9f`；精确Code、major、StackMapTable处理和三方日志见本目录。

原 class 的6项通过`-Xverify:all`，JADX完整输出编译、执行逐行一致。固定首版CLI feed5c 的jarde完整输出零引用、javac成功，但正常两个分支仍重复调用：trace12变121、32变323；另外四项producer/mark失败结果相同。首版CLI复制到`/tmp/jarde-cli-deferred-first-feed5c`仅用于保存这次对照构建，SHA记录于summary。未来重放可指定对应CLI路径，不能把后续不同hash的结果覆盖成本次基线。

源码审读可确认switch_join预先渲染每个arm的返回值，之后才构建arm语句；而首版同块候选规则未为跨join消费提供成功绑定证明。不能把根因直接表述成“已存在的绑定只是尚未提交”：候选本身可能未创建。可证实的缺口是预先决定的返回表达式和随后呈现的arm没有共享一次求值/拒绝决定，导致调用作为独立语句与返回值重复出现。

这是deferred-order的跨区域拒绝边界；当前设计允许在无法证明正确保存时给出完整来源拒绝，不要求为此新增任意stack phi恢复或跨区域提升机制。若保留正常Java输出，则必须实际保持唯一调用、原trace和异常先后。
