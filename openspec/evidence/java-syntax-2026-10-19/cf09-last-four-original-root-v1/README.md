# CF09 剩余四固定原件：完整三方基线

候选产品`412245d514896b1d0eb9a8fe24388f6a70668637`，主线文档HEAD344c44dae，CLI64,631,664 bytes、SHA-256 `d29b6fee723d8f59fbecd9fd8be9f15a8ec73de3a90f5a2a29b24ff91e4af607`。四原件及关联Smali已实际git show核对，和固定JADX提交`2fb1b16386941660fda07e9017285aec40fcb37f`逐字相同。Luna并行准备，root实际执行；其中两组Java采集实际重叠运行，源码不变且无Cargo并行。root重开4896唯一path/SHA pin，四项各有独立raw审计。

| 原件 | 实际命令/raw | API | Jarde整类编译 | JADX整编/生成VM | 结论 |
| --- | ---: | ---: | --- | --- | --- |
| TestDoWhileBreak2 |46/92|8|16失败，无JardeVM|2 JDK23成功/4 VM同原|真实gap：0块被纯测试门拒绝，遗漏17/19/31 |
| TestDoWhileBreak3 |94/188|8|16成功，32 JardeVM|2 JDK23成功/4 VM同原|现有机制足够；全部36生成VM/360 CASE同原 |
| TestBreakInLoop3 |46/92|8|16失败，无JardeVM|2 JDK23成功/4 VM同原|真实gap：writeMore重复owner11 |
| TestBreakInLoop6 |12/24|非JVM输入|不伪造Java替代|Smali、Dex各成功解码|Jarde CLASS/ZIP输入边界 |

三个Java原件均完整保留outer与target，不删check、不截取单方法；8API分别覆盖debug/no-debug×class/完整classJAR×default/all，完整方法/字段库存、身份和来源有效性均核对。每原件尝试20个完整生成source编译（Jarde8格×2compiler+JADX2profile×2compiler），所有成功生成物再以双runtime运行，按同profile/runtime和原class对照。Jarde编译的是完整API target类，JADX编译其完整outer+target源码树，不能把前者说成反编译了外层JUnit。JDK8原始完整outer编译与JADX两整树编译的framework55/52失败均保留；Do2/Loop3的JDK8 Jarde源还存在其方法恢复错误，不能只用框架边界解释失败。Do3的Jarde完整target不依赖outer框架，故JDK8/23的16次编译均成功。

运行隔离为driver-only应用classpath、parentless目标loader、精确CodeSource、独立空cwd、超时与-Xverify:all。Do2/Do3/Loop3的生成wrong-CodeSource拒绝分别2/18/2次，原件另各1次。所有失败stderr保存，API default/all不改变同一输入文本、质量、身份和maps；物理BCI只能引用真实方法。原件oracle各4 VM：Do2每VM两CASE及实际check（JDK23 check pass、JDK8有JadxAssertions55/52异常），Do3每VM10 CASE且无check，Loop3每VM一CASE+fresh check均builder12135678且check在双runtime pass。Do3的迭代消费、hasNext/next有序事件、异常类别/消息/stdout及剩余值共360生成CASE一致；栈保留但不强求生成源码行号等同。其test来源10/10，Do2 fallback来源不完整，Loop3全方法quote覆盖物理BCI不等于结构化恢复。

NYI边界：Loop3源码的@Test被注释，保留@NotYetImplemented；本轮没有执行外层test43，但实际JADX1.5.6的debug/no-debug源码可重编，并在双runtime对照成功。NYI是测试启用状态，不可据它排除该真实JVM恢复缺口。

Smali/Dex边界：原Loop6.smali1216B以本机已安装smali组装为788B Dex。JADX原Smali及Dex两份完整源码都恰有1个break且无continue；原测试disableCompilation，缺少test.Test定义，本轮无JVM运行或等价Java控制替代。Jarde对原Smali和Dex的default/all共4次返回invalid_input/unknown_artifact_kind；只含classes.dex的908B ZIP两次class-source返回operation_target_not_found。最初list-classes错误附带--policy的argv失败原流保留并排除证据；补跑help及正确inventory实际为0 candidates/items[]、complete_within_schema。独立校验Dex magic/header/Adler32/SHA-1、单class/method/code item及JAR成员字节一致。v1从1开始、唯一写入受v1==2守卫限制，内层不可达只是Smali静态归纳，没有冒称运行或JVM合法性证明。项目架构暂不以DEX/APK为核心目标，不为该原件另增前端。

采集工具适配错误保留：Do2 v1的oracle变量/field tuple/schema在执行前被root拒绝；Do3 v1首次launcher因私有copy与upstream路径比较错误在任何JDK前退出，v2修正后完整执行；Loop3准备阶段曾在首次封存通知后纠正check协议，最终执行新v2，未用准备阶段结果冒充执行。Do2旧适配audit的wrongroot argv未计dependency URLs，实际独立审计按完整argv及精确依赖URL验证，没有删除隔离门。原规范程序与目标恢复错误分开记录。

[归档](evidence.tar.gz)共682 payload、29,317,861未压缩bytes，压缩3,759,645 bytes，SHA-256 `f959a64adbc073c8351f57872fb6e7721632ef7093d2f2bb0f2d12dbe5b1be06`；[库存](inventory.json)与[capture](capture.json)逐成员复开bytes/mode/SHA。成员保留所有root收据/原始流、两个正确准备版本及被拒绝版本、独立审计、原件和driver；按库存原路径反查成员前缀。IR及fixture单列于[同轮IR](../cf09-last-four-ir-root-v1/README.md)。

基线接受不等于修复接受。固定13原件此刻为10已接受（含Do3）、2已证JVM恢复gap、1Smali/Dex输入边界；两项OpenSpec仅证据完成，CF09/71/612保持partial。
