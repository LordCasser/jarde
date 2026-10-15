# class_view duplicate identity：真实公共 API 复现

结论：候选已成功编译；唯一成员和 foreign-owner 两项控制真实通过，精确负测真实触发行为断言失败。旧 `class_view` 对相同 raw name/descriptor 的两个物理成员，经 `BodyRef::Method` 首取 abstract/no-Code 记录，返回 `Performed / NotDeclared(Abstract)`，没有返回歧义候选。无需修改生产代码即可复现。

**私有 Cargo owner 已释放**：finally 严格 `argv=["cargo","clean"]`、cwd `/home/lordcasser/.codex/worktrees/review-core-fixes/jarde`，退出 0、guard_stop=null，结束时实际 target 不存在。raw 显示删除 672 files / 345.7 MiB。释放后本 agent 只读取保存证据，不再读取或操作 live target。

## 实际执行与范围

| 命令 | 实际退出码 | 真实结果 |
| --- | ---: | --- |
| rustfmt --edition 2024，仅新 test 文件 | 0 | 仅新文件格式变化 |
| cargo test --test class_view_ambiguous_identity --locked -- --skip duplicate_listed_identity_is_rejected_before_no_code_admission --nocapture | 0 | 2 passed / 0 failed / 1 filtered |
| cargo test --test class_view_ambiguous_identity --locked duplicate_listed_identity_is_rejected_before_no_code_admission -- --exact --nocapture | 101 | 0 passed / 1 failed / 2 filtered |
| finally cargo clean | 0 | target absent，明确释放 |

负测 stderr 明确有成功 `Finished test profile` 和 `Running tests/class_view_ambiguous_identity.rs`，不是编译失败。所有原始 stdout/stderr 与退出码均保存；本次没有遇到编译失败，也没有隐去失败后改期望。

唯一成员控制实际完成 abstract、native、Code 三个子场景，分别保持 NotDeclared / NotDeclared / Read；foreign owner 仍先报 `class_view_body_foreign_owner`。

重复矩阵虽然配置 9 行，但**本轮只抵达第 1 行 `abstract-code`、第 1 次 listed identity 请求**。同一物理 class 的 Name 请求先返回 Ambiguous，两候选和零 body/code charge 断言均通过，然后 Method 请求失败；测试在这里停止。其余 8 行和第二份 equal listed identity 没有运行，不能计覆盖。

实际 panic 在格式化后测试第 204 行：

```text
abstract-code: a duplicate identity must not elect a body: Performed(ClassViewReport { ... bodies: [NotDeclared { ... no_body_kind: Some(Abstract) ... }] ... })
```

完整报告原文在 `raw/duplicate-identity-negative.stderr.raw`，包含两个 MethodItem 的 index 0/1、同一 owner/name/descriptor 身份与错误 body 结果。测试没有绕过公开 API，也没有手工构造 SSA、binding plan 或报告。

## 冻结基线与源变化

唯一执行工具链的 checkout 是指定私有 worktree；启动前观察 HEAD `f01402867ffa20085e421a680a6ebbe878e63338`。验证的是既有私有组合源码，不冒称 pristine HEAD。先固定 13 个工程候选 SHA、build.rs 的既有 Stop 测试以及全量窄源码／literal include 集合，之后才复制新文件。

- build.rs 在整个过程保持 `b973da39da521dad9a71c53114c22901eaf056f63fa55823949004b2cc234bb8`，包含既有六场景 prepass/failed-bind 测试，未编辑。
- 13 个工程 pins 全部保持；既有 `tests/interface_proof_stop_repro.rs` 保持 SHA `f285e92259533d5a55c2c351436c5107b68433d92774181d7285b6ac8b83420f`。
- 3563 项原 source/include hashes 前后不变；唯一增加 `tests/class_view_ambiguous_identity.rs`，之后为 3564 项。每个命令另保存 before/after gzip pins。格式化只变化该新文件；两个 Cargo test 和 finally clean 前后源码集合完全相同。
- 没有修改任何生产文件、主 root 树、index、commit、Git 分支或 OpenSpec。

复制输入来自 `class-view-ambiguity-repro-prep/`。实际编译的格式化新文件保存在 `formatted-class_view_ambiguous_identity.rs`，SHA `9a19cf07915c56db62533c51daec7649378a7d78bd9b40a1ae5aabdeba8fc257`。导出的 `test-only.patch` 只新增该测试文件，SHA `d1f8ec3acea425749005c4e28e7d55abd4b3be366a7dea77c7b6871a8701054c`。准备阶段测试头注释“not compiled”保留为历史输入文本；本轮收据已经确认实际编译和运行，未为了改注释覆盖已测试源码。

## 收据与运行约束

`run.py` 负责固定基线、只复制测试、顺序执行和 finally 清理；`guard.py` 复用 prepass-v2/interface 窄 source/literal include 方式。构建／测试采用 stable、offline、jobs=1、incremental/dev/test debug=0、test threads=1、默认 stack，清除 target-dir／stack／Java options／CLASSPATH；每秒检查 5 GiB free / 1 GiB target，工作命令 peak 为 362464385 bytes（约 345.7 MiB），均未触发 guard。finally 为保证清理执行，明确使用 cleanup-only 放宽的资源阈值；实际 argv 仍是严格无参数 `cargo clean`。

`execution.json` 保留终态 `observed_behavior_failure_pending_independent_review`，不伪装成测试全通过。`audit.py` 是执行 agent 在释放 Cargo 后的**收据自核**，55 项检查通过，结果见 audit.json；它只读保存材料，不代替父任务的独立审计。所有原 raw、源 pins、基线和失败摘要都保留。

本交付仅为新缺陷的真实 repro，未实现 fix，未运行宽 workspace、Clippy 或完整 CI，也未扩展 partial-member-table 语义。后续独立 OpenSpec／修复不属于本轮。
