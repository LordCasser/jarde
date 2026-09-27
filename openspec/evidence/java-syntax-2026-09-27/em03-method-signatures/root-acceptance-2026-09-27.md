# EM-03 主线独立验收

root 合并 `recover-proved-method-parameters` 后以主线重建 CLI（SHA-256 `e6055d64bc15970f24dc5acfcbeb05cd434f4f364aaa813f54f9a50540863bc9`）独立重放固定 [replay.py](replay.py)。原 class、固定 JADX `2fb1b16386941660fda07e9017285aec40fcb37f`、Jarde 完整 Java 8 源码均重编并通过 `java -Xverify:all`；六行 stdout 逐字相同，包括 `paramStr:false:number:true` 和两行 `[class java.io.IOException]`。Jarde 声明写 `named(java.lang.String paramStr, final int number)`，正文也使用 `paramStr`；与实现代理生成的源码逐字一致。独立摘要、三方源码和日志在 [root-replay/](root-replay/)。

代码审查确认 reader 按成员属性一次预算读取完整 `MethodParameters`，类源码仅对无 LVT、参数个数/槽位/Java 名称完整一致的非桥非合成方法作证书；名称进入同一次正文恢复，`final` 与声明一并出现。失配、无名、重复、非法 flags 及宽槽冲突保守回退。root 运行 `jarde-reader` 178 单测及集成、`jarde-java` 全套、`jarde` lib 135 单测、`class_source` 84 集成，workspace check、格式和本 OpenSpec strict 均通过。合并时另行修复了既有 `member_inner` 测试调用缺少 BCI 的债务和 EM-27 夹具增加导致的 reader 计数断言；这两项均是独立测试提交，未改 EM-03 生产逻辑。Smali 畸形异常/签名、LVT 冲突、隐式或合成参数重排仍待扩验。
