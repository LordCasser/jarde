## Context

动机见proposal。Root同版本failed-api-0.bin已经精确绑定CF11 focused-v4执行的生产源码；真实LLDB默认线程在render_value_definition的原生stack probe崩溃，链路为多次Builder::region→block→instruction→render_value。Region walker不在该调用链。机器码Builder::region单帧0x31000+0x6f0+0x20=202512bytes；render_value_definition约42512bytes；Engine prepare_physical_class_source约272336bytes。数字只代表该ARM64 debug构建，不当作跨平台固定预算或调用链总量。

## Goals / Non-Goals

**Goals:** 把巨大变体的临时变量留在只为该变体调用的私有函数中，使普通Sequence/If/Loop嵌套不承受所有Try/finally复杂分支的栈帧；真实完整类在默认2MiB测试线程实跑。保持既有停止、declared/settled/stmt缓存、RegionPath、字节来源、作用域和表达式副作用顺序。

**Non-Goals:** 新通用AST遍历器、改表达式或CF11证明、修改IR/AST形态或递归准入上限、降低支持深度、提高RUST_MIN_STACK、专用大栈线程/stacker/新依赖、全面拆分build.rs、所有历史native stack问题的普遍证明。

## Decisions

1. 先按实际源码和机器码确认候选最大frame来源，优先把Builder::region的Loop与Try大match arm抽成私有方法。借用现有Region和RegionPath，函数内解构原变体，原消费语句保留顺序；声明等公共前置保持原统一入口一次执行。必要时根据真实帧与默认栈结果进一步最小拆分，不凭猜测扩大。
2. 只移私有函数边界。声明、nested作用域、循环transfer解析、finally checkpoint/restore、budget charge/poll和return/Stop/Fallback的位置及调用顺序MUST保持。现有Budget/Runtime、标准库和类型足够，无库维护或license缺口；不需要第三方栈包装。
3. 原默认栈失败与8MiB诊断分别保留；8MiB只能判别正常完成，不能验收。LLDB crash前三次尝试区分初次参数错误、第二次已实测crash但无bt、第三次真实45帧bt。Region比较分支提取未改善此崩溃，撤回它的无必要变化，计费修正仍保留在CF11原变更。
4. 使用相同真实CF11 class/API/来源验收和旧循环/try/finally/预算控制，正常env无RUST_MIN_STACK覆盖。验证实际调用帧下降，再以行为和公开停止验收。full workspace/Clippy/strict/产品自身CI可以与依赖它的CF11产品同版本共同运行，但不能借旧结果。

## Risks / Trade-offs

- 抽取时丢局部绑定或提前return改变状态 → 逐句归一化对照原arm、编译及完整旧try/loop回归。
- 只缩小dispatcher仍保留过大的helper或outer frame → 保存新机器码测量并实跑默认栈；必要时继续最小抽取。
- 为测试改变结构准入掩盖崩溃 → 参数、depth、预算、原源码与来源恒同；不接受专用大栈成功。
- finally失败恢复被改变 → 原checkpoint语句原序移动，旧Stop/refusal/finally控制及fresh suite接受。

## Migration Plan

本片是CF11产品验收阻塞项，独立文档与变更文件。Luna只准备私有build.rs候选，root审查后应用并串行守卫实跑；5GiB/target1GiB一秒监控并clean。不新建branch/worktree，不关闭整CF11/71单元。完成同版本检查、自身产品CI和clean后单独勾选本片交付。

## Evidence-driven refinement (2026-10-11)

Try-only实跑后frame从202512降至190912 B，default API仍stack overflow。源码Guard arm1458行/Loop255行明显大于If92/CompoundIf96/StringSwitch142/Switch158；按原decision允许的必要最小进一步抽取，增加Loop与Guard私有helper，Try保留。原body逐句归一化恒同；Guard统一前置checkpoint创建/declare_at仍原位，只按值转入helper一次。不是Try-only默认栈已接受。新frame与行为待fresh实测。
