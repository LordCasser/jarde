## Context

CF-14 的原始 `NestedStringSwitchAudit.choose(String)` 在外层 String switch default 中再含 String switch。外层最终整数 dispatch 在 BCI 39，default 从 BCI 62 进入内层 lowering；内层 hash dispatch BCI 71 的 `b`/`c` 分支写 slot 4 于 BCI 105–106/120–121，hash default 和两个分支在 BCI 123 汇合，BCI 125 的最终整数 dispatch 指向 BCI 152/154/156 三个返回。现有结构 Region 持有 BCI 62/96/105/111/120，却把 BCI 123/152/154/156 留给 uncovered fallback，继而 `build.rs` 因 slot 4 跨引用区拒绝。固定 JADX 在此正例可重编运行，但对独立 hash 用途的 `ExtraHashUse` 写出未定义 `r0`；不能照搬其全局 String switch 改写。

代码路径提供更窄的解释：`region_at` 会把一个 switch 的局部 join 作为 `Run` 的 `next` 返回；根层与循环体有续接走访，而 `switch_region` 对每个 case arm 的 `(arm_run, next)` 丢弃 `next`，只把首个 region 收进 arm。BCI 123 尚未被结构化走访，uncovered 扫描因而引用它及下游目标。此因果关系是代码和归档 Region 证据的结合推断，实施阶段应以定向测试验证；诊断中的 “normal-flow leaves out” 不足以指认某种 canonical edge 被过滤。

## Goals / Non-Goals

**Goals:** 在嵌套 switch 的 arm continuation 经 CFG 证明只属于该 case 且仍在其词法边界内时，继续走访至该 arm 正常结束或已有边界；让 inner hash switch 与 final discriminator switch 成为相邻 Region，交给现有 `project_string_switches` 精确字节码证书折叠，并保持每块恰一 owner。

**Non-Goals:** 新 String lowering 模式、任意深度的复杂 cross-case flow、异常/子程序边、额外入口、独立 hash 值删除、从局部变量类型推断 switch 控制流、对 JADX 错误负例做补丁。

## Decisions

1. **继续 arm 的已证 `next`，不改 normal-flow 投影。** 在 `switch_region` 的 case arm 构造中，若子 region 返回 `next`，先核对它不是本 switch 的 join、其它 case entry 或 enclosing frame 的边界；要求 switch case entry 支配这个续接，所有 normal predecessor 位于该 case 的受证前向区域，且没有循环回入或已认领 owner。复用现有 `Frame` scope、`case_entries`、`forward_join_predecessors`、`visited` 与预算检查。证明不了则保留引用；不能默默忽略 `next` 或放宽全局 source ownership。若一般性多次续接会扩大边界，先支持该样本的一次 child hash→final switch 续接，并给出明确拒绝。
2. **把两个物理 switch 留在同一臂内的有序 `Region::Sequence`。** 内层 hash switch 和 BCI 125 final switch 都由同一个 outer default arm 走访，原顺序不变。已有 `project_string_switches` 递归进入 `Region::Switch` groups，再在 Sequence 内用 `stringswitch::prove` 校验 selector store、hash/equals、判别 key、被隐藏 BCI 的准确集合；只有完整证书才形成 `Region::StringSwitch`。不得直接在 `switch_region` 里推断字符串标签，也不得把 slot 4 跨引用区单独改成无条件声明。
3. **所有权和行为是发布门槛。** 对 BCI 62/71/96/105/111/120/123/125/152/154/156 检查 exactly-once Region owner，`choose` 无 `@bytecode` 且 Java 8 可编译。原/JADX/Jarde 五行结果和 null 异常一致；普通碰撞/分组八行不变。`ExtraHashUse` 中 hash 仍有独立消费者，必须保留其两层分派并与原 class 六行一致。固定 JADX 对该负例不能编译，不能要求三方运行等价。
4. **失败不发表半份方法。** 多入口、cross-case、循环/异常出口、预算或取消使 arm continuation 无法完整证明时维持现有保守拒绝或中止；不要让已走访的部分 block 被外层 arm 与 fallback 双重认领。

## Verification

以归档 CF-14 三个原 class 为固定输入，增加 Region 续接正例及至少额外入口/cross-case、独立 hash 用途负例，检查完整 class-source、source-map、预算/取消。对三方可编译者以 `javac --release 8` 与 `java -Xverify:all` 对照原值；保留固定 JADX `ExtraHashUse` 编译失败事实。运行 String switch、嵌套 switch、Region ownership 定向测试、格式、适用 crate check 与 strict OpenSpec 校验，清理独立 Cargo target。
