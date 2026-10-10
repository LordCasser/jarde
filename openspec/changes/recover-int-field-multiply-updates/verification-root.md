# 字段乘法更新：root 验收入口

当前tasks5/7，最小产品和三项永久测试已通过本地与完整类独立验收。06:48 UTC guarded build-v5九命令全部成功、资源守卫无触发，真实Java层333/compound7/名称2/Facade8/reader178/fingerprint5且忽略1。新CLI /private/tmp/jarde-field-multiply-cli-v2 SHA b518c7ae311a86ce43d9fe88492fcfbdb7d114f3b701d21ad6bec8ebc6859a59，metadata SHA f0dcf85e67539e9f91a3cd2a089536c55405482af53924e01438832b6a515db8，build execution SHA 9322060b2aba5dae02afcccd16489978a50187768822df8db7f480a77dea911c。基线HEAD准确977f，源码冻结时本产品未提交。

06:56 UTC collector-v3实际40命令（8渲染、8原样完整源码重编、8-Xverify:all运行、16fresh javap）、165闭合文件，所有raw与已接受双JDK原oracle一致。07:03 UTC独立verifier-v6实际退出0，结果results/candidate-full-family-root-acceptance-v6.json；全部物理字段/方法/flags/owner、OriginSet BCI、嵌套A来源及default/all完整正文和source-map等值接受。default read_details为not_requested且field rows空，all为complete并核准确访问呈现。待提交推送及本片自身CI独立接受；不借375dee或977f的CI。

以下按历史时间记录准备及失败过程，当前准确接受以build-v5/collector-v3/verifier-v6为准。

原形来自JADX TestFieldIncrement2，完整outer/private A类族基线由root实际执行33命令、126闭合文件；原双JDK2/2、JADX default/none四份4/4，Jarde四份生成源码在test2拒绝占位处编译失败，零候选运行。05:51 UTC独立verify-baseline-luna-v3实际接受全部成员/原类族身份/OriginSet primary+derived并集与BCI；v2 constant-pool解析失败的版本与raw保留。来源见em23-receiver-chain-next/README及results/baseline-independent-acceptance-luna-v3.json。

JADX真实路径是SimplifyVisitor.convertFieldArith，不采用audit v1的错误上下文归因。现有Jarde proof要求准确dup产物、相同字段、单消费者和封闭RHS区间；最小扩展是imul+ArithmeticOp::Multiply、AssignOp::Multiply、field_write映射与derived field read登记。保留两次独立receiver读取的显式赋值，不用结构等价表达式代替同次SSA证明；无需新pass或全局机制。

private-implementation-luna-v2.patch SHA85e0ba61688e072501c84bafe8734ecf7e3425ae81e1c656ea3264669e59a7bb已root完整读审及git apply --check，尚未应用/编译。它包含三条新永久测试：正例所有BCI/三字段access呈现，双receiver与字段/宽度/消费者反例，以及公共输出预算/取消原子停止。v1借用与名称遮蔽问题仅在私有v2修正，历史版本保留。

root实际controls旧CLI基线同样33命令、126files；原2/JADX4完整类运行均成功，Jarde四份编译拒绝。独立验收仍在准备。Runner覆盖普通值、max/min溢出、0与负数、null receiver、8/n除零、失败旧field保留以及null先于除零；不能把这份旧基线当成新候选通过。

run-validation-build-luna-v2.py已root读审相对成熟整数runner的全部delta，使用实际CI29项lint表、10产品pins、测试与真实include输入闭合、9条命令、--source-base准确HEAD核验、20GiB机器/1GiB本仓target守卫。尚未执行。候选完整类collector仍在修正原baseline schema和命令清单；交付后须root审阅并实际执行，冻结新CLI并完成独立对照才能勾2.1/2.2/3.1。

