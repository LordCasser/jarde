# Design

## Context

动机见proposal。已交付主线f4a81440d与产品51de58710保持4922项编译源码/模式/冻结CLI绑定，产品自身CI通过。本片只有原类控制和真实探针，不能声称恢复完成。

NoCall两编译器Code均为62字节/36条指令，7个非克隆canonical块，正常自然循环为10/16/40、唯一latch40。异常表为[22,34)→46与[46,48)→46；34/40/48三份iinc在保护外。54索引更新与57回跳实际融合在块40，不存在独立块54。当前Region实测terminal sets为空、for_header=None，loop scope不含return28/handler46。

## Goals / Non-Goals

**Goals:** 在这个完整有界形状中证明局部更新、已保存返回、每条正常离开路径、异常表与真实SSA；复用现有结构完成全类恢复。

**Non-Goals:** 不改通用iinc清理文法、P2保守exception edge/phi，不添加loop终端全局registry、新循环发射框架、公开枚举字段或依赖。无需强制counted/enhanced for。

## Decisions

1. 在guarded的FINALLY分支、resources之前加入专用完整证书。先固定36 opcode/rows/current位置的有限无收费判别，非候选保持旧路径计费。通过判别后逐指令、边、row、effect、SSA/phi/use收费与poll，最后检查点先于证书发布。拒绝全局扩充finally_copy_shape/cleanup_sequence，因为同slot不能证明同值，亦会改变无关候选。
2. 返回既有Shape::Finally(structured、SavedReturn32/39)、lead16..22、body22..34、owned仅16/28/40/46、join实际header10。私有FinallyFallthrough追加可选exact loop_header：新片Some、旧片None；cleanup40..43、transfer43、Tail54..60。lead中的arrayload19/store20必须完整证明并发射在try外，不能放宽旧statement_free gate。
3. 对每块指令与正常/异常边作完整匹配，核无foreign入口、额外row/clone/unreachable。protected22..34与self-binding46的实际effects均不抛且无handlers；arraylength5、arrayload19确实可抛且保护外。保留raw表与edge，不把保守handler当不存在。
4. 证明c/header phi只由init1与fallcleanup40进入；index/header phi只由init8与update54进入且步长1；arrayalias/length与element→x链闭合。返回28→29/31→32→37→39保持Int定义/唯一消费者，先保存再清理。normal34/40读cphi；handler48实际读另一phi，输入包含cphi、normal34新值与handler48自反馈，不能强行同值。结合全保护区nonthrowing证明其异常边不可执行，并核handler同slot/Int/+100及store46→load51→throw53完整链，列出全部写入和消费者。
5. region::finally_body仅在私有loop_header与outer.own_loop、Plan.join、当前真实loop匹配，且current16/cleanup40已在outer.scope时，局部授权唯一已证terminal28；child scope仍exact16/28，handler46只在成功后按既有owned认领。保存/恢复visited，要求精确覆盖、save一次、next40且cleanup未走。无需改通用loop_terminal_returns。
6. 复用现有builder Finally checkpoint及saved-return、Tail发射。普通while body在try后发射54一次。现有for_header对latch其它effects的拒绝保持；若实测整合产生新continue boundary问题，仅对该私有证书修正，禁止全局清空loop targets。fallback或未声明局部按既有checkpoint处理；停止不得发布未提交正文/来源，不退款或改写原因。不声称每个内部helper Err都在本地立即restore，未发布产物的原子性按既有公共契约验收。
7. 继续使用项目已有维护中的reader/jvm/Java结构与Rust标准集合；无第三方库提供本项目SSA身份/预算/ownership证书，故仅增加小型私有证明，不引入库或许可负担。parse成功、Java8 dialect、选择Java8/23运行时、验证与源码恢复分别报告。

## Risks / Trade-offs

- handler phi与normal值不同 → 明确完整输入集合与nonthrowing前提，反例必须拒绝，保留原始事实。
- 当前driver无法观察待return后的dead local更新次数 → 不改原Java/driver/oracle，使用结构证明与真实class反例，运行证据只宣称实际观察范围。
- inner scope与outerloop控制混淆 → 私有exact header授权单一return；不把handler放进正常child范围；核所有owner/visited与后继。
- 整类FY/FX仍有其它失败 → 全方法保留、单独重编screen，父任务不因独立fixture成功而全部勾选。

## Migration Plan

先保存旧控制与探针闭包，再增加永久fixture/测试和最小产品改动；最终原样全源码编译、隔离双JVM运行、兼容/资源门禁与exact-head自身CI通过后归档。回滚限本片提交，保留历史证据；主规范仅在成功验收后同步。
