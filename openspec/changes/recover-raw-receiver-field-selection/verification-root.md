# raw receiver 字段选择：root 验收

主线基线 `564e22c1`。先读根 handoff 与已有字段/构造器验收，再冻结16族四腿、审查实际发射与发布顺序，Luna实施，root独立审查、编译和对照。最终CLI SHA256为 `3f75dab495c9753ec2c3f1bd0ac91665ed42f8b877eea65cb7c69a6879448f70`；源码与探针hash见 `results/acceptance-manifest.json`。最终门禁与远端交付状态在下方单独记录。

## 架构闭合

在同次最终Program上关联实际FieldAssign、唯一presented BCI、精确own putfield和唯一SSA load。formal依Names/physical slot/entry definition/reuse匹配；raw local还要求唯一实际raw声明、直接formal初始化、store与load来源对应，拒绝重绑定与声明歧义。折成this的原raw alias不获新许可。只在实际类头已发布且有类formal时应用raw选择，普通非泛型类的引用不等于raw type。

方法头先结清，再收集同一物理方法实际发布的参数表，最后字段按所有写入/读消费者原子提交。原字段声明Signature、class binder、擦除和物理身份门不变；raw访问点单独使用已验证字段descriptor作RHS赋值目标。Object/数组到Object、数组组件协变、primitive array与Number最左上界均有封闭规则；不猜用户类层级，不放宽旧generic helper。没有新pass、parser、IR、fixpoint或源cast。

已发射synthetic FieldWrite accessor会把callee receiver替换成caller receiver，因此单独收集同次Program最小accessor身份，撤销该setter的raw许可。此guard不依可选RuleDetails；essential/all比较一致且完整源码可编译。后续lambda/enum/array/member-family重发射已审查，不能用物理callee的formal冒充实际访问receiver。

计费覆盖SSA索引、AST节点、二次写匹配、事实存储、facade关联与方法完整性清单。collector在完整Ok后同时交付两份sidecar；Err停止且外层清空未完成事实。真实四指令setter的单测验证足额positive、重复BCI/错误receiver origin拒绝、SSA索引额度用尽后在首个AST节点BCI2停止，以及同候选取消传播。初版conditional fixture跨分支receiver不满足单消费约束，失败保留，改换fixture而未放宽生产门。

## 独立完整类对照

真Corretto8u432/OpenJDK23.0.1，各有debug/no-debug。64个冻结jar逐个核对hash，原源码重编class与冻结字节相等。原/JADX/baseline/candidate分别完整重编，javac使用真实空classpath/sourcepath目录；运行只包含各自新classes，`-Xverify:all`执行。必要helper也分别反编译，不借原helper源码或原jar。CLI非空自述头、完整命令与失败全文保留。

| 结果 | 基线Jarde | 最终Jarde | JADX |
| --- | --- | --- | --- |
| 完整编译并行为一致 | 64/64 | 64/64 | 60/64 |
| 完整字段泛型反射一致 | 12/64 | 60/64 | 60/64 |
| 方法泛型API与原类一致 | 52/64 | 52/64 | 60/64 |
| 类formal与上界反射一致 | 60/64 | 60/64 | 60/64 |
| class/field/method全部反射一致 | 8/64 | 44/64 | 60/64 |

候选恢复48个此前擦除的字段输入，所有64输入的方法API和类formal保持原样。TypedReceiver、ShadowMethodT与MultiFormalRawParam仍拒绝部分方法头；不能把恢复字段T算成恢复原方法API。继承控制RawOwnerChild没有本类字段投影，其父字段反射保持T，但Child原类formal仍未发布。InstanceRawLocal实际输出this，字段继续Object；JADX此例四腿完整编译失败，失败不被排除。

初版探针把Object实参错填成receiver，原行为比较存在盲区；root发现后保留历史并改为独立marker、准确末位RHS身份、递归数组marker及Number数值验证。当前探针SHA为 `a411376a5e0aef02c9da74de267722cd25c06692bf745c6446613be3da069489`。只有 `results/baseline`、`results/candidate` 是最终新探针对照；旧evidence历史manifest不解释为当前源。反射检查GenericDeclaration身份，不按变量名字或Method引用地址合并binder。

前片23字段族双JDK，基线/最终候选均每腿22/23完整编译并行为一致；SCGB是原有正文拒绝。20构造器族80输入，均72/80完整编译/行为、36/80完整泛型反射、48/80构造器反射，无新增回退。CallHold/ExceptionHold八输入仍是原有Object实参到已发布T callee的编译失败，普通调用适配另立片；不能说成正文拒绝。

两个Corretto8 alias压力输入保留重绑定/phi和同名分离scope的实际重复局部赋值；原Java合法，当前证明保守保持Object。完整候选源码编译检查与Rust断言覆盖这些拒绝。这些复杂alias不计入本片已恢复范围，也不代表整个泛型单元完成。

## 门禁和交付

最终CLI已完成四腿与两组旧族回归。fmt、CI同口径workspace clippy、两固定seed各3253 passed/0 failed/93 ignored、显式ignored P3 3项/constructor 1项/bound-receiver 1项全部通过。strict OpenSpec交付前重核322/322通过；完整汇总见 `results/local-gates/summary.json`。首轮全量测试因新增fixture尚未登记corpus指纹而停止；root使用既有regenerator补登记46个本片输入，独立确认1768个旧文件及其它schema/table均未变化，再重跑两seed。随后既有initializer报告等价测试抓到零formal的静态初始化器也支付了receiver扫描费用；此类方法无法提供本片raw formal/local来源，现于accessor guard之后提前跳过，10项专项复验通过，没有放宽等价断言。最终CLI与所有Java矩阵按修正后源码重新构建和重放。第三次全量扫到新增standalone `SyntheticAccessorGuard.class`，全部方法解码通过但旧fixture population断言仍是968 classes/4161 bodies；独立Luna只读核对该class恰为3个直线Code bodies、无handler/branch/subroutine后，root仅更新计数为969/4164/456/2660/8，不修改reader解析或削弱断言。对应失败日志保留为 `seed-5350648285461741569.population-failure.log`。日志和命令在 `results/local-gates`；只有实际通过后才勾选tasks。远端真正JDK25 oracle与最新main HEAD的全部CI尚待提交推送后验收，不能以本地8/23替代。

后续优先已冻结的普通泛型调用实参适配；重绑定raw alias、参数化receiver通用代换、this委派和继承选择另有边界。71验收单元总数不变。

root共享Cargo清理8105个文件、19.0GiB，target与fuzz/target均不存在，清理后可用72GiB；不触碰另一项目的target。验收二进制与基线二进制已保存在target之外。辅助worktree全部detached、干净且是main祖先，没有新增或占用分支。远端CI待实际提交推送后核对。
