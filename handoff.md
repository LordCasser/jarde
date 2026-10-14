# Jarde 接续入口

持续目标 active，在当前聊天继续。按本地 JADX **71 语法单元 / 612 测试文件**账本先完成大颗粒 MVP，再探索双方未覆盖的形态。局部通过不增加整体单元完成数。

## 2026-10-14 当前进行中的交付

普通 try List 泛型及其整数前向条件栈 carry 已实现。修复版冻结 CLI SHA `0a30cf2cef1974006105e141789d5a9a8116497fae12d20479b2644d3a0ad2d7`，59537768 bytes；focused-v14 为629/0/2，fmt、CI exact Clippy、CI OpenSpec1.11 strict、420 Java库/178 reader库及原ICN控制均通过。v13/v14 CLI逐字相同，仅cfg(test) fixture population tuple/三行注释变化，生产reader前缀逐字相同。

持久证据根 `/home/lordcasser/.cache/jarde-validation/ordinary-try-list-20261014`：matrix-v3 root主审计accepted/errors0，16完整Jarde cells及45/33行oracle；JADX baseline8编译失败、admission8成功保留。forward-order-v3/root-audit-v2接受四完整16方法cells/64maps/12行时序；BI三输入/default-all旧v6与v13正文/quality/map/身份不变；ICN两编译器/default-all旧v6与v13整类正文不变。完整fixture/Runner未手改；不增加CF11/CF18/71/612总体完成计数。

fresh workspace-v5/v6的真实ICN回归与旧fixture计数失败已保留。workspace-v7进程在17批后消失，未完成收据不接受；独立中断审计及actual cargo clean/target absent已记录。当前fresh双seed为workspace-forward-v8，由用户级服务 `jarde-forward-validation-v8.service` 管理；必须核服务真实PID、terminal收据、两个seed各365targets/96batches及独立root acceptance。其通过后脚本才串行执行六个CI ignored runtime命令、两个examples及tree/feature/closure检查。

两change的最后交付门仍开放：提交/push确切产品、接受自身实际CI工具链、逐成员复开最终归档及源码身份、更新完成审查和核真实最终HEAD clean-main。不得以局部矩阵或service运行状态代替完成。四项cache/no-Code/emitter/canonical债务独立0/4；私有worktree试验不冒充主线接受。

## 上一批主线（历史）

