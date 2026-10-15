# Design

## Context

动机见 proposal.md。reader 已保存物理重复声明，prepared locator 也能返回全部同名同 descriptor 记录；有 Code 的 reader 解码路径已有歧义拒绝。遗漏在 jvm driver/callee 提前取首项并对无 Code 成员返回。修复发生在成员选择层，不改变 reader 的原始解析结果，也不把选择错误当成 JVM verification 或 Java source recovery 结论。

当前共享仓库还有普通 Try/List 变更；源码、构建与 Git 操作由 coding root 串行管理。本变更先生成私有候选，应用时核验原文件摘要并重新 review。现有 jvm engine/callee 文件基线摘要分别为 `a1f7f9a8e0fd8d9faf9a8474da576bf5a95c8a32636f07e4d69f86ee37f9f6b2` 与 `32d8d8f3c76d790663c8098ae9773993da1cd3fbc05a4bf017d5b279f3d070d5`；它们仅标识候选准备基线，不代表未来验证版本。

## Goals / Non-Goals

**Goals:** 使用现有完整成员表和错误模型，在 body 准入前统一 direct/prepared driver/callee 的选择顺序；用真实物理重复成员证明顺序无关和零 body 增量。

**Non-Goals:** 不新增公开 member ordinal identity、统一 resolver 候选模型或新 reader API；不执行目标方法、不扩展反编译模式、不新增依赖或跨 crate 通用选择框架。

## Decisions

1. direct driver 的过滤迭代器只取首个和第二个匹配，第二个存在即拒绝；prepared driver 对现有 locator 结果确认唯一后取首项。扫描受既有成员表解析预算约束，不为唯一性解码 Code。替代方案是始终先调 body decoder，但这会改变无 Code 语义、消耗 body 配额，也无法覆盖 callee 的提前返回。
2. callee 的私有 locate 返回 `Result<Option<CalleeRecord>>`，在 Read/Prepared 分支检查重复；调用点传播错误后才进行 no-Code 判定。保留既有 binding 与成员表完整性检查顺序，避免用残缺证据判定歧义。替代方案是在每个调用点重复筛选，容易再次出现不一致；不引入公用泛型 trait 或框架。
3. 重用 `Error::invalid_input` 与既有 `classfile_method_ambiguous` 码和消息。若两文件需要同一构造器，可在 engine 中增加最小 `pub(crate)` 私有 helper；不新增 refusal/report 类型。driver 保持现有 Failed 报告组装，callee 保持返回 Error。
4. 使用现有 `tests/p3_prepared_input.rs` 的物理 classfile 构造及 callee/direct/prepared 测试设施，覆盖无 Code+Code 双顺序、两条无 Code、两条有 Code、唯一/缺失/不同 descriptor 控制。prepared 用量从 preparation 后取差值；歧义测试同时检查 Header、阶段状态和 body/Code/IR 增量，不能仅对比错误字符串。
5. 标准库迭代器和现有内部组件已满足能力；无维护库能替代本地成员唯一选择这一小逻辑。新增外部库只会增加依赖与许可维护成本，因此保持当前依赖和许可证边界。

## Risks / Trade-offs

- [两条有 Code 的歧义此前可能计入一次 body 尝试] → 新契约明确在选择阶段拒绝，增量改为零；测试显式记录此行为，保留 reader 解码门作为防御。
- [prepared 预读取 Header 的历史用量被误计为本次 body 工作] → 从 preparation 后记录预算差值，分别检查 Header 保留与 body 为零。
- [共享实现变动使私有 patch 过时] → 应用前核对摘要，重新读取冲突文件，不把过时测试结果当作当前实现通过。
- [测试只覆盖错误而遗漏唯一成员的行为] → 用有 Code、abstract/native、缺失及其他 descriptor/方法的重复作控制，保证选择范围准确。

## Migration Plan

独立应用源码和测试后运行相关 P1/P2/P3 与 artifact binding 回归、当前 fmt/clippy 门及 workspace 集成；保存实际命令、日志、版本与失败修复证据，再更新任务状态和独立提交。验收后按既有流程同步主规范及归档。回退以该独立实现提交为单位，保留发现记录。
