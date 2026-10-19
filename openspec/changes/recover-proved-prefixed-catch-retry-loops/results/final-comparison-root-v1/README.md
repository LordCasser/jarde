# 四组完整对照：root与独立验收

产品 `8ea98e2e11d64f02092ac200d2b596511dcf5e40`，同一冻结CLI SHA256 `ed74e005c1bc97bcdf4a15f6a27da9e9d8a1ecdf7fe576836820ed2ce323222d`。每项均覆盖debug/no-debug × class/fullJAR × default/all的8 API；完整生成源码保留后重编，双runtime parentless加载、driver-only应用classpath、CodeSource和独立空cwd控制。5341个唯一(path,SHA)由root重开。

| 完整输入 | 命令/raw | 生成编译成功/尝试 | 生成VM/CASE | 生成wrong-root |
| --- | --- | --- | --- | --- |
| 原始DoWhileBreak2，11例（含原2例） | 70/140 | 10/20 | 20/220 | 10 |
| 原始BreakInLoop3，空路径与实际check | 94/188 | 18/20 | 36/36 | 18 |
| 原始DoWhileBreak3，10例 | 94/188 | 18/20 | 36/360 | 18 |
| 独立无文件副作用PrefixedRetryControl，10例 | 105/210 | 20/20 | 40/400 | 20 |

所有可运行产物均与同profile/runtime原class一致。Do2实际check在JDK23通过，在JDK8保留相同的JADX框架major55/52错误；其10次JDK8整编失败也仅为框架边界。Loop3/Do3各仅JADX完整outer树的2次JDK8编译失败。Loop3 builder为12135678且实际check通过，NYI/@Test注释不等于当前JADX不能恢复。Do3无target check，未执行外层JUnit文本断言。原始Smali/Dex Loop6的输入边界见共同基线，不以Java控制替代或宣称VM观察。

独立控制JADX默认包重命名为defpackage的旧32-Jarde-VM回执不算完整三方；增加官方`--rename-flags none`后保留合法原默认包及完整原样输出，补齐20 eligible编译、40VM和20wrong-root。source/driver/caseplan不变。NPE/CCE诊断文本与栈原样保留，语义投影比较返回、异常类型、事件、状态、stdout；显式注入的IllegalStateException消息亦精确比较，不强求不同JDK诊断拼写相同。

映射有效性、身份与default/all稳定均通过；不宣称全BCI覆盖：Do2 ctor3/3、test15/15、check45/45；Do3 ctor3/3、test10/10；Loop3 writeMore48/49缺88，其余8/8、6/6、10/10；控制run40/41缺71，其余8/8、38/38、46/46。两处缺口独立解析为3字节goto，保留分母，另列债务；不推断与旧Loop5 BCI103同根因。原件空路径不覆盖File.exists真分支，控制的两臂不冒充该文件路径动态覆盖。

归档保留全部实际收集失败、审计适配失败、诊断和已拒绝版本。Luna v6脚本首次执行因并行native重新创建target而失败，该回执仍为FAILED；root在native清理后原样重跑v6，真实argv/exit0与root v3 review一起接受，不把Luna旧失败产物改称通过。根聚合接受和独立归档/物理映射审计均有原流。1522成员逐bytes/hash/mode重开，tar SHA256 `b197c34f9a4291d1af3a7288bede6a5c8d0e9f8064912ecb06cf9fed17722b7f`。

[本机完整workspace](../native-accepted-root-v1/README.md)独立于本包；本产品自身CI及最终交付另行闭合，不以本包替代。
