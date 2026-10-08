# raw receiver 字段选择：冻结输入与历史取证

正式 root 对照在 [验收目录](../../changes/recover-raw-receiver-field-selection/results/) 与 [root 验收](../../changes/recover-raw-receiver-field-selection/verification-root.md)。这里保留16族、真 Corretto8u432/OpenJDK23.0.1、debug/no-debug的64个冻结输入jar及源码；输入hash在 `versions/frozen-input-jar-hashes.tsv`。任何新回放都不能覆盖这些输入。

## 探针修正与历史边界

这里原有 `jdk8/`、`jdk23/`、`versions/`、`commands.tsv`、`runtime-comparisons.tsv` 是旧探针的历史输出，不作为最终行为验收。旧 ReceiverProbe 用 `params[i].isAssignableFrom(cls)` 识别receiver，Object RHS因此可能也拿到了receiver；普通Object形不能排除写入对象错配。原探针已按历史SHA精确保存到 `historical-pre-marker-probe/ReceiverProbe.java`，SHA256为 `621a87bc5004fd3e3c783a51be80421d4e3dc728ff25dc9b5db2975bda3d6b38`；旧manifest里的 `source/ReceiverProbe.java` 对应这份历史源，不能解释为当前源。

当前 `source/ReceiverProbe.java` SHA256为 `a411376a5e0aef02c9da74de267722cd25c06692bf745c6446613be3da069489`。只按物理参数精确等于subject class识别receiver，其余Object使用独立marker；末位RHS核对对象身份，NullRawParam明确没有RHS。数组递归核对marker、primitive array核对输入身份，Number核对17。反射同时记录物理类型、泛型类型、类和方法binder的真实声明身份，不按同名T合并。

旧64项还有空字符串classpath/sourcepath局限，不能据此证明隔离；初版不完整harness另保留在 `historical-initial-harness/`。修正脚本使用真实空目录，并拒绝已有输出目录。root已用新探针独立回放baseline和candidate，结果分别保存在正式验收目录；历史失败和旧输出未删除。

## 覆盖与出处

16族包括原六族，以及直接instance raw参数、宽槽、T[]/二维引用数组/primitive array、Number与Comparable<T>上界、null、多方法formal、保留alias和继承owner控制。每个jar仅subject与必要helper；Jarde/JADX helper均从同一jar反编译，不能借原helper源码使候选成功。Rust fixtures在 `tests/fixtures/raw-receiver-field-selection/`。

InstanceRawLocal的JADX输出把raw alias折成this后将Object写给T；四腿完整编译失败原文保留。Jarde保持字段Object，不能把原raw SSA来源用于放行实际this表达式。TypedReceiver/ShadowMethodT/MultiFormalRawParam字段恢复与方法API分别验收；不能借未发布writer Signature或sibling native头。

额外两个真实alias压力jar在Rust fixtures，来源SHA见 `alias-pressure-provenance.tsv`。重绑定/phi及同名分离作用域被实际输出合成同一local重复赋值，当前严格证明仍拒绝字段T，合法原Java及候选完整源码均可编译。合成accessor负例只对真实Corretto8 setter的ACC_SYNTHETIC做显式变异，先断言四条opcode，再验证essential/all证据选择不改变字段结果；不宣称javac原生生成了该synthetic flag。

JADX算法和用例出处：本地 `jadx-core/src/main/java/jadx/core/dex/visitors/typeinference/TypeBoundFieldGetAssign.java`、`nodes/utils/TypeUtils.java`，以及 `tests/integration/types/TestTypeResolver26.java`、`generics/TestGenericFields.java`、`generics/TestGenerics3.java`、`types/TestGenerics6.java`。参考访问点receiver决定成员类型的原则；没有复制Java代码或引入JADX类型传播机制。

## 重放

baseline CLI SHA256为 `3e7241e99b0068ede015b4dbc419212f5a4f63cb9a78f37429662b34f6d7a48c`；最终candidate与JADX库hash以正式结果各自 `versions/` 为准。脚本重新编译原源码并逐class比较冻结jar字节，完整重编和运行只使用新classes，不包含原jar。失败stdout/stderr、命令、源hash和反射均保留。

```sh
python3 openspec/evidence/raw-receiver-field-selection-2026-10-09/replay.py \
  --cli /absolute/path/to/jarde-cli \
  --cli-sha256 EXPECTED_SHA256 \
  --out /tmp/new-raw-receiver-replay \
  --label candidate-name
```
