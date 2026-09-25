## 1. 冻结三方完整类与拒绝边界

- [x] 1.1 冻结 `enum-declaration/` 与 `user-static-boundary/` 的 Java 8 源码、完整 class SHA、私有成员在内的 `javap -v -c -p`、JADX/Jarde 全类文本及 runner；root 从复制目录独立重放，两组原/JADX `javac --release 8` 与 `java -Xverify:all` 输出相同，Jarde 均只在常量字段式声明处报错。
- [x] 1.2 构造 JVM 合法或受控的 name/ordinal、构造器、`$VALUES`、`values`/`valueOf` 不一致及常量前缀夹带用户效果样本；对能执行者先跑 `-Xverify:all`，记录真实 BCI、原行为和预期拒绝，验证不能按名字或 flag 删除成员。name/ordinal 两例在 `enum-declaration/refusal-boundaries/` 已由 root 复制重放；五组 baseline、辅助方法修改、构造器效果和常量前缀效果在 `enum-declaration/helper-constructor-prefix-boundaries/` 冻结，并由 root 从独立复制目录重放，`generated/summary.json` 字节相同。两组辅助方法修改均可验证执行但 JADX 编译后行为不等价；另有 `enum-values-access/` 的额外 backing-array 读取反例，原/JADX 完整类在 `-g`/`-g:none` 下均可验证执行却分别得到 B/A，证明类级恢复须检查所有物理方法对隐式成员的使用。

## 2. 有界类级证明与源码投影

- [x] 2.1 在一次 class-source prepared read/成员运行中携带恰够使用的结构化 Code/恢复语句，证明两常量字段、构造调用、隐式数组与标准辅助方法整体匹配；核对所有物理方法对拟隐藏成员的使用，定向测试验证各条件及 1.2 的拒绝，预算/取消均有界，无第二次不计费读或 Java 文本解析。[Root 独立验收](verification-root-proof.md)：Stage/Measure 正例及 1.2/额外 `$VALUES` 读取反例 8/8；类正文/公开 JSON 未变化。
- [x] 2.2 仅对已证 `<clinit>` 前缀之后只剩正常 `return`、无用户语句的类，把两个常量及源整数参数拼成 Java enum 常量列表，并把构造器声明/体中注入的 name、ordinal 和 `Enum` super 调用移除；`Stage` 完整类可编译并逐项匹配原 class/JADX 的 values、valueOf、字段和方法结果。带用户后缀的 `Measure` 在 2.3 前不得做部分投影或隐藏其 `<clinit>`。[验收记录](verification-2.2.md)
- [x] 2.3 首片仅接受 `Measure` 已冻结的单项用户后缀：前缀结束 BCI 34 后执行 `sumUnits()`、BCI 37 写入 `totalUnits:I`、BCI 40 正常返回；须将同次 `ClassInitializerCandidates` 的唯一后缀 `FieldWrite` 与 Code 身份、顺序、来源逐项对齐，再用现有表达式 emitter 发射 `static { totalUnits = sumUnits(); }`。其它后缀形状、跨边界依赖或停止一律保持整组未投影，不删用户效果；`Measure` 完整类 Java 8 重编与 `java -Xverify:all` 的值和执行次数对齐原/JADX，`Stage` 与普通类回归不变。[验收记录](verification-2.3.md)
- [x] 2.4 更新物理成员记录与类级文本投影的文档/测试：每个原始 field/method item、outcome、真实来源仍在 JSON，隐式成员不重复写入类源码；默认/完整 evidence 文本相同，预算/取消与普通类、普通字段、静态初始化回归不变。[验收记录](verification-2.4.md)
- [x] 2.5 与 `present-proved-java-structure` 的旧枚举局部场景协调：明确其 `MUST NOT` 只针对未获类级证明的单方法 `putstatic` 表达；验证两份 active delta 不再互相冲突，且未证明 enum 仍保守呈现。旧场景前提限定为单方法恢复且无完整类级证明；新规则仅在整体证明成功后投影，证明不完整时保留逐项赋值及原始成员来源。两项 active delta 均通过 `openspec validate --strict`。

## 3. 完整执行与主代理验收

- [x] 3.1 用修后完整 Engine/CLI 原样生成两组类与 runner，分别 Java 8 编译、`java -Xverify:all` 对照原/JADX 全部行；执行 1.2 边界、enum 方法/来源、普通 class/静态初始化回归，不手工修改生成正文或删去失败成员。[Root 完整对照](verification-root.md)
- [x] 3.2 root 独立审查类级证明、前缀/用户后缀归属、原始成员事实及拒绝来源，冻结重建 CLI 重放全部完整类；运行受影响 Rust/Java 测试、reader census/fingerprint、fmt、Clippy、OpenSpec strict 与磁盘核查，记录已知独立债务。[Root 验收及未闭合独立门禁](verification-root.md)
