# CF-11 类型诊断实际验收

两轮 observation-only 诊断均由 root 串行真实执行，独立验收器均实际 exit0。原始命令、stdout/stderr、输入及恢复记录由 `archive-pins.json` 绑定。这里只接受观察，不代表类型修复、泛型修复、整个 CF-11 或全工作区重新验收。

## 局部引用类型

observation-v3 的 runner 实际 exit0，精确测试1/0/0，guard_stop=None，target 峰值327613755B。4744→4745→4744输入、两份文件原字节恢复、默认Rust栈、固定 finally cargo clean exit0、target absent均核准。独立 accept-v5 exit0，status=observed_same_ir，SHA256 `27bc1b0e46686d8d11c10478d6527f4ec0f0b175dcbe6c30af9044375ebd053d`。

新增完整读链：Local1/index0的Phi Value2→Load29→Stack0 Value14，Value14只有BCI30消费者；该SSA instruction为opcode176、读取同一Stack0/Value14、没有写出，operation=Return。物理方法descriptor返回Object。首写String常量与循环Object next写属于同一现有LocalVariable，最终声明仍错误选String。读链已实测，不再借相邻BCI或调试名字推断。

最小设计方向是在现有逐写/声明决策中核准完整生命周期的写入兼容性，同时核准读取处的静态类型要求；不要无条件Object化。no-debug、参数、slot reuse、成员调用和数组控制仍须纳入后续实现 spec。旧v2虽然测试进程exit0，但结束余量低于5GiB、runner失败，原记录独立保留，不能用v3覆盖它。

## 方法泛型签名

method-signature-v1 runner实际exit0、精确既有测试1/0/0。default/all/no-rule-details各一组方法投影和Program/SSA事件，三组原始内容一致。target峰值353652756B、guard_stop=None、4744输入恒同、两文件原字节恢复、default stack、finally clean exit0/target absent。独立accept-v1实际exit0，status=accepted_observation_only，SHA256 `1dfdff9c2d1bc9f028fd997e0ef4d997db0f153b55bfdb9f11f4f1692ff2b306`。

同次raw明确Signature含两个List<String>，解析与对List/List descriptor的erasure证明成功；candidate=None、same_class_body_proof_present=false。最终拒绝原文仍是 `ordinary_generic_source_unproved: same-run Program/SSA cannot prove the body under parameterized types`。

Program非ragged，13 statements/5顶层nodes，Code45指令、0handler，SSA13blocks/21Phi。首个candidate拒绝源行为仪器化源码9154，对应原源码9115附近的“首句必须是带值Return”检查；此前严格直线void分支要求statements等于顶层nodes、单SSA block和无Phi，当前循环体不满足。root核对候选及原源码，不把首拒误归为Signature parser或descriptor擦除失败，也不借旧foreach门解释它。

下一步先分析如何复用现有Program/SSA及源码类型证明覆盖这个结构化void循环体。不能简单去掉单block/Phi限制就发布泛型声明；参数的所有实际使用仍须在List<String>契约下成立。此片没有改parser、泛型推断、body证明或产品输出。

方法诊断导入helper生成的一份本仓pycache已由root准确清理。产品73adae72自身CI和最终干净主线审计仍单独验收；71/612完成数不变。