本仓target不存在，历史源/class/raw和冻结CLI全部保留。机器空闲受其它进程影响，resource-preflight-root-v1一度仅高于20GiB约11MiB，后续v2又低于阈值；不据单次读数启动长构建。若空间仍不足，保持产品未应用和干净文档检查点，待资源满足后继续；不降低守卫、不删除其他项目文件。71/612分母及EM23整单元状态不变。

候选collector v2已root完整读审v1和全部delta。root随后用真实blake3仅执行其只读baseline_inventory/validate_oracle/class_members helper，两组各126files/原2/JADX4和16+20物理methods身份/完整BCI来源均通过，记录results/collector-helper-preflight-root-v1.json；没有运行collector main或任何CLI/JDK工具，不代表新候选成功。此前agent在缺blake3时以闭合清单记录适配的静态helper检查不作为实际BLAKE3验收证据。

06:04 UTC root全文读审后实际执行verify-controls-baseline-luna-v1，退出0：126闭合files/33命令，原2/JADX4完整源码成功，旧Jarde4编译拒绝/零运行，全部物理成员/准确multiplyDivide opcode与BCI/默认-all来源和完整异常溢出raw接受。结果results/controls-baseline-independent-acceptance-luna-v1.json，真实argv/raw/hash在controls-independent-execution-root-v1.json。当前两份旧基线均独立接受，tasks仍2/7；下一步在干净文档检查点上应用已审private v2，cargo fmt后以该确切HEAD作为run-validation-build-luna-v2的--source-base，在守卫满足后实跑。成功CLI v2/meta与build须独立核pins，再执行collector v2的新8完整腿；禁止把本轮两个baseline接受当作新产品成功。

06:11 UTCroot实际git apply --check/apply已审private v2，cargo fmt --all与git diff --check退出0；4文件改动，产品仅3处闭合映射，永久测试3项。真实argv/raw/hash保存在results/apply-format-root-v1；4份应用后源码SHA见source-applied-root-v1.json。06:15 UTC本change OpenSpec strict有效。源码写入和格式化没有产生target，机器free19332894720bytes仍低20GiB，root及fuzz target均不存在；没有执行Rust编译/测试，不勾2.1/2.2。构建必须传准确977f作为--source-base。候选独立验收器和确切CI验收脚本Luna准备中，不替代root实际执行。

06:32 UTC邻项TestVariablesDefinitions2条件局部自增完整类基线root实际执行：原2/JADX4成功，旧冻结整数CLI的Jarde4份完整生成源码都因缺return不能编译、零运行。countEmpty解释为block0两branch arms不满足单join，不是i++拼写缺口；generic Signature拒绝只是无body proof的后续结果。证据在em23-variable-postfix-loop/baseline-root-v1，独立核验待执行；不混入本乘法产品或计EM23整项完成。

06:45 UTC当前乘法产品仅3生产文件+3永久测试，真实Clippy/Java层333通过，compound新预算前提修正后7/7通过；fmt/change strict/diff检查再次通过。guarded build v2失败raw保留；v3/v4预检资源拒绝均零Cargo命令，v5已root审全部窄delta、尚无validation输出或新冻结CLI。清理本仓497.2MiB后实际机器free19118002176bytes，根/fuzz target不存在。条件局部自增邻项独立v3实际接受31cmd/112files、原2/JADX4成功/Jarde4失败/零runtime与完整BCI来源。71账本新增精确缺口链接，不计EM23整体完成。

07:03 UTC真实独立验收已完成。collector-v2运行raw全一致，但错误要求default可选field明细；collector-v3仅修证据契约并完整重跑。verifier-v5因固定helper紧凑返回缺fields失败；v6先保持原helper身份/来源核验，再按已验证方法身份附接原report fields/quality，独立接受。两次失败raw分别candidate-collector-execution-root-v1及candidate-independent-execution-root-v1；成功真实执行在对应root-v2，不修改旧记录。06:53 UTC本仓cargo clean释放814.8MiB、target不存在，CLI保留。
