# 已证明 for 回跳来源：root 验收入口

当前5/6。root已应用两文件最小修复；本地构建、八份完整生成类已独立接受，CF07新候选已实跑并独立接受，本产品自身CI已独立接受，最终干净交付待完成。用户明确授权机器5GiB余量、本仓target1GiB上限、一秒实时中止与完成清理。

生产仅修改region.rs既有implicit_tail_latch_origin和单臂join gate。已经存在的ForHeader证明需同时满足唯一自然latch、末尾Straight、真实goto/goto_w、唯一完整canonical正常边以及update_block/update_bci/更新槽一致；不新增IR、Frame、pass或扩大for识别。末尾回跳作为derived origin归属完整LoopStmt；poll仍在for分支返回前执行。永久gateway测试核三方法精确for范围、物理owner/method/全部BCI，增加真实槽位突变和非回边transfer两项反例。Emitter的LoopStmt包含闭括号与换行，root修正精确span期望，实际审查记录span-review-correction-root-v1.json。

## 本地实际验收

validation-build-root-v1在87090b3b4735693d0930f24f817d19f41cb63d98基础上冻结未提交的本片产品，12命令全部退出0，9测试组558 passed/0 failed/1 ignored。fmt、CI同范围Clippy、Java层335、gateway9、根arm_join4、double_jumps7、terminal5、effectful12、reader178、fingerprint5+1ignored、boolean3和CLI构建完成。target峰值701574616 bytes，50源码/测试/canonical pins前后恒同。root独立verify-validation-build-root-v1实际退出0，接受文件results/validation-acceptance-root-v1.json；24 raw streams及实际调用保存，不用私有脚本准备代替实跑。

冻结CLI为/private/tmp/jarde-proved-for-latch-cli-v1，0555，SHA256 d2d9773d94011a680e18d968ea1b3684036791ceb7dfa769455adf9247f0d33a。metadata results/candidate-cli-v1.json，SHA256 aeb3a081e3f05e27f64f5afea48d4fc4a2ccaa410209910eb40065436f54e54c。validation runner SHA33ec5b321c44218c2085a48749b992771a53248e2bc435e3d3d02841da41500f；既有5GiB guard模板v9 SHA51103f51323197db7663279776c80bd1eab95adbf5e98e26f9360293d82b8c33。此基础commit和未提交标志为历史构建身份，不在后续提交时改写。

root实际cargo clean释放3022 files/669.1MiB，root/fuzz target不存在且冻结CLI保留；raw/execution在results/root-clean-build-v1。

## 新CLI完整类验收

results/candidate-whole-classes-root-v1实际57命令、20完整类编译运行腿：原4/JADX8/Jarde8，双JDK8/23与Jarde default/all。八份Jarde生成类均原样重编，fresh空CP/SP/classes、-Xverify:all，只做Runner必要package适配，exit/stdout/stderr逐字等于同JDK原class。default/all完整正文及map恒同。root实际verify-candidate-root-v1退出0，results/candidate-whole-classes-acceptance-root-v1.json明确accepted/full_bci_acceptance=true，全部28物理方法profile记录完整；Plain三方法的goto20与原形goto42均已覆盖。原v3失败与observations-only接受证据保持不变。

新CF07对照results/cf07-candidate-root-v1实际29命令、10完整腿（原2/JADX4/Jarde4）已全部编译运行成功。所有正文与既有origin保留，andWhile@15/counted@30依旧derived完整while，counted@20与lastIndexOf@25为独立未覆盖物理anchor，不能宣称CF07完整。root已实际verify-cf07-candidate-root-v2退出0，119 inventory members/29 raw命令/10完整腿/全部物理成员与准确缺口独立接受，文件results/cf07-candidate-acceptance-root-v2.json为accepted_scoped_observations/full_physical_bci_coverage=false。v1实际因原Runner已有cf07包名却重复插入而拒绝，raw保持，v2遵循实际helper同包名原样保留的协议后成功；此前wrapper的runner文件名缺v只发生在私有准备稿，root执行前已纠正，并固定完整基础commit身份，历史private原稿留存。

## 前置与交付边界

上一片ca43 fresh v3只接受observations-only，Plain三方法四profile仅缺goto20，不能回写其full verifier失败。单臂确切产品ca43ac74767c4609284f13b251de9d6c349c67a4自身CI38048944005已独立接受4jobs/52steps、双seed3383/0/97；它不替代本片来源产品自身CI。本片提交推送后须捕获精确headSha并核双seed与50pins。全部验收后才关闭本片3.1/3.2/3.3及上一片3.1/3.3。71单元/612测试文件与EM23/CF07整单元计数不变。

产品提交d714a6bcc86c7dfd757f7ff70f1f86a5b46a230c已推送main；自身CI38062706229已由root独立v2实际接受，3.2已关闭。全量OpenSpec strict实际338/0，见results/openspec-strict-after-implementation-root-v1。新八份完整类full BCI及CF07 scoped对照共同关闭本片3.1；上一片3.1依赖亦关闭，其旧失败证据不改。

15:54 UTC精确产品CI接受：results/ci-product-v1/acceptance-proved-for-latch-ci-root-v2.json，4jobs/52steps全部成功、双seed各354records/3385 passed/0 failed/97 ignored，50当前/产品Git blob pins、12本地命令/24raw/558pass与冻结CLI完整绑定；JDK25/MSRV/fuzz/supply/三方完整类步骤全部通过。真实capture与verifier argv/raw见ci-capture-invocation-root-v1、ci-verification-invocation-root-v2。v1真实验收失败源于fingerprint摘要被交错stderr的下一Running行截断；v2只对该stdout block精确6名字/5ok+1ignored、唯一summary及无另一个running标记重核，未改raw或降低totals。v1脚本/失败调用保留。

当前包含新If规划的全量OpenSpec strict实际339/0，raw见results/openspec-strict-pre-delivery-root-v1；338/0是历史产品阶段记录。最终3.3等待全部文档提交推送与真实clean main审计。
