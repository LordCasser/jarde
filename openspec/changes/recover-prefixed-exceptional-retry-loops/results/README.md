# 真实基线与首拒（尚未实现）

本目录只证明问题与首拒，不证明候选实现通过。原产品仍为 `cf857abaabc4fcff82689812226f9906b8d932b2`，规划起点为 clean main `4990edcbab7ac1b1ba9303c971ffdd57e87bd04c`。本片任务当前2/7，CF-11与71/612整体计数不变。

| 归档 | 实际范围 | 成员 / bytes / SHA256 |
| --- | --- | --- |
| original-baseline-root-v1 | 24命令/48raw，完整原始源、debug/no-debug、八Jarde与四JADX请求、六方法Code事实、4次原JVM/28观察行、七行oracle、错误CodeSource根负控及root API审查 | 90 / 556303 / aa8bcba5035095c38ea2da82966f725acc41faeb6042262d05f3d11bb431d763 |
| baseline-rebuild-matrix-root-v2 | 37命令/74raw，24完整原样源码重编，6成功类/12隔离运行/84case与原oracle相同，完整源/报告/编译副本及class树不变；601项审查root读脚本后重跑并另核24条compiler argv | 105 / 488804 / a460b09fc0aee3ee71920c15172f0a451a335310db2e3b6813b59f6b93d23303 |
| first-refusal-root-v1 | stderr临时构建及exact clean、八实际公开请求，完整非usage JSON等基线、实际CFG/SSA/effects及四classifier拒绝，源码完整恢复、target absent；root独立审查 | 43 / 2000891 / ac9c16c59e3d4e9d28a4c97c8bd4f636557e9c25fde46d076f9be6bf729875f9 |

所有归档含manifest，capture完成后root另逐成员核bytes/SHA重开，238成员均通过。CLI二进制未归档，精确pin在收据中；诊断CLI不能当产品。collector/checker/driver/诊断及复用guard源码快照已打包，外部JDK与依赖仍按原绝对路径及SHA收据固定，不声称已随包提供所有依赖。

原始源码为本地JADX `jadx-core/src/test/java/jadx/tests/integration/loops/TestNestedLoops3.java`，SHA256 `7152231b3aeb8419a6cfa5ecb9f639809386109233ce66af43143e0451b0ea72`。首次javac8编原外层测试类因既有IntegrationTest.class major55失败；随后由javac23 `--release 8` 编原文两profile，完整outer+target均major52，target六方法不删。目标Code42字节，不是usage累计code_bytes52；BCI40/41为iload_2/ireturn。旧诊断v1的末两opcode草稿错误由root审查发现，从未执行；v2按真实Code逐条修正核对后才构建。

Jarde八配置保留完整六方法，40个已结构化方法配置全物理BCI映射；八个test fallback各缺13个BCI，未冒称覆盖完整。default/all正文、质量、fallback及map投影相同；standalone/archive owner各自核一致，不错误等同两种物理身份。Python缺blake3模块，API审查没有独立重算BLAKE3，只核同次owner绑定、class长度及SHA固定输入；SHA256与BLAKE3未混用。

16次Jarde重编全部为test缺返回，无rebuilt runtime。JADX child四编译及bundle两次javac23成功，双JVM共84case等原oracle；bundle两次javac8只因55.0/52.0框架边界失败，无runtime，不能当JADX语义错码。成功运行CP仅rebuilt目标、独立driver和AssertJ，-Xverify:all及CodeSource根断言通过；wrong-root原负控确实失败。方法正常/异常与同实例累计均在固定七行oracle中。

实际首拒：自然环头0，唯一latch29，单successor。catch join36→6已证、irreducible空；latch-chain/effectful-dual-exit/header-tested/latch-tested均None，尚未进入Try/body。natural set确为{0,6,17,29,36}，handler36由逆前驱收集纳入，不能沿用旧四块假设。原始失败、非执行草稿与实际结果区分，候选实现和完整验收仍待后续任务。

私有原始目录：`/private/tmp/jarde-nestedloops3-baseline-root-v1-20261018`、`/private/tmp/jarde-nestedloops3-rebuild-matrix-root-v2-20261018`、`/private/tmp/jarde-nestedloops3-first-refusal-root-v1-20261018`。后续采集使用新目录，不覆盖历史结果。
