# 分离的架构债务

本次只修改 init/build 的数组及构造组合。不把以下问题混入数组准入或使用新门槛掩盖，后续各取真实基线再建立独立 OpenSpec。

## 普通 self-new 被送入匿名接口投影

ReverseArrayOrder 的 natural 输入在旧 CLI7287 下尚因数组拒绝；新 CLI0c9（早期诊断构建）恢复全部三个方法，各为 Structured / Complete，但类级变为 Partial、CLI4：`anonymous_typed_nesting_unproved`。两版 javac 均如此。swapped 仍拒绝方法组合，未触发投影。真实命令/源码/raw 位于 reverse-originals-root-v2；该诊断 CLI 的库测试整体未通过，不能冒称最终产品。

facade 调度仅以唯一 direct-return allocation 作为匿名投影候选。普通 `new ReverseArrayOrder(...)` 因而进入 project_class_source_anonymous_interface；该类没有匿名 InnerClasses row，适用性不成立被映射为聚合 Unsupported Partial。这不是预算/取消停止。最小独立方向是先排除确定的 allocation internal name == root this_class，再保留真正匿名候选及其依赖/Stop 契约；不吞掉 Budget/Cancelled/Deadline。需要普通自构造正例、真实匿名正例和依赖停止反例共同验收。

本次完整数组正向家族与原始 TestFinally3 不受这个类级问题影响。最终实测 SameOwnerArrayArguments 的三处构造使其不满足唯一匿名候选，四配置均 class Complete，8重编/16运行通过；只有 ReverseArrayOrder 的唯一 self-new 仍触发该 Partial。后者的方法/源码运行结论必须与类级 Partial 分开记账。

## 通用 fallback 构造来源缺口

完整诊断类 DirectArrayConstructorArguments 的 twoArrayArguments 保留原断言分支。数组构造已成功恢复，但 AssertionError/StringBuilder 是另外的呈现拒绝；Quality 为 Fallback，`fallbacks` 区域级列表可以为空，正文以真实 BCI49/52 注释解释，不保证含有 `AssertionError` Java 名称。

公共测试实测还发现 StringBuilder allocation head53 没有 source-map 记录；其 NewRecord.presented 仍可为 true。这表示先前构造验证记录与最终 fallback 正文之间存在来源/承诺不一致，不能凭 presented 标志推断最终输出已经包含它。final-gates-root-v2/v3-failed 保留原始失败断言和 raw；control-provenance-baseline-root-v2 已实际对照旧7287与新e01两CLI、双编译输入和default/all：head53在两产品均无映射；all的presented为true，default没有请求news。因此确认为既有债务，后续独立修复。

本次公共正例全部构造均核完整来源；诊断控制只核断言分支之前 head<49 的完整数组构造及全部 map anchor 的物理身份，保留整方法 Fallback 和真实拒绝正文。原输入、所有方法均不删。独立正向完整家族88方法配置与 TestFinally3 100配置均通过全BCI来源审计；这些结论不覆盖本段通用 fallback 债务。

## 对抗性审查未证实的顺序风险

Luna 初步因 prefix/suffix 使用集合而提出一般重排疑虑，复审已撤回“发现生产缺陷”的结论：数组 element/store 连续区间、精确唯一 use 和 sources/effects 会排除额外 stack shuffle。原合法 SWAP 变体已由双 JVM 验证并被现有门拒绝。额外 suffix dimension 重排已在两个编译输入上通过双JVM8次验证：原trace123，变体132且实参值不变；自然形态被接受，变体实际首拒在array BCI5的jre_new_interleaved_effect，不冒称首拒在SWAP。无错误接受，不增加排序机制。

新增集合的规模由已计费候选、依赖与 canonical effects 限定；额外逐集合操作收费建议已撤回，不凭静态猜测重复计费。
