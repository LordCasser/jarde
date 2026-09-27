# EM-03：`MethodParameters` 与 `throws` 的 Java 8 首片

固定 JADX 为 `2fb1b16386941660fda07e9017285aec40fcb37f`，Jarde 基线 CLI SHA-256 为 `8f1f0012324350e4fc65c7fef4b3e3835102e6d4fda00df280b466270c98e727`。五份相关 JADX 测试的 SHA-256 由 [replay.py](replay.py) 检查。`TestMethodParametersAttribute` 明确用 `-parameters`、无 debug 编译并断言名称和 `final`；`TestThrows` 断言一组真实 Java 源码的 `throws` 形态；`TestMissingExceptions`、`TestInvalidExceptions`、`TestIncorrectMethodSignature` 则是 Smali 输入，不能用本轮 Java 8 正例代表其畸形元数据边界。

[input/em03/](input/em03/) 的完整类以 `javac --release 8 -g:none -parameters` 编译，隔离一个带两参数且第二参数为 `final` 的方法、一个只有声明 `throws IOException` 的方法和一个实际抛出该异常的方法。共同 Runner 检查运行值、`Method.getParameters()` 的名称及 `final` flags、两处反射 `getExceptionTypes()`。固定命令：

```sh
python3 openspec/evidence/java-syntax-2026-09-27/em03-method-signatures/replay.py \
  --jarde /tmp/jarde-cli-accepted-dt31 \
  --jadx /Users/lordcasser/workspace/testzone/jadx/jadx-cli/build/install/jadx/bin/jadx \
  --jadx-checkout /Users/lordcasser/workspace/testzone/jadx \
  --out openspec/evidence/java-syntax-2026-09-27/em03-method-signatures/baseline
```

[summary.json](baseline/summary.json) 中原/JADX/Jarde 三份**完整源码**均通过 Java 8 重编与 `java -Xverify:all`。值和异常类型的五项运行观察中，`throws` 两行三方一致；参数反射一行原/JADX 为 `paramStr:false:number:true`，Jarde 为 `arg1:false:arg2:false`。第二次重放的 summary 和 Jarde 源码与基线逐字节一致。[javap](baseline/javap-Signatures.txt) 确认 `named` 的 `MethodParameters` 属性列出 `paramStr` 和带 `final` 的 `number`，没有 `LocalVariableTable`。Jarde 源码中参数名由 `arg<slot>` 回退生成，第二参数没有 `final`；这会改变重编 class 的可见反射事实。

架构缺口位于属性来源而非 `throws`：`jarde-reader::attribute_facts` 已对成员级 `Exceptions` 作有预算的解析，却未将成员级 `MethodParameters` 作为类型化事实；`src/class_source.rs::parameter_names` 只从 `Code` 内的 `LocalVariableTable` 取名，因此 `-g:none` 时退回槽位名，`arguments` 不读取参数 flags。固定 JADX 在 `DebugInfoApplyVisitor::processMethodParametersAttribute` 先校验参数数目，再给参数代码变量写名与 `final`。Jarde 应保留自己的“同一方法声明和正文共享一种命名”约束：仅在完整、准确计数、合法位置与安全源名获证时，让成员属性与正文使用同一组名字及 final 修饰；缺失或畸形的属性仍保守退回，不能只改 header 而留下正文中的 `arg1`。见[窄 OpenSpec](../../../changes/recover-proved-method-parameters/proposal.md)。

本次只证实 `-g:none -parameters` 的可观测差距；有 LVT 的优先级、多槽宽参数、隐式参数、畸形属性及 Smali 三种负例仍需在实现中按证据另测，不能从一个正例推定 EM-03 整项追平。
