## 1. 精确事实与对照输入

- [x] 1.1 冻结并验收固定TestNestedLoops4完整类、同次IR诊断、JADX源码算法与两个耦合拒绝点。验收：root核已提交25命令三方基线及真实1-test的15节点/19边/40指令，区分index/BCI、内部gate源码推导与实测trace，保留原失败；不凭Java文本推断所有权。
- [x] 1.2 构造独立returnThreshold/循环边界和可观察计数的真实Java控制类，运行原/JADX/当前Jarde完整基线。验收：root双JDK编译并-Xverify:all，实际覆盖早return、正常break、跳过inner/zero迭代及计数，原class作行为真值；固定方法返回12不冒称动态覆盖return0。

## 2. 局部证明与既有消费

- [x] 2.1 Luna私有候选，root审查后复用现有normal/natural/精确边/return/transfer判据证明单入口前缀→一层child loop→准确break gateway路径。验收：真实原class中获证准确owner/端点，全部canonical入/出边和path核准，natural集合不改，不写死BCI/成员数，不新增公开IR/pass或通用walker。
- [x] 2.2 以证明结果扩展body Frame/expected并复用原Region恢复；保持child loop、terminal return、break顺序和目标。验收：实际Region owner恰等于完整expected、每个物理owner一次，gateway/return仍有来源，不以空arm或仅natural covers接受。
- [x] 2.3 扩展外层prefixed-arm接续消费该完整证明并保持原单出口路径。验收：header正常出口与已证gateway都到准确join，return离开方法，prefix完整、join/tail一次；未知额外source/exit拒绝，不任意放松closed_body或owners超集。
- [x] 2.4 加永久真实class/API与同次IR边扰动反例。验收：额外前驱/异常边、wrong gateway目标、shared return、不可约/跨path/额外exit、重复owner等明确拒绝且来源完整；IR扰动不冒称可验证class，旧prefixed/foreach/return-arm/loop-transfer控制通过。
- [x] 2.5 核新增证明和消费的预算/取消与trial回滚。验收：真实IR证明阶段实际Stop的dimension/usage/location可追溯，错误候选恢复visited等变更状态，无半正文/map/owners；公开晚Stop及预取消准确记录，不用预取消替代中途停止。

## 3. 同版本完整验收与交付

- [x] 3.1 root冻结新CLI，对固定上游及受控输入做双JDK完整原/JADX/Jarde对照。验收：生成完整类原样重编运行exit/stdout/stderr同原，实际覆盖threshold早return与计数；default/all全文/map一致，所有物理BCI/准确方法身份/UTF8 span/loop gateway来源闭合，JADX失败单列。
- [x] 3.2 执行必要库/integration、fresh workspace、同CI Clippy、fmt及OpenSpec strict。验收：5GiB free/target1GiB一秒守卫、源码完整closure前后恒同、准确命令与原失败记录保留，最终CLI绑定该完整closure并clean；旧计费pin仅按实测修正。
- [x] 3.3 提交推送实际产品，验收它自身全部CI job/step/双seed及named outcomes。验收：精确head、CI raw与本地closure/测试绑定，不能借旧产品或仅GitHub绿色状态。
- [x] 3.4 更新ledger/verification/handoff并交付干净main。验收：实际HEAD=main=origin/main、分支/worktree/target/冻结CLI核验；CF11/71整单元完成数不因该窄片改变。
