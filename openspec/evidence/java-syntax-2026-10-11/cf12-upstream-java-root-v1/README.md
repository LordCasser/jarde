# CF12 五份真实上游 Java 测试：完整基线

本轮以本地 JADX `2fb1b16386941660fda07e9017285aec40fcb37f` 的五份原始 fixture 为输入，保留全部 target class、check() 与 Labels.Inner；没有摘出 switch 方法、修改原测试或 stub AssertJ。冻结 Jarde CLI 为 `/private/tmp/jarde-proved-if-join-cli-v1`，SHA `7b751758ee7f0b49bd61332a1a78412b36ecef898e43171ecf6af620d65cc9a6`。此目录是混合结果观察，**CF12 尚未接受**。

| 原上游 fixture / Java profile | JUnit | JADX 完整源码重编/运行 | Jarde default/all完整源码 |
| --- | --- | --- | --- |
| TestSwitch.test | 通过 | 与原 class 相同 | 两配置编译/exit0，但含明确 bytecode quote，默认字符分支效果缺失，raw不同 |
| TestSwitchNoDefault.test | 通过 | 与原 class 相同 | 两配置编译拒绝；null 首写的 String 局部被呈作 Object，println(String) 被拒 |
| TestSwitchLabels.test | 通过 | 与原 class 相同 | 两配置运行相同，常量名仍数值，Inner以独立完整class呈现 |
| TestSwitchLabels.testWithDisabledConstReplace | 通过 | 与原 class 相同 | 两配置运行相同；这是上游 replaceConsts=false 控制，不是 Jarde 对应选项 |
| TestSwitchFallThrough.test | 通过 | 与原 class 相同 | 两配置完整check()及额外case运行相同 |
| TestSwitchWithFallThroughCase.test | 因重复block警告失败 | 独立重编/check()/60组合运行与原 class 相同 | 两配置编译拒绝；条件fallthrough先触发case重叠，局部跨fallback拒绝是后果 |

## 实际执行与边界

- root按官方 Maven 坐标获取6个任务私有测试SDK（共17,708,025 bytes），核上游SHA1并保存POM、SHA256、URL和manifest；复用现有JADX CLI产品jar，未用Gradle或重新构建上游产品。SDK二进制不加入仓库或产品依赖。
- `harness-v1-compile-failure` 保存23-source缺少测试helper JadxInternalAccess的实际编译失败。阅读后受控增加该helper，24个原source和独立只读capture观察器fresh编译；不使用旧Gradle testclass。
- `harness-v2-dex` 的6/0/0是真实结果，但默认走dx，且JadxDecompiler.close()删除tempDir；JUnit NEVER不能保留输入。该结果不替代Java输入或实际class。
- `harness-v3-java` 明确TEST_INPUT_PLUGIN=java，官方JUnit Console选择五类六方法。观察器在AfterTestExecution、Harness清理前复制实际8个输入class（Labels双profile各2件）和完整JADX源码，原测试无改动。JUnit是5通过/1失败/0skip；仅FallThrough的源/反编译check各1次，其余check不虚构。
- `render-root-v1` 为8实际class × default/all共16个完整Jarde报告，加8个原class javap；24命令全exit0。CLI完成不等于完整Java恢复。8class中有6个唯一物理class字节身份，Labels重复profile不重复计单元。
- `full-replay-root-v1` 实际39命令：Runner编译1、原class运行6、生成源码编译18、编译成功后的运行14。6JADX腿均原样编译/verify/check/运行raw同；12Jarde腿中6raw相同、2raw不同、4编译失败。Runner覆盖char的11个字符串、NoDefault的9个case、Labels的10个输入对、FallThrough的check和7个额外case、条件fallthrough的check和60组合。
- 原/生成class以JDK23.0.1 `-Xverify:all`运行，fresh原输入由上游默认`-g -source1.8 -target1.8`编译；完整输出同样target8编译。这不是原生JDK8或双JDK接受。SDK/harness helper完整classpath已pin；helper目录排除switch测试target，避免原class掩盖输出失败。
- root资源门为5GiB machine free/1GiB root target/private输出上限，一秒进程组中止；无Cargo构建。源码、class、失败raw、工具和执行路径保持历史实际值；复制到仓库后不改写manifest路径。

## 明确的后续工作

1. `recover-proved-local-source-types`：复用现有DeclarationPlan.decided/SSA全写事实，恢复C producer的char局部与null/String局部；不凭LVT或调用期望类型强制cast。范围是上表两份真实完整class，其余基线结果保持。
2. 条件fallthrough：扩现有fallthrough证书与case-entry边界，只有准确join或紧邻case出口可以共享；不用循环新机制修补假re-entry，不复制case正文。需同次真实IR与反例证书后才实施。
3. 嵌套根常量名：同类字段已在报告，现有投影被member_family Refused门跳过。先核未发布family时能否复用同类投影；Inner引用外类CONST_ABC属于额外绑定范围，单类根不能伪造。上游默认Inner仍写数值case3294，明确不别名private CONST_CDE_PRIVATE。

四项具体差距归于三个工作流；常量名工作流另有跨类绑定边界。它们复用已有事实与恢复出口；目前没有证据要求新IR/Frame/pass。原71单元/612文件分母不变，历史合成IntegerSwitchAudit首片成功保留，不能据此覆盖本轮真实反例。

## 独立观察核验

root `verify-observations-root-v6.py` 实际exit0，接受记录 observation-acceptance-root-v6.json：六子目录429文件/1,860,833 bytes闭集、六SDK jar SHA256/官方sidecar SHA1、准确JDK/完整编译与运行argv、helpers排除原target且逐字节同fresh harness、八输入实例/六唯一class、16报告计算BLAKE3准确物理身份、保存源码等于report.text、19对method source_map/default-all恒同，stdout与stderr均独立比较。此接受只证明上表混合观察，不接受整个CF12。

私有v3未经执行的稿保留；root先改正javac与render的evidence参数混淆并计算真实BLAKE3、补准确完整classpath。root v4因原harness未列javap pin、v5因closed-inventory迭代dict而失败；两次真实raw保留，v6补准确javap SHA和path字段后通过。未覆盖任何原执行记录。
