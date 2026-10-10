## 1. Root基线与架构

- [x] 1.1 root固定完整11成员ReturnedIntArrayUpdates与返回值/十轨迹Runner，两真实JDK原程序2/2、fresh JADX4/4、当前Jarde完整source compile失败0/2，独立156checks，所有原始流和失败保留。
- [x] 1.2 root核对AST/既有compound/postfix及真实dup2/dup_x2消费者，明确最小值语义表示、同一所有权与严格边界，禁止全局copy/type/pass或合成局部层。

## 2. Luna最小实现

- [x] 2.1 Luna产出最小值语义AST/遍历/emitter和准确returned-array claim patch，复用左值/RHS证明与Budget/Stop；root先验收前片确切CI再apply，不修改仍冻结的源。
- [x] 2.2 Luna接入canonical完整fixture的结构/真实来源/ignored执行，精准升级旧returned边界并保留merged，额外消费/复制类别/未知行/非int与预算取消控制；root实际focused验收。

## 3. Root完整验收

- [x] 3.1 root冻结所有实际相关产品源的新CLI，新完整双JDK2腿全source空CP/SP编译、仅新classes-Xverify/runtime与原raw一致；旧nested4与旧24腿不回退，独立来源/消费/返回值核验，原JADX/原程序历史hash复用与fresh执行分开声明。
- [x] 3.2 root实测reader/指纹/P5计费，完成fmt/MSRV/CI-exact Clippy/OpenSpec/diff与必要Java；确切产品 CI 38001720578 两个 fresh workspace seed 各 354 records/3353 passed/0 failed/97 ignored。全workspace20GiB停线；实测局部目标采用1GiB target/2GiB机器余量双守卫，原失败不覆盖。
- [x] 3.3 root对抗复审、更新71账本/handoff；产品6ddc77d5cc2a6fcf098aad40998f6d2602731c59已提交推送，确切CI四job/52steps全部成功，新returned ignored真Temurin25.0.4+7.0.LTS执行1pass。完整日志/源码身份经root verifier v2接受，本仓提交构建残留已清，冻结CLI保留；不把窄片计为整个单元完成。
