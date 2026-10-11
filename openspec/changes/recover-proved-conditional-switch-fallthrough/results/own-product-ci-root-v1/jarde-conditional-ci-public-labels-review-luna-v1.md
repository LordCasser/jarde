# Conditional CI public-label wrapper review

审查范围：私有包装器 `/private/tmp/jarde-conditional-ci-public-labels-root-v1.py` 与未改的 `openspec/changes/recover-proved-conditional-switch-fallthrough/results/verify-conditional-ci-root-v5.py`。只阅读源码并用 Python 标准库 AST 提取两个函数做边界输入检查；没有运行 verifier 的 `main()`，没有读取凭据或尝试还原脱敏文本，也没有调用 Git、Cargo、JDK、CLI，未改 workspace。

文件身份：wrapper SHA-256 为 `d7a6bcebb703b1520332cf4c7419c50d21d919d6a0c6123162d0d4d78bbcac65`；原 v5 SHA-256 为 `bea37c7d4d37182168a7c1cbbde3ee31c94d61f6bc32667bedbdd02ce7e5b0ca`，与 wrapper 内置的断言一致。wrapper 在加载原模块前检查这个固定 hash，只覆盖内存中的 `parse_ci_workspace` 函数；原文件本身没有编辑。

## 绑定规则与原有约束

无 `***` 的公开名称原样返回，随后仍须与当前 target 的完整本地 outcome tuple 完全相等。含 `***` 时，wrapper 把星号段当作公开名中的掩码，候选仅来自同一个 product-pinned/local-live target 的 outcomes，且状态必须与该条 CI outcome 相同；候选数必须严格为一。零候选、多个候选、状态不符都会失败。绑定后再次检查名称不重复，并要求完整有序 `(name, status)` tuple 与本地 target 完全相等。

`parse_ci_workspace` 保留 seed 检查、CI Running headers 与结果 blocks 数量必须等于完整 target inventory、header target key 无遗漏/重复/新增、逐 header/block 配对、每 target 的 summary counts 精确相等。由于 v5 的 `stdout_blocks_cargo_json` 先验证 declared 数量、唯一原始 outcome、ok/FAILED/ignored 数量与 Cargo summary 一致，wrapper 没有改动这些语义；每个 seed 的 totals 和 target count 比较也仍在 v5 `main()` 中。

其余验收入口也仍由原 v5 执行：CI capture 的 4 jobs/52 steps及全部成功状态；product commit/run/head/status；固定 source/Git/live/include pins；两个 workflow seeds；完整本地 target baseline；冻结 build/replay/JDK invocation/typed baseline 参数。这些路径未被 wrapper 替换或跳过。wrapper 不编辑、重写或规范化原始 masked log；仅在 v5 成功创建的新 acceptance JSON 中追加 wrapper/v5 文件身份、逐条脱敏映射与扩展后的 acceptance_scope。原文件保留且 hash 被记入，因此结果应描述为“v5 其余检查加上 wrapper 的公开标签绑定”，不能称作原 v5 独立运行成功。

## 机械边界检查

对 AST 提取的 `bind_public_name` 和 `parse_ci_workspace` 以隔离 stub 执行了以下情形：无掩码精确名与唯一同状态掩码被接受；零候选、状态错、多个候选、重复绑定、重排 outcome 和 summary count 不同都被拒绝。重复绑定情形用人工输入的重复 baseline name 只为直接触发 wrapper 的 duplicate guard；原 v5 解析器本身已禁止 baseline target 内重复 outcome。执行结果均符合预期，未导入或调用 verifier 主流程。

## 一个运行参数边界

wrapper 在 `m.main()` 成功后用 `sys.argv.index('--acceptance')` 取得要追加元数据的路径；原 argparse 对重复 `--acceptance` 会采用最后一次出现的值，而 `index()` 会取第一次。root 的预期 invocation 只有一个该选项时路径一致；若包装器未来要承受任意命令行，建议在启动前要求 `--acceptance` 恰好出现一次。此点不放宽测试或 CI 判据，但属于防止写错 acceptance 文件的最小参数约束。

顶层 schema/status沿用 v5 的 `...-ci-product-acceptance-root-v1` / `accepted`，单独看这两个字段无法区别原 v5 与 wrapper 结果；不过同一文档包含 `ci_public_label_binding.wrapper`、`unchanged_product_verifier` 的路径/hash以及明确追加的 acceptance_scope，能说明实际验收运行采用了私有 wrapper。交接或引用结果时应引用 wrapper SHA 和该字段，不应把它呈现为原 v5 原样独立通过。

结论：我未发现标签绑定会放宽计数、target inventory、status 或结果顺序约束。这个绑定仅允许 GitHub 公开日志中出现的 `***` 通过“同 target、同状态、唯一 baseline outcome”解决名称差异；失败条件保持严格。运行参数方面建议确保 acceptance 选项唯一。此次审查本身不构成真实 verifier 或 CI 验收通过证据。
