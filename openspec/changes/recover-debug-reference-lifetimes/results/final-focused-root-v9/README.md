# 联合候选最终局部门验收

root 实际 v9 流水线的9条正常命令及 finally exact cargo clean 全部退出0。独立读取20份raw、命令规格、具名结果、4860个当前来源及前后pins，接受状态为 `accepted_focused_component_only`，不代替workspace、自身CI或最终主线交付。

- Java library 410/0/0、reader library 178/0/0；相邻integration 71/0/1，完整11方法structured-list报告8/0/1，总计667 passed / 0 failed / 2 ignored。
- fmt、当前CI的完整Clippy argv（原29条既有allow未增加）、strict 360/0、fresh CLI build通过。
- 最少5GiB、target1GiB、1秒进程组守卫及默认Rust stack保持；最大记录target为811216269 bytes；finally exact clean退出0，target不存在。
- fresh CLI为62562992 bytes / 0555，SHA `fe00fa7f0edc587a4e85bcc1136bd4397738c2a9df632432a2e79a8221c8bdd6`，实际字节与已独立接受的完整17场景及10异常边界矩阵相同。外置路径不同没有被误作二进制不同；各自真实路径、bytes、SHA都单独核验。

## 实现与对抗审查

引用生命周期沿现有exact type、SSA定义/读取owner、alias存活及canonical normal-flow证明拆分后，才把一一对应的LVT名称附到Split。无读取身份仅接受唯一真实Store与下一decoded BCI严格等于row.start，且row完整位于身份范围；普通无LVT路径不增加该扫描。现有NameTable给第二个同名身份确定性`value_2`。实际原完整rawAliasEscape核Integer/String、两个循环的局部declaration owner，永久同次SSA反例保留跨后写入的旧Integer栈值、拒绝有LVT/无LVT/人工匹配row三条完整Plan路径。非邻接、跨身份row、歧义/重叠/缺失、Stop/取消均有通过的永久测试；未把另一个definitions顺序债务混入当前实现。

普通单range prefix+Loop在现有private Frame中携带一一绑定当前Catches range/type/handler/ordinal的named row事实；Loop header/body只在该试探继承同scope。复用现有Try/Sequence和物理range-end纯transfer证书，不新增pass、solver或公共结构。owner collector有界计费，不去重普通重复owner；nested Try/Guard/fallback拒绝。全部可变Walker字段失败/Stop回滚，已消耗预算不退款。

6个永久protected-prefix测试实际通过，包括真实Guard拒绝、未设scope、额外/同handler不同range/catch-all row拒绝、partial paid row扫描Stop、晚期child Stop完整WalkerState回滚和真实效果后缀反例。纯bridge证明继续核完整canonical competing/exception/call进入、单Transfer、准确exclusive range-end、无效果/保护/handler角色及唯一join；原shared competing-edge测试和相邻Switch/fragmented/retry/transparent/finally门全部通过，未放宽旧门。

新增Negative.class源于root真实JDK23 --release8 -g编译和javap（原始receipt/raw入包）。严格suffix测试首先证明同次ordinary shape、prefix[0]→header7、child[7,16]及完整visited delta，循环next26小于protected_end34；随后真实try_level拒绝续接，body仍仅prefix，与独立prefix+handler replay的全部WalkerState相等，消耗预算更多。没有把未证明candidate入口的direct-return诊断加成永久成功测试。

root独立Code census为1class/3Code/2handler/5branch/0jsr；与已修正typed alias fixture合计后reader当前1165class/5163Code/503handler/3284branch/10jsr由真实178项reader库测试验证。永久default/all/source-map和中止原子性API测试2项，以及完整11方法default/all/report/source origins测试通过。

## 归档与历史失败

`capture.tar.gz`保存v9 receipt/20raw/命令规格和脚本、13个当前生产/测试/fixture exact snapshots、真实negative及typed alias JDK材料、v8失败receipt/raw/exact source和工具适配材料。root复开69成员、逐bytes/SHA核验；`archive-review-root-v1.json`记录归档SHA。`acceptance-root-v9.json`是独立接受结果；原v9执行状态保持commands_passed_pending_independent_acceptance，不改写原receipt。

v8仅因新增测试的temporary Plan/Sites引用E0716编译失败；root保留exact source，改为明确局部绑定，不改变生产逻辑。v7 Clippy int_plus_one及更早失败在[联合完整类归档](../joint-fullclass-candidate-root-v1/README.md)保留。历史失败没有被当作通过命令拼接。

两项1.2/1.3可按当前实现、永久测试及root审查关闭。2.3仍开放：fresh双seed工作区正在运行，精确产品提交推送、自身CI、真实最终HEAD clean-main尚未接受；CF11/CF18/71/612计数不增加。
