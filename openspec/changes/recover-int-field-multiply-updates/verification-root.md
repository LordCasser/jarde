# 字段乘法更新：root 验收入口

当前仅完成基线与设计，任务2/7；没有应用产品补丁。前置整数数组产品375dee的确切CI38026598963已root独立接受，后续先提交文档/证据检查点。

原形来自JADX TestFieldIncrement2，完整outer/private A类族基线由root实际执行33命令、126闭合文件；原双JDK2/2、JADX default/none四份4/4，Jarde四份生成源码在test2拒绝占位处编译失败，零候选运行。05:51 UTC独立verify-baseline-luna-v3实际接受全部成员/原类族身份/OriginSet primary+derived并集与BCI；v2 constant-pool解析失败的版本与raw保留。来源见em23-receiver-chain-next/README及results/baseline-independent-acceptance-luna-v3.json。

JADX真实路径是SimplifyVisitor.convertFieldArith，不采用audit v1的错误上下文归因。现有Jarde proof要求准确dup产物、相同字段、单消费者和封闭RHS区间；最小扩展是imul+ArithmeticOp::Multiply、AssignOp::Multiply、field_write映射与derived field read登记。保留两次独立receiver读取的显式赋值，不用结构等价表达式代替同次SSA证明；无需新pass或全局机制。

private-implementation-luna-v2.patch SHA85e0ba61688e072501c84bafe8734ecf7e3425ae81e1c656ea3264669e59a7bb已root完整读审及git apply --check，尚未应用/编译。它包含三条新永久测试：正例所有BCI/三字段access呈现，双receiver与字段/宽度/消费者反例，以及公共输出预算/取消原子停止。v1借用与名称遮蔽问题仅在私有v2修正，历史版本保留。

root实际controls旧CLI基线同样33命令、126files；原2/JADX4完整类运行均成功，Jarde四份编译拒绝。独立验收仍在准备。Runner覆盖普通值、max/min溢出、0与负数、null receiver、8/n除零、失败旧field保留以及null先于除零；不能把这份旧基线当成新候选通过。

run-validation-build-luna-v2.py已root读审相对成熟整数runner的全部delta，使用实际CI29项lint表、10产品pins、测试与真实include输入闭合、9条命令、--source-base准确HEAD核验、20GiB机器/1GiB本仓target守卫。尚未执行。候选完整类collector仍在修正原baseline schema和命令清单；交付后须root审阅并实际执行，冻结新CLI并完成独立对照才能勾2.1/2.2/3.1。

本仓target不存在，历史源/class/raw和冻结CLI全部保留。机器空闲受其它进程影响，resource-preflight-root-v1一度仅高于20GiB约11MiB，后续v2又低于阈值；不据单次读数启动长构建。若空间仍不足，保持产品未应用和干净文档检查点，待资源满足后继续；不降低守卫、不删除其他项目文件。71/612分母及EM23整单元状态不变。

候选collector v2已root完整读审v1和全部delta。root随后用真实blake3仅执行其只读baseline_inventory/validate_oracle/class_members helper，两组各126files/原2/JADX4和16+20物理methods身份/完整BCI来源均通过，记录results/collector-helper-preflight-root-v1.json；没有运行collector main或任何CLI/JDK工具，不代表新候选成功。此前agent在缺blake3时以闭合清单记录适配的静态helper检查不作为实际BLAKE3验收证据。

06:04 UTC root全文读审后实际执行verify-controls-baseline-luna-v1，退出0：126闭合files/33命令，原2/JADX4完整源码成功，旧Jarde4编译拒绝/零运行，全部物理成员/准确multiplyDivide opcode与BCI/默认-all来源和完整异常溢出raw接受。结果results/controls-baseline-independent-acceptance-luna-v1.json，真实argv/raw/hash在controls-independent-execution-root-v1.json。当前两份旧基线均独立接受，tasks仍2/7；下一步在干净文档检查点上应用已审private v2，cargo fmt后以该确切HEAD作为run-validation-build-luna-v2的--source-base，在守卫满足后实跑。成功CLI v2/meta与build须独立核pins，再执行collector v2的新8完整腿；禁止把本轮两个baseline接受当作新产品成功。
