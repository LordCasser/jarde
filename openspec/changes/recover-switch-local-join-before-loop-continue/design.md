## Context

CF-13 原 class 的 loop header 在 BCI 4，switch selector 在 BCI 9、`lookupswitch` 在 BCI 12。case 0 的 BCI 43 跳向 BCI 55，default 从 BCI 52 落向 BCI 55；case 1 的 BCI 49 跳向 BCI 58，BCI 58 是外层循环更新，BCI 61 回到 BCI 4。BCI 55 的共享 `score += 3` 只能在未 `continue` 的路径执行。Jarde 的 `switch_region` 当前在全方法 post-dominator 为 loop `break_target` 时才调用 `switch_loop_join`；该证明的 terminal 集只含 loop break target，且 `region_at` 对当前循环的 `continue_target` 不直接生成 `LoopContinue`，因为普通循环尾也会到达该更新块。于是 switch arm 可能吞并更新或与另一个 arm/外层 loop 重复拥有 block。

JADX 的 `SwitchRegionMaker` 会按 switch 出口集合与 loop end 插入 continue，`SwitchBreakVisitor` 再清理 terminal 后的 break。CF-13 固定版本输出仍含 `continue; break;`，不能作为可复制的目标；本实现借鉴其区分 switch 与 loop 出口的顺序，但以本项目的 canonical CFG、Region owner 与物理 BCI 做证明，直接构造正确语句。

## Goals / Non-Goals

**Goals:** 对一个已识别外层循环里的整数 switch，证明至少两个正常 case 路径在唯一 switch-local join 相遇，另一个完整 case 路径精确结束在当前循环 `continue_target`，且该目标由循环外层更新拥有。随后生成可重编、运行等价、来源唯一的 switch/continue/共享语句。

**Non-Goals:** 一般 switch 出口重排、任意嵌套 switch、跨循环 continue、异常边、多个候选 local join、额外入口、字符串 switch、修改通用 post-dominator 或后处理不可达 Java 语句。

## Decisions

1. **先证明局部 join，再走访 case。** 当全方法 post-dominator 恰为当前已证循环的 `continue_target`，不能直接把它当 switch join。复用现有 `switch_loop_join` 的 bounded candidate walk，加入对这一个准确 continue target 的终止路径；要求候选在当前 loop scope、由 switch 支配、不为 case entry/loop boundary，至少两个不同正常 case arm 达到候选，且候选唯一。每个 arm 只能到达此 join、经已证 loop transfer 结束，或到达已有安全 terminal；跨 case、多入口/循环或不可分类路径不授予证明。原有 loop-break local join 路径与拒绝规则保持。
2. **精确归属 continue，不抢循环更新。** switch case 路径确实从自身 transfer source 指向当前循环 `continue_target`，且未先进入 switch join 时，输出 `Region::LoopContinue`，来源锚到该 transfer BCI。此判定限于前项证书覆盖的 switch case，不能将普通 loop 尾部到更新块也改为 continue。BCI 58 的更新由外层 loop 一次拥有，BCI 55 的共享语句由 switch 之后的 continuation 拥有；每块仍须通过现有 completed Region tree exactly-once 检查。优先传递现有 `Frame.loop_targets`、`switch_join` 的事实，若缺少明确的 case-scope 身份，仅添加私有 frame 约束，不扩大公共 IR。
3. **由结构保证无不可达 break。** `LoopContinue` 结束该 arm，不在后面合成 switch `break`。正常到 BCI 55 的 arm 仍可按现有 switch join 规则结束；共享语句只在 switch 之后走一次。不要通过发射后删除 `break` 修补，也不要沿 BCI 顺序推测源码执行顺序。保持 BCI 9/12 selector、BCI 49 continue、BCI 55 共享语句与 BCI 58 更新的来源可追溯。
4. **失败局部回退，预算原子。** 证明不唯一、handler/异常边、多入口、case 嵌套或资源不足时，不发布看似完整但丢失路径的 Java。保留现有 quotation/explanation 与预算/取消结果；不要让该变化放宽现有 `SwitchLoopAdjacent.nested` 负例。

## Verification

增加以 CF-13 已归档 class 为输入的定向测试，检查完整源码无 `@bytecode`、无 `continue;` 后不可达 `break;`，每个关键 BCI 一次归属且对应来源。用原 class、修后 Jarde 完整源码分别 `javac --release 8` / `java -Xverify:all`，runner 输出精确为 `38`；固定 JADX 编译失败作为对照事实保留，不把它算作行为等价。运行 `p3_switch_loop_exits`（尤其 nested 负例）、相关 loop/switch tests、预算/取消、格式、OpenSpec strict 校验。实现时用小型负例验证多候选 join、非 case 路径到更新块及跨 case/嵌套情形保持引用，清理独立 Cargo target。
