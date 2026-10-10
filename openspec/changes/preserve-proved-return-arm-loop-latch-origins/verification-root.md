# 直接 return/latch If 的循环来源：root 验收

当前 **6/7**。最小生产改动、永久边界回归、本地验证、新CLI完整源码独立对照及自身精确产品CI均已接受；最终主线交付待实际核对。71单元/612测试文件和整CF07计数不变。

## 实际证明和产品

root同次真实IR诊断v2证明Loop5末尾直接If16 joinNone，thenStraight19/return21，elseStraight22/SSA22,25，唯一natural latch22、完整唯一Normal22→5、唯一owner。v1少解一层引用的E0277与v2真实1/0/0原raw均保留，52pins恢复后cargo clean；results/diagnostic-acceptance-root-v1.json绑定该证据。

仅扩region.rs的implicit_tail_latch_origin：直接末尾无join If，两arm各单block Straight，恰一侧实际SSA末指令为return，另一侧才成为候选。复用原exact header、非不可归约自然环唯一latch、末尾物理goto/goto_w、投影唯一successor和完整canonical唯一Normal→header证书；返回block完整无出边。最后才写入既有gateway_origins，Builder继续使用完整while范围。无新IR、Frame、pass、依赖或递归尾搜索。

永久来源测试检查全部18BCI、准确owner/name/descriptor、含缩进/末尾换行的完整while span、旧condition8/16和counted20。实际字节码错误target25→28会使loop消失，旧If CLI同样缺25来源，单列wrong-target-source-gap-debt-root-v1.md；该反例只证明不凭空造while来源，不冒称直接命中helper目标门。iinc替代goto保留primary25，return21→athrow无循环来源。三份真实Java异常/嵌套/多回边IR的拒绝层级见boundary-review-root-v1.md；没有伪造Straight或冒称每个证书门均直接命中。

永久同次真实IR测试重用实际Loop body/canonical/SSA/operations：0步止于16；2步止于25的natural-latch计费；3步止于25的全canonical扫描计费；分析完成后取消止于16，充足预算Some25。公开末期预算和取消都不发布部分正文/map。gateway首轮14/1的错误预期和修正后15/0、真实IR单测两轮1/0、临时diagnostic patch失败与成功raw保留；临时修改已移除。

## 本地和完整源码

results/validation-acceptance-root-v1.json：12实际命令 **566 passed/0 failed/1 ignored**，lib337、gateway15；fmt与CI同范围29项allowlist Clippy通过。独立核24raw stream、实际测试名、52pins（17产品/8测试/27class输入），前后恒同。target峰值701764764 bytes，5GiB free/1GiB target守卫无中止。

冻结CLI `/private/tmp/jarde-return-arm-latch-cli-v1`，0555，SHA `9ae5817d25749add0709b4f156c71ce23ac0e5ad41d66ee0136742024461a28f`；metadata SHA `9913bb51cfa4acc91ed10708f94acdcedb6b4eb85dd4ea31d827febbd05ad6ef`，build SHA `d81a2918b1a55e7d910ebd59d82956c14155b6968f4bfb602e85c99d88ab70de`，runner SHA `8c1b90733cf1f3baa6f62467b4c2536f465997b4b2c23c6256de358152e3e84f`。实际source base41fe336462448eae3547cafa2a141d52e85a08e0/uncommitted标志不回写。

results/cf07-candidate-acceptance-root-v1.json：新CLI **29命令/119闭包成员/10完整类腿**（原2/JADX4/Jarde4），双JDK8/23，源码和Runner仅作既有同包适配，原样重编并以-Xverify:all运行；stdout/stderr同原。default/all正文和map一致，16方法profile全部物理BCI/owner/descriptor独立重核；唯一四profile新增lastIndexOf@25的derived完整while，所有旧If/loop来源与正文恒同。

CF07 verifier v1错误期待观察状态连字符、v2错误用历史baseline读取已接受If的三处refs，实际失败raw保留。root v3匹配真实下划线状态并显式read_ref(IF_BUNDLE,...)核长度/SHA，实际exit0；未修改产品或重放raw，未放宽全BCI/来源/span要求。第一次collector缺Python blake3在工具链前失败，第二次使用已有离线uv缓存；原raw保留。

results/old-controls-exact-acceptance-root-v1.json：新CLI **57命令/20完整类运行腿/28方法profile**，8份Jarde生成类原样重编运行；全部物理BCI完整，正文与所有source_map精确同已接受If控制，原For来源和Postfix基线也恒同。collector/verifier都实际exit0，绑定新CLI及52pins，不借用旧CLI。

## CI、交付和独立债务

本片产品 `0ae30a7c8d217522f2e5f5b954aa86c9b59356dd` 自身CI [38073837512](https://github.com/LordCasser/jarde/actions/runs/38073837512) 已root独立接受：4jobs/52steps成功，双seed各3393/0/97、354结果记录，lib337/gateway15及新增3个gateway与1个region测试全通过。验收绑定52份live/Git产品pins、冻结CLI/build/runner和前片If v4前提，见results/ci-product-v1/acceptance-proved-return-arm-loop-latch-ci-root-v1.json。capture实际exit0；verifier第一次plain Python缺blake3在验证前失败，ci-verifier-execution-root-v1原raw保留；未改脚本，使用既有离线uv缓存第二次实际exit0，见ci-verifier-execution-root-v2。不能用后续文档CI替代此产品验收。OpenSpec strict最新实际342/342通过，见results/delivery-strict-root-v2完整raw。results/clean-target-root-v1/execution.json实际cargo clean3022files/669.3MiB，target不存在，free67077578752bytes；提交后仍须实核main/origin同、全worktrees clean且无待合入分支占用。

computed-init for债务见results/computed-init-for-debt-root-v1.md：preheader Push(Int)+Store门不支持end-1的load/sub初值。本片物理来源补全不修改while为for，不代表整CF07追平。下一项从已冻结CF12真实原上游差距推进局部类型完整写集合证明，须先完成本片自身CI和clean交付。

CF12前置诊断在等待CI时临时运行：两个真实IR测试各1/0/0；全部52pins恢复后实际cargo clean341files/145.0MiB，target不存在。wrapper v1真实失败与v2成功、独立诊断验收都保留在openspec/evidence/java-syntax-2026-10-11/cf12-real-ir-diagnostics-root-v1；观察不算下一片产品实现。
