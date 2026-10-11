# 验收记录

## 已接受的独立边界（4/6）

默认失败、8MiB仅诊断和真实LLDB/机器码frame见results/default-stack-diagnosis-root-v1；Try-only继续失败见results/try-only-default-stack-probe-root-v1。Root逐句归一化核Try、Loop、Guard移动的原body恒同，公共declare_at/checkpoint顺序保持；Region比较分支抽取撤回。最终源与private base/candidate pins见两轮private-extraction-review。

同版本focused-v7前7命令均exit0、无guard Stop，4707-source closure格式后到clean恒同，RUST_MIN_STACK实际None。真实Reader proof/consumer/row反例1 passed，新默认线程API3 passed；旧6循环targets36 passed、4 try/finally targets14 passed、Java recovery/gateway69 passed/1 ignored；完整Java库353 passed。含重复unit的实际统计476-0-1不是去重产品总数。结果results/default-stack-acceptance-root-v1/root-acceptance.json和全raw。

新ARM64 debug machine-code单帧：region 0xa000+0xf80+0x20=44960 B（约43.9KiB），原202512 B；loop helper20976 B、guard125600 B、try11632 B。Frame在当前真实API已通过的二进制上测，binary因后续CLI脚本错误被runner命名failed-api，不能理解为API失败。仅当前构建单帧，非跨平台上界或所有深度证明。

新增预算和停止：proof limit3/usage3@23；consumer full12332、limit12331/usage12326@31，4 mutable fields复原；public late source17858/limit17857/usage17857，Partial AnalysisSteps，precancel零usage Cancelled。公开预取消不冒称中途异步取消。

## 待交付

focused-v7第8命令错误使用不存在的bin target jarde，CLI无产出；原101与clean0保留，target不存在。正确目标jarde-cli单独同closure重建，不能借此错误遮掩或重复发布测试。新CLI正确构建同4707 closure并经root独立双JDK三方完整类接受：固定和十case控制各29命令，原样重编-Xverify:all输出/计数/全物理来源/default-all相等，详CF11 results/postfix-root-v1。fresh workspace正在执行，Clippy/strict及自身产品CI/main clean仍待；3.1/3.2尚未全完成。

## Fresh workspace 第二轮已完成，lint待接受

首轮command92 Reader census旧tuple失败完整保留于CF11 results/full-workspace-failed-root-v1。仅cfg(test)测量tuple/comment按两class独立javap修正（+2class/+5Code/+30branch，无handler/jsr）；Reader生产语义和Builder源码不变。新CLI v9以4707当前closure重建冻结，字节SHA与实际三方v8完全相同；focused整体closure因Reader测试不同，不冒称完全同字节。

Fresh v2从头95批/360实际metadata targets，3455 passed/0 failed/97 ignored，source pins前后相等、guard未触发、clean0/target absent。目前仅完成workspace实际执行，独立raw复核/CI等价Clippy/strict及本产品自身CI/clean仍待，因此3.1/3.2尚未全勾选。

## 最终源码检查进展

最终lint v4的fmt/CI全范围Clippy/global strict351/0/clean均实际exit0；最终CLI v10已重新构建冻结且用其字节实际完成固定/控制各29条双JDK完整三方，root独立checker接受。与focused-v7的Reader census及region两等价Clippy差异保留精确逆变换；最终fresh workspace v3仍在执行，不能借修正前workspace v2通过关闭3.1。共享证据在CF11 results/final-local-evidence-root-v1。

最终fresh workspace v3已root独立接受95批/360目标/3455-0-97，完整4707 inputs同最终CLI/lint，target峰值814346343bytes且clean后不存在；实际raw/hash/动态exact-once目标核验均通过。共享证据CF11 results/final-workspace-root-v3。任务3.1接受，本片5/6；自身产品CI和干净主线3.2仍待。

## 3.2 产品自身 CI 与主线（6/6）

共用实际产品 b155877ee9a3b1894b053e70ffbf1cbf0a9512fa 自身CI38144212651：root独立验收4 jobs/52 steps，双seed各360目标/3455-0-97，named outcomes与fresh workspace相同，完整4707 inputs绑定产品Git/live/最终CLI。共享全部原raw及真实调用见../recover-proved-nested-loop-side-exits/results/own-product-ci-clean-root-v1。本债务6/6，只接受保持预算、作用域与执行顺序的Try/Loop/Guard私有frame抽取；不代表跨平台统一栈上界，也不关闭CF11/71。

实际产品clean checkpoint已接受main=origin/main、仅main、15checkout干净且14辅助detached/main祖先、无target。关闭文档推送后的实际新HEAD另在outside-repo执行final audit；不得将产品checkpoint写成尚未发生的文档HEAD审计。
