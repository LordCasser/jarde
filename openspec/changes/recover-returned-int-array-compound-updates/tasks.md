## 1. Root基线与架构

- [x] 1.1 root固定完整11成员ReturnedIntArrayUpdates与返回值/十轨迹Runner，两真实JDK原程序2/2、fresh JADX4/4、当前Jarde完整source compile失败0/2，独立156checks，所有原始流和失败保留。
- [x] 1.2 root核对AST/既有compound/postfix及真实dup2/dup_x2消费者，明确最小值语义表示、同一所有权与严格边界，禁止全局copy/type/pass或合成局部层。

## 2. Luna最小实现

- [ ] 2.1 Luna产出最小值语义AST/遍历/emitter和准确returned-array claim patch，复用左值/RHS证明与Budget/Stop；root先验收前片确切CI再apply，不修改仍冻结的源。
- [ ] 2.2 Luna接入canonical完整fixture的结构/真实来源/ignored执行，精准升级旧returned边界并保留merged，额外消费/复制类别/未知行/非int与预算取消控制；root实际focused验收。

## 3. Root完整验收

- [ ] 3.1 root冻结所有实际相关产品源的新CLI，新完整双JDK2腿全source空CP/SP编译、仅新classes-Xverify/runtime与原raw一致；旧nested4与旧24腿不回退，独立来源/消费/返回值核验，原JADX/原程序历史hash复用与fresh执行分开声明。
- [ ] 3.2 root实测reader/指纹/P5计费，完成fmt/MSRV/CI-exact Clippy/OpenSpec/diff与必要Java及最终完整双seed门禁；20GiB停线，原失败不覆盖，阈值变化仅凭实际计费解释。
- [ ] 3.3 root对抗复审、更新71账本/handoff、产品提交推送并验收确切CI四job/全部必要steps/新returned ignored真JDK执行，清本仓编译残留保留冻结CLI，不把窄片计为整个单元完成。