2026-10-13：产品 **597010a8cb9992eead8276b2f2a67186ada393d9** 已提交推送 main；[自身 CI 38321041720](https://github.com/LordCasser/jarde/actions/runs/38321041720) 已由 root 独立接受，4 jobs / 52 steps 全成功。两 seed 各 365 targets / 96 batches / 3547 passed / 0 failed / 98 ignored，4860 来源与本地及产品一致。[CI 与主线检查点归档](openspec/changes/recover-debug-reference-lifetimes/results/own-product-ci-597010-and-clean-checkpoint-root-v1/README.md)的30成员已由root逐成员复开，18捕获raw/gzip流恒同；检查点43命令只有bytes/SHA记录，不声称保留raw。

已接受 clean-main 检查点 **bcf426e5ef4d2c49c4f1d519c5ef974393b6bf98**：仅 main，15 worktrees，其中14辅助checkout detached、clean、main ancestor，无target，4860来源字节及mode恒同。关闭文档提交后root再审计实际最终HEAD，结果固定外置 `/private/tmp/jarde-reference-prefix-clean-main-final-root-v1/execution.json`；接续必须核该结果状态及head，不以检查点证明未来提交。

## 本批已验收

[引用生命周期](openspec/changes/recover-debug-reference-lifetimes/tasks.md)与[普通try前缀循环](openspec/changes/recover-protected-prefix-loop-sequences/tasks.md)均 **6/6**。前者把LVT范围绑定实际引用身份，保留alias/CFG门；后者复用精确named row贯通prefix与foreach异常归属，完整Sequence留在Try.body，只有已证range-end纯bridge成为兄弟continuation。未新增solver/pass或公共carrier。

- [联合整类证据](openspec/changes/recover-debug-reference-lifetimes/results/joint-fullclass-candidate-root-v1/README.md)：双JDK/debug-none/default-all，8条Jarde整类运行各17场景同原；另8条运行各10异常边界场景同原，390成员/164raw已复开。JADX16条整类编译失败如实记no-runtime。
- [最终focused-v11](openspec/changes/recover-debug-reference-lifetimes/results/final-focused-root-v11/README.md)：680/0/3，14具名门，fmt/当前CI Clippy/strict360/fresh CLI/finally exact clean均通过，4860来源不变。
- [fresh完整工作区](openspec/changes/recover-debug-reference-lifetimes/results/full-workspace-accepted-root-v3/README.md)：195命令/390raw，两seed合计7094/0/196，默认stack及资源守卫通过，finally exact clean、target absent。

冻结产品CLI `/private/tmp/jarde-reference-prefix-final-focused-run-root-v11/candidate-cli`，62562992B / 0555 / SHA `fe00fa7f0edc587a4e85bcc1136bd4397738c2a9df632432a2e79a8221c8bdd6`。

旧workspace-v1因过时同名LVT失败要求终止，v2因4个新夹具未登记指纹终止；原失败分别保留在[同名对照与失败归档](openspec/changes/recover-debug-reference-lifetimes/results/same-name-fullclass-and-workspace-failure-root-v1/README.md)和[指纹失败归档](openspec/changes/recover-debug-reference-lifetimes/results/workspace-v2-fingerprint-failure-root-v1/README.md)。这些失败不拼接为成功。上一批循环catch8/8、字段拼接6/6、Structured List9/9的历史交付见[完成审查](openspec/completion-review.md)，旧证据不替代新产品验收。CF11 / CF18 / 71 / 612 计数保持账本原值。

## 上一批规划（历史）：普通 try 内 List 泛型

[recover-ordinary-try-list-generic-bodies](openspec/changes/recover-ordinary-try-list-generic-bodies/tasks.md) **1/6**，spec-driven ready，root已读全部apply上下文。复用同次Try/canonical/code的精确row与owner闭合，打开现有完整void用途扫描的handler交集，逐指令核Code/operands/SSA/effects/Operations，并包含catch用途。统一处理现有精确E，至少String/Integer分支与null、remove(Object)前缀、nestedforeach；unknowncall/rawalias/wildcard/重赋值/indexOverload继续按既有证明拒绝。不要只删handler早拒或加通用solver。

[44行完整基线](openspec/changes/recover-ordinary-try-list-generic-bodies/results/baseline-product-597010-root-v1/README.md)已接受：17物理方法、双JDK/debug-none各27命令/54raw，8条Jarde整类运行同原，8条JADX编译失败no-runtime。[45行补充基线](openspec/changes/recover-ordinary-try-list-generic-bodies/results/baseline-45-supplement-product-597010-root-v1/README.md)已root执行auditor-v4接受并逐成员复开251成员/118raw绑定；旧44行逐字作为前缀，新增可变output的handled正常完成，effects=2。五目标方法当前仍raw泛型，不能把基线通过算作新实现完成。

Luna正在私有目录准备report/region候选与永久正反测试，另一Luna准备公开integration/改名/catch用途fixture。root审读后才应用共享源码并串行编译。实际候选须重跑至少45行完整类、异常注入、default/all、Stop和最终交付门。[JADX算法与架构分析](openspec/evidence/java-syntax-2026-10-13/try-list-signature-next-mvp-analysis-root-v1/README.md)供参考；按原class契约验收，不照搬JADX rawalias错码。

## 边界与资源

root独占共享Git/Cargo/rustfmt/JDK/JADX/CLI；Luna私有准备或只读，不创建分支/worktree。用户授权提交推送、主线清理和持续推进。

构建前重新核可用 **5 GiB**、target上限 **1 GiB**、**1秒进程组守卫**、默认Rust stack，finally exact cargo clean。不得并行root工具链或清活动target。ScriptOS此前已结束产物累计释放2.35GiB，独有验收材料保留；该数字属于上一批主机；当前每次持久receipt记录实际free_bytes，不能沿用历史磁盘余量。其他任务仍使用磁盘。

[引用definition-order债务](openspec/evidence/java-syntax-2026-10-13/reference-definition-order-audit.md)仅静态推断，尚无实际错码反例；匿名类super前capture编译差距、DF.loopFinally及纯goto文本映射缺口独立保留，不混入当前List改动。证据质量须区分source、物理身份、结构恢复、声明投影与完整运行；禁止删方法、手修生成Java或降低断言。
