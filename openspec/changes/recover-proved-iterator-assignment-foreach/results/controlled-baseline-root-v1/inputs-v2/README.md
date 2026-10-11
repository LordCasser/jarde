# CF11 外层赋值控制源码（v2，准备稿）

本目录只包含 Java 8 控制源码、精简输入 manifest 和说明。源码尚未编译、运行，也未经过 JADX/Jarde。准备过程没有运行工具链或 Git 命令，没有编辑仓库。

## 正例与 oracle

`CF11OuterAssignmentTarget.positive` 在循环外声明并初始化 raw `String target = "seed"`。循环首句是 `target = (String) iterator.next();`；每次成功赋值后，runner 通过 `Runner.assigned(target)` 记录值。正常结束时方法直接返回 target，因此空循环返回 `seed`，非空循环返回最后一个元素；null 元素可以成功 cast 并令返回值为 null。

runner 的 trace 同时记录自定义 List/Iterator 的 `iterator`、`hasNext`、`next` 调用和成功赋值后的 observer 事件。wrong-type case 先完成并记录 `last-good`，第二次 `next` 后 cast 抛出 `ClassCastException`，因此没有第二条 assigned 事件。iterator/hasNext/next 抛错时 runner 捕获异常，并留下抛错之前的调用与成功赋值事件。异常展开后局部变量已不可由 runner 直接读取；这些 case 只提供成功赋值记录、失败前的副作用/调用顺序和异常类型，不声称直接观测展开栈上的 target。

`positive-hasNext-throws-before-write` 在 cursor 为 0 时由 `hasNext` 抛错。`positive-hasNext-throws-after-write` 使用独立的 `hasNext-after-write` 模式，只在 `phase == hasNext && cursor > 0` 时抛错；输入含两个元素，故首次 hasNext 成功、next 和赋值 observer 成功后，第二次 hasNext 才抛错。这使 after-write 路径实际可达。

正例 cases 覆盖：空列表、最后值、null 元素、先成功写入再遇错类型、null iterable、iterator 抛错、hasNext 首次抛错、next 首次抛错、成功写入后 hasNext 抛错。比较完整 collector 输出时使用 stdout、stderr、exit status、类文件集合，以及各次调用/observer trace；异常路径没有局部变量直接快照。

## 窄拒绝 controls

- `noCastAssign`：循环首句是没有 cast 的既有变量赋值。
- `bodyEffectBeforeAssign`：cast 赋值前有 runner 可观察调用。
- `extraNextConsumption`：cast 赋值前额外调用一次 `iterator.next()`。

它们只验证这些可见形状不误投影，不覆盖同一个 next 结果存在多个 SSA 消费者的更细微字节码情形。

## 完整类采集与实现边界

复用 `/private/tmp/jarde-for-condition-collector-root-v3/collect_controls.py` 的完整类流程：输入 manifest 接受多个唯一 `cf11.*` target 与一个 `cf11.Runner`；collector 编译并执行 javac8/javac23 原始源码，保存原 class 和 javap；对 javac23 class 执行 JADX default/none；再用 Jarde default/all 渲染原 class，并将生成源码和相同 runner 组合编译、执行。launcher 参考 `openspec/changes/recover-proved-hoisted-iterator-foreach/results/controls-baseline-private-prep-v2/run_controls_baseline.py`，为本输入绑定新的 pins 与独立输出目录。这里没有调用 collector。

当前 `crates/jarde-java/src/build.rs` 候选入口只接受 while body 首句 `Declare { value: Some(..) }`，所以目前正例 Assign 与三个控制的首句 Assign/调用都会在初始形状门被拒绝。固定 CF11 同 IR 诊断已确认外层 body 首句是既有 Assign，数据路径为 next17 → CheckCast22 → Store25；该诊断没发布 LocalVariable 身份，不能从 slot/name 推断绑定身份。候选扩展应只接纳首句 inline `Cast(Call Iterator.next())` Assign、保留目标/cast/store 和声明作用域、仅替换 next 叶子为 synthetic Object foreach 元素；精确调用 descriptor/receiver、唯一消费者、iterator 声明 owner/gap、phi/use 和异常 handler 等既有 proof 门保持不变。
