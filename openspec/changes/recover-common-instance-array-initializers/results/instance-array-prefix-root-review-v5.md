# Root 静态审查：实例数组共同前缀 v5

状态：可进入真实编译和完整类对照，尚未应用/编译/运行，tasks仍2/8。补丁SHA `6d21b5dc235c843d11d521a5b70437c9f688bea50194cc2f23ce28c61b4630aa`，基底生产与冻结静态CLI仍属41b8检查点。

root全文审阅v3，再逐项审阅v4/v5差异，并实际执行 `git apply --check .../instance-array-prefix-luna-v5.patch`，exit0；这不是Rust类型检查或行为验收。v1/v2拒绝及各准备版本保留。

主要已核条件：完整物理method/field census与唯一constructor join、一或多个direct-super ctor、全部claimed非static putfield库存、紧邻super的连续primitive一维array write、完整own field身份及声明顺序、所有非static ConstantValue拒绝。RHS强比较保留类型、数组元素顺序、完整long与float/double bits、cast、full static目标及实参，不用名字/文本软比较；未知目标、this链与不完整信息不提交。

原super实参与所有suffix/return由原AST发射保留。field初值使用原emitter fragment，ctor装配复用assert_projection_text，不另造artifact parser。v4删除假造body+空segment emission，改成仅member_texts；root已核static fold的consumer：文本投影没有emission时若需要token/declaration重拼会明确拒绝（facade当前29399附近），不退回physical body覆盖已删前缀的构造器。物理method.text/RecoveryReport/source_map保留。

预算核点：field sidecar clone计费、census one-match scans、RHS worklist、constructor深clone和输出均沿现有预算/Stop；v5删除重复且无调用的compare_literal，removed-BCI列表逐项poll/charge，最后mutate前poll。所有field/member texts先local stage，失败/Stop到提交点之前不发partial组。公开zero-budget和pre-cancel tests只覆盖early entry行为，未证明动态late-stage注入，不虚构该测试结论。

剩余实际验收：两现有positive都有clinit；无clinit双ctor+不同super实参、final双数组source/Runner已冻结并由root实际运行baseline v2，原/JADX/当前Jarde共8腿编译运行全成功。collector仅因javap8不输出成员数量摘要而误报javac8-original，失败manifest/raw保留；独立class census/source-map验收待执行，再把真实fixture接入。Rust/fmt/Clippy需20GiB机器/1GiBtarget守卫满足，后续全控制类新CLI双JDK、default/all与完整member/BCI再验。静态gate不保证这些用例通过，也不能借41b8或6fd静态片CI结项。


2026-10-10追加：no-clinit基线独立verifier v2已实际接受1053checks/0fail，原/JADX/Jarde8腿通过。v6将冻结fixture接入永久测试后，root实际dry-apply并应用；当前根Rust验收尚未全通过。Java适配5/5；已修实际语句/类型/测试match问题，正例输出断言继续核对。原private v5/v6和所有失败raw保持不变；当前产品更改与private文本已有root最小修正差异，不可将未应用版本hash当现产品hash。
