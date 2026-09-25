# `assert` 源级恢复审计（进行中）

`AssertProbe.java` 由 javac 23.0.1 以 `--release 8 -g` 编译，原 class SHA-256 为 `7933b18c609d21d549b1c2c5b9356ad4ebd40a811b48cbbe06af3af66437dfde`。`javap.txt` 保存完整成员与 Code；`JadxAssertProbe.java` 是 JADX 1.5.6 对原 class 的完整输出。当前样本刻意让断言条件和 detail 表达式分别增加计数，以区分关闭断言、条件通过和条件失败时的求值。

原 class 经 `java -Xverify:all -ea` 输出 `1|0;bad:-1|2|1`，经 `-da` 输出 `0|0;0|0`。JADX 输出展开为 `$assertionsDisabled || guard(value)` 与显式 `throw new AssertionError(detail(value))`，不恢复 `assert` 语法；它的完整类按 `javac --release 8` 重编后，两种断言模式的输出与原 class 一致。这里的“语义一致”只指这个受控样本和两种开关，不能代替更广泛的结构证明。

JADX 文本中的 `/* synthetic */` 只是注释：重编后的 `$assertionsDisabled` flags 是 `0x0018`（`ACC_STATIC|ACC_FINAL`），原 class 是 `0x1018`（还含 `ACC_SYNTHETIC`）。因此这份样本的运行结果虽一致，物理元数据没有重建；源码级 `assert` 若经严格证明再投影，才有机会让 javac 自行重建该合成标志。

原 class 中 `$assertionsDisabled:Z` 是 `ACC_STATIC|ACC_FINAL|ACC_SYNTHETIC`，没有 `ConstantValue`；`<clinit>` 在 BCI 0–16 以 `AssertProbe.class.desiredAssertionStatus()` 的反值写它。`check(I)String` 在 BCI 0 读该字段，BCI 3/10 依次跳过 guard 与失败分支，BCI 13–24 只在失败时创建并抛出 `AssertionError`，detail 在构造错误前求值。因而恢复 `assert guard(value) : detail(value);` 不能只在方法里替换一个 `if`：完整源码还必须证明并处理合成字段及其类初始化写入，否则会留下重复或不同的开关状态。

`AssertProbe-wrong-owner.class` 是 `patch_wrong_owner.py` 从原 class **仅改 `<clinit>` 一个 `ldc` 的常量池索引**所得，SHA-256 为 `539af1f7fba5b8e543ff67048707c2c79f0ca570dfce5d168449dca8ce3e3232`。开关字段的名字、flags、`check` Code 和所有分支不变，但状态现在来自 `StringBuilder.class.desiredAssertionStatus()`。同时指定 `-ea:AssertProbe -da:java.lang.StringBuilder` 时，原 class 输出 `1|0;bad:-1|2|1`，补丁 class 经 `-Xverify:all` 输出 `0|0;0|0`。若只凭合成字段名和 `if`/`throw` 形状就投影成 `assert`，会错误地改用当前类的开关；必须证明 `<clinit>` 的 class literal、调用目标、取反及唯一字段写入。

`JadxAssertProbe-wrong-owner.java` 保存补丁 class 的 JADX 完整输出；它仍保留显式分支，准确写出 `StringBuilder.class.desiredAssertionStatus()`。这份类源码经 `javac --release 8` 重编后，在对应 `-ea:defpackage.AssertProbe -da:java.lang.StringBuilder` 配置下经 `-Xverify:all` 输出 `0|0;0|0`，与补丁 class 一致。

Jarde 的完整类输出、重编译、运行及边界样本尚未核对；当前不能据此提出已验证的实现任务。后续须用同一原 class 对照 Jarde，尤其检查 `<clinit>` 的来源、合成字段在报告中的物理身份，以及异常和旁路分支是否完整恢复。若确认确有语法缺口，优先复用现有方法 AST/类装配侧车作有界的跨成员证明；只有它们不能表达所需事实时才考虑新机制。
