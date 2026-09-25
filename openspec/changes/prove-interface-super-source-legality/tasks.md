## 1. 冻结正反例及架构入口

- [x] 1.1 独立重放 `redundant-interface-super/replay.py`、`abstract-interface-super/replay.py` 与 `superclass-redundant-interface-super/replay.py`，核对 Java 8 `-g`/`-g:none` 原/Jarde 的双接口 11/22/33、继承 default 3、自声明 default 4、JADX 错值/不可重编，以及三个 patched 负例的 JVM/javac 结果和 SHA 清单。
- [x] 1.2 核对 `preserve-special-call-dispatch` 的特殊调用/生产者回退入口和 facade 选定定义读取链，写出本变更触发/不触发依赖读取的定向测试；确认 method-only 与 class-source 用同一证明，不扩旧 change 3.2。

## 2. 有界接口非冗余证明

- [x] 2.1 仅在实际 `invokespecial InterfaceMethodref` 候选上按环境选定 owner、其它直接接口及完整直接父类链，沿所遇接口继承边有界证明 owner 非冗余；用 Child→Parent、父类已实现 owner、无关 Left/Right、`java/lang/Object` 固定根、其它父类缺失/歧义定义、传递继承及循环/预算/取消控制测试证明未知不会当通过。
- [x] 2.2 沿同一选定接口闭包证明精确 `name+descriptor` 的唯一可访问 default 与无未证重载绑定；用自声明 default、继承 default、目标及中间父接口的抽象覆盖、多个父 default、同名不同 descriptor 和不完整成员表控制确认正确接受/拒绝。
- [x] 2.3 将已证可写的精确调用目标作为窄输入交给恢复层，在 `special_receiver` 保留既有 this/private/class-super 门并要求接口目标证明；用定向测试确认冗余 `Parent.super` 与抽象 `Child.super` 拒绝、BCI 及延期实参生产者保留，且无环境的接口恢复保守拒绝。

## 3. Java 8 三方及接口行为闭环

- [x] 3.1 对无关双接口、唯一直接接口的自声明/继承 default，以及接口 default 调用直接父接口 default 的 fixture 的新 Jarde 完整 class-source 与 method-only，在 `-g`/`-g:none` 下核对精确 `I.super`、Java 8 重编和 11/22/33、4、3、4 执行；class-super、private 旧回归也通过。
- [x] 3.2 对 patched 直接接口冗余、父类链冗余与抽象覆盖样例核对不发布非法 `I.super`，报告含方法/BCI 缺口；在缺少继承或目标定义、重载歧义和紧预算/取消条件下核对拒绝或执行停止，不把运行时验证通过误报成完整源码恢复。

## 4. Root 独立验收

- [x] 4.1 重跑相关 reader/facade/jarde-java 回归、冻结证据和 Java 8 编译执行，检查来源、预算/取消、essential/all/range 不改变决定；执行 fmt 与可归因门禁、`openspec validate --strict`，记录 `verification-root.md`，清理私有 Cargo target，仅勾选实测完成项。
