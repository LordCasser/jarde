# Root Verification — Constructor Primitive Conversion Arguments

本片2/7任务：1.1前置确切代码CI及冻结、1.2基线/周边审计完成；代码5a6264de的CI37947791151四job/48steps全部success，产品/测试/canonical fixture尚未改。实际基线对照允许在CI等待期完成，产品改动仍必须等前置CI全部成功。

## Frozen Complete Baseline

从创建起两个完整顶层类ConstructorPrimitiveConversionControls与PrimitiveLongPair，不剥离任何成员。原源码四类mark、五wrapper、Integer对照、fresh六元素Number[]、普通return new、stored-local重复参数、全部15转换及三层舍入链全部由main显式观察。两真实JDK原始输入source8/target8或release8、g:none、显式空classpath/sourcepath；全部原/反编译source独立编译，只在各自新编译目录-Xverify:all运行。

baseline-v1 manifest SHA `0afb246902e6a6b76c7ff17939313d22810dcc586eca63f2741f7e806aa5858d`，冻结CLI2 SHA78cfb53212489c017a8ac64292df2531d65300739cdc3297435d76993b2fa273，源码两份分别4e5aff3dced47b6493a3db64b8fe2998a0c9328fde22ce5c7fd745d1e03e2c1e、904ecf774d469580f0afe716d82b1c12744fdee721274fe69e31f3507cb36e60。

| profile | 完整compile | verified run exit0 | 原始双流一致 | 语义验收 |
|---|---|---|---|---|
| 原程序 | 2/2 | 2/2 | 两原腿一致 | 2/2 |
| 冻结Jarde CLI2 | 0/2 | 不运行失败source | 0/2 | 0/2 |
| JADX none | 2/2 | 2/2 | 0/2 | 0/2 |
| JADX default | 2/2 | 2/2 | 0/2 | 0/2 |

Root独立verifier核查实际文件闭集、hash/bytes、原源码复制身份、两class jar entries、两个JDK与CLI2、实际全部source参与空CP/SP编译、私有runtime目录与原始exit/stdout/stderr；全部6比较腿核验零问题，见results/baseline-root-verification-v2.json（SHA `1c200d0989599e059d73373fd921105ef7e446073931a64108694eeab2e866b6`）。初verifier错误沿用javac23 flags核对javac8候选，真实exit1及3错误永久保留于v1脚本/JSON；v2按腿重绑定，未回写baseline或失败证据。

## Actual JADX Cast Loss

完整已安装JADX四腿都删除longViaFloat的l2f/f2l和longViaDouble的l2d/d2l；输入16777217L及9007199254740993L，原程序返回16777216L及9007199254740992L，JADX返回输入值。doubleViaFloat仍保留float cast并匹配。两原class的javap BCIs10/11明确是两个转换，constructor在12；不是源码常量折叠推测。结果及57个installed jar/launcher hash、参考SimplifyVisitor/InsnDecoder/JavaInsnsRegister源码hash见results/jadx-cast-chain-audit-v1/audit.json（SHA `0c8add88e5388db1d3c473ab1ac640ae65626d798d86b8139f439e3cd2fb4b24`）。SimplifyVisitor的processCast/shadowedByOuterCast是算法审查参考，未插桩，不断言运行时精确触发分支。Jarde复用已有Cast保留每层，不照抄删除策略；compile/run0不能充当语义oracle。

## Necessary Budget Plumbing

真实ordinary Site入口verify当前创建VerifyMeter budget:None。数组seam已使用Some(budget)，不能泛化该预算证据。下一实现将单一report调用的同一Budget传入已有verify_metered/census并Result返回Stop，Refusal仍记录；Stop时丢弃局部Sites、report直接stopped，不进入region/build/materialize。必要地修普通construction verifier整个入口既有漏计，不建立feature开关、conversion预扫、长度估价或其它规则迁移。预算/取消须在实际ordinary return-new路径证明，测试unmetered入口不作预算证据。

extra-dup的真实SSA是一读两distinct ValueId，依赖集合不证明single-use，保守拒绝来自StatementFree。stored-local在BCI3 markInt一次、6 istore、7 new、11/13两load、12/14 i2l、15 constructor、18 areturn；旧CLI仍在12结构拒绝，不能把markers数组为空误算该正文成功（真实source有@bytecode）。成员/statement、一般alias、nested covariance与BigDecimal不扩围。

前置冻结结果见results/antecedent-freeze-root-v1.json：CLI/metadata及确切CI绑定，旧factory/direct四完整六源腿文件hash逐项再次匹配；factory2/2、direct0/2，物理BCI/拒绝保留。前置产品门禁已解除，下一步骤为2.1/2.2限定实现。
