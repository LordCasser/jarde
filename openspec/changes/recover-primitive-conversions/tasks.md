## 1. 固定原始转换与链式语义

- [x] 1.1 将15个opcode基线及root链式对照收成一份最小永久Java8主class，helper/runner保持源码；记录hash/Code/真实opcode、完整原/JADX/jarde阶段与修前红测试。253项原始审计保留独立输入，JADX13项重载及7项链式错误不得当oracle。root统一冻结语料。
- [x] 1.2 补充同型局部保存、表达式分组、转换后字段/数组消费以及合法boolean descriptor、旧局部、丢弃/重复消费的内存边界；输入先经JVM验证，明确无转换的窄局部/ireturn债务不混入。

## 2. 接通现有Cast路径

- [x] 2.1 解码15条转换的源/目标事实并核对真实一元SSA操作数，固定Int族/Long/Float/Double类型及boolean拒绝单元回归，不改frame模型或另造转换框架。
- [x] 2.2 用现有Cast逐层表达结果，覆盖i2b/i2c/i2s静态类型、舍入往返与调用descriptor；复跑统一emitter分组和既有invocation/negation/cast回归，不新增转换消除。
- [x] 2.3 将转换接入producer reader、失败追溯和共享求值位置/保存；验证左右调用、异常优先级、独立语句与旧值，不以转换纯净为由重算操作数。
- [ ] 2.4 若浮点常量已接入，复用其NaN准入和有界闭合表达式判断，实测转换链不会重新触发改变bits的折叠；完成IR/工作/深度/取消/来源预算、essential/all和commit/replay对照。

## 3. 完整执行与root验收

- [x] 3.1 实际恢复的完整类原样javac/执行永久fixture，并重放180项原始及73项链式输入，逐行比较整数、raw bits、调用目标、trace及异常；不可手改输出或删方法。
- [x] 3.2 root独立审查转换链和消费类型、运行原始审计，复跑调用、算术、取负、比较、浮点、移位组合及延期值相邻回归，未落地相邻功能的组合边界单列。
- [x] 3.3 root统一Java包、语料census/fingerprint、fmt、严格clippy及OpenSpec strict，记录既存门禁债务和ireturn/窄局部后续任务，不扩大本项。
