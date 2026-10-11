# 外层赋值 foreach 的旧产品完整控制基线

产品993e9f8，冻结CLI8bdb868…；不包含永久候选。

baseline-v2 是 root 21:47:09–21:47:40 UTC 的真实采集：29命令/58 raw，双JDK原2/JADX4/Jarde4共10完整类运行腿，12 case stdout逐字相同（SHA25619b4371e85e777c957ea83abf6a46c014bc7171ebf3c9e1d5334503c648f1715），stderr均空。四Jarde输出正例及三拒绝控制仍while；positive保留String声明、seed、cast赋值及return，三个控制分别无cast、赋值前副作用、额外next消费。20方法profile全部物理BCI/UTF8来源完整。全部4732旧输入前后/live恒同，target不存在。这是冻结案例观察，不证明一般语义等价或新改写成功。

accept-v2由root 21:54:07–21:54:08 UTC真实执行exit0，status accepted_controlled_baseline_observation_only。8个generated编译/运行观察来自actual4 JADX+4Jarde，不能写成12个generated。v1验收器把Python symlink executable与resolve路径严格字符比较而失败，原raw保留；v2仅 executable按resolve身份核，argv[1:]完整一致。

baseline-v1保留独立类型债务：Object target="seed" 被Jarde错误收窄成String，4生成profile共8编译尝试均失败，没有Jarde runtime；原/JADX6运行腿仍同12 case。输入v3仅把该负向控制的初始化替换为Runner.seed():Object，避免当前foreach测试混入独立类型问题。不能称类型已修复；详账本 local-reference-type-narrowing-loop-debt.md。

canonical未来fixture是baseline-v2/capture/legs/javac8-original/classes/cf11/CF11OuterAssignmentTarget.class，1494B，SHA25628b92d58ce18fb08362bd39b0558de98c3c41830cc8cbc6bd60ec3a183b21956；同源JDK23是1485B/8c40426f3184d7fddf4a6d4e2f55fd4a5e379bf50640cf98368eabd04d487c58。这里只归档证据，未复制到永久fixture。

manifest按每项original/copy bytes+SHA256核准。receipts内绝对路径保留原执行值，迁移可按manifest定位对应副本；不改原始回执。
