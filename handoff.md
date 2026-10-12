# Jarde 接续入口

持续目标 active，用户要求在当前聊天继续。固定 JADX **71 语法单元 / 612 测试文件**的账本，先完成大颗粒 MVP，再探索双方未覆盖的形态。组件测试或局部方法通过不能增加整体单元完成数。

## 当前主线交付

2026-10-13：产品 `7e93eb30647338dd8a6ddfc9994392f23457aa3e` 已提交推送 main；[自身 CI 38297425449](https://github.com/LordCasser/jarde/actions/runs/38297425449) 的 4 jobs / 52 steps 全部 completed/success。root 实际捕获后独立核两 seed 各 96 批 / 364 targets / 3525 passed / 0 failed / 98 ignored，逐 target、逐具名结果及 4854 个来源与本地一致。[CI 归档](openspec/changes/recover-ordered-field-concat-operands/results/own-product-ci-7e93-root-v1/README.md)保留实际捕获、v4 verifier、接受结果和两次私有 verifier 适配失败；生产代码未因 verifier 适配而修改。

循环 catch 父项当前 **7/8**，字段拼接子项 **5/6**；只剩最终真实 HEAD 的干净主线交付审计。CI 已接受，不能继续使用旧的“CI 尚在运行”状态。本次检查点提交、推送和 clean-main 审计完成后关闭父 3.3 / 子 2.3，再对关闭文档的实际最终 HEAD 外置审计。检查点不证明未来 HEAD。

- [局部门禁](openspec/changes/recover-ordered-field-concat-operands/results/reader-census-fix-and-final-local-root-v1/README.md)：focused-v12 八命令全部 0，705 passed / 0 failed / 4 ignored，4854 来源前后恒同，fmt / 当前 CI Clippy / strict / CLI build / finally exact clean 通过。receipt SHA `887bff1cb4999f13d03e9d3e6b411a032c46ce26151b0f566ea78448a7869401`。
- [完整工作区](openspec/changes/recover-ordered-field-concat-operands/results/full-workspace-accepted-root-v9/README.md)：195 命令全 0，两 seed 合计 7050 passed / 0 failed / 196 ignored，资源守卫、默认 stack、最终 clean / target absent 已独立接受。
- [最终整类矩阵](openspec/changes/recover-ordered-field-concat-operands/results/wrap-load-regression-fix-and-final-local-root-v1/README.md)：父项 40 命令 / 12 全类运行、字段 24 命令 / 4 Jarde + 4 JADX、TWR 23 命令及 C2W 27 命令 / 8 全类编译 / 16 次运行均同原。匿名类完整物理类组仍有 super 前 capture 赋值的编译差距，准确保留 compile-failed / no-runtime，不宣称匿名类通过。

冻结 CLI：`/private/tmp/jarde-ordered-field-final-check-root-v12/candidate-cli`，62277024 bytes / 0555，SHA `c8c18f26a256027262b514455c33f24d0ae1cbcec658e3d6ab76b2c3a9c2ccfd`。当前没有 root 活动工具链，本地 target 不存在。旧的 Cargo、CI 捕获与 verifier session 均 terminal，不要再次 poll。

## 正在推进的两个关联语法点

[recover-debug-reference-lifetimes](openspec/changes/recover-debug-reference-lifetimes/tasks.md) **1/6**，planning strict 通过。当前 debug/none 双 JDK 完整类基线 50 命令 / 100 raw、11 方法、原 17 场景已[归档](openspec/changes/recover-debug-reference-lifetimes/results/baseline-product-7e93-root-v1/README.md)。JADX raw alias 丢失导致编译失败；Jarde debug 有 slot5 Integer/String 类型冲突，debug/none 另有 handled() 拒绝。整体均 no-runtime。

Luna 私有候选 v5 在 `/private/tmp/jarde-debug-reference-lifetimes-impl-luna-v5`，尚未应用共享源码、尚未运行 Rust。方案复用现有 reference_retype_split / Split.by_bci，LVT 读取范围与既有 SSA 身份一一对应，保留旧 alias / CFG / owner 门；标准 BTreeMap 排序匹配，新增工作逐项计费。真实旧值逃逸 fixture 已由 root javac 编译，v5 patch 不含二进制 fixture，应用时须单独复制真实 `.java` / `.class` 并核 pins。root 与独立 Luna 对抗审查后运行永久测试、default/all/source-map/Stop，再用新 CLI 完整类对照；handled 未恢复前不能关闭整类运行门。

[recover-protected-prefix-loop-sequences](openspec/changes/recover-protected-prefix-loop-sequences/tasks.md) **1/6**，planning strict 通过。root [同次 Walker trace](openspec/evidence/java-syntax-2026-10-13/outer-try-foreach-trace-root-v1/README.md)证明 try 只消费 prefix，返回 next=7；[同次子调用](openspec/evidence/java-syntax-2026-10-13/outer-try-foreach-child-probe-root-v1/README.md)实际在 header7 返回 ExceptionEdge(7,0)，未进入 loop body。来源 4854 全部按字节恢复，测试 1/0/0、finally clean / target absent。不能把预期的 Loop 当观察。

设计以一一绑定当前 Catches 的 private named row 事实贯通 header/body，限定 ordinary 单 range 的 prefix + 单 Loop；完整 Sequence 留在 Try.body，严格证明纯 range-end bridge 才成为兄弟 continuation。arm 继承，nested try/guard 清除或拒绝；保留 physical row、takeability、coverage、Stop/checkpoint 等旧门，不新增 pass/solver/global index。Luna `/root/reference_load_regression_fix` 正在私有 v2 实现，root 尚未应用。

[独立边界完整类基线](openspec/changes/recover-protected-prefix-loop-sequences/results/boundary-baseline-product-7e93-root-v1/README.md)真实 24 命令 / 48 raw / 20 全类编译 / 4 原始运行：iterator、hasNext、next、output.add 四边界各 RuntimeException/Error 加 success/empty 共 10 场景；原四腿逐字同，16 份 JADX/Jarde 全类编译均失败、无 runtime。这是基线，不是候选通过。两项最终代码必须共同通过原 17 场景及此 10 场景的未手修全类运行。

## 架构债务与推进边界

[reference-definition-order-audit.md](openspec/evidence/java-syntax-2026-10-13/reference-definition-order-audit.md)记录 alias gate 的 definitions 顺序可能受 CFG/SSA 发布顺序影响；目前是静态路径推断，没有实际 IR 反例或错码证明。单独核验，不把排序修复混进当前 LVT 改动。匿名类 constructor 与 DF.loopFinally 的独立差距分别保留在各 change results 和 [completion-review.md](openspec/completion-review.md)。

前一项 Structured List MVP **9/9** 已交付；自身产品 ca45 与真实最终 HEAD 9dc64e4d 的干净主线审计均已接受，详见其 [verification-root.md](openspec/changes/recover-structured-list-generic-bodies/verification-root.md)。旧产品证据不替代新产品验收。CF11 / CF18 / 71 / 612 完成数保持账本原值。

## 工具、Git 与资源

root 独占共享 Git / Cargo / rustfmt / JDK / JADX / CLI；Luna 只读或私有准备，不改共享源码、不运行工具链、不创建分支/worktree。仅 main；14 个辅助 checkout 已 detached，须持续核 clean / main ancestor / 无 target。用户已授权提交推送和主线清理。

构建最低 **5 GiB** 可用空间、target 上限 **1 GiB**、**1 秒进程组守卫**、default Rust stack，finally exact cargo clean。不得并行 root 工具链或清活动 target。ScriptOS 已结束任务的缓存/重复临时产物累计释放约 2.35 GiB，独有验收材料保留；2026-10-13 实查可用约 21.4 GiB。

下一步先完成上述主线交付审计，再审阅和应用私有候选，串行执行必要局部门与整类对照。通过后再 fresh 双 seed workspace / 精确产品提交推送 / 自身 CI / 真实最终 HEAD clean-main；不得拼接历史失败、删方法、修补生成 Java 或降低断言。
