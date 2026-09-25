## 1. 冻结行为与有效拒绝边界

- [ ] 1.1 在 `recover-conditional-values` 的 `<clinit>` 显式赋值验收后，用当前 CLI 原样重放 `assert-core/replay.py`：记录 subject/runner/CLI/JADX 哈希、`javap` 和原/JADX/Jarde 的完整源码、javac 与 `-Xverify:all` `-ea`/`-da`/选择性启停输出；证明当前差异只是源级形态，历史 BCI 13 RED 不冒充现状。
- [ ] 1.2 冻结无消息、两条以上断言及类静态初始化有其他效果的 Java 8 正例，和 wrong-owner、非 0/1、额外读取/写入、改变异常处理器或构造器的 JVM 可验证反例；逐例保存物理 class SHA、关键 BCI/descriptor/flags、原/JADX/Jarde 重编与运行状态。无效 classfile 不计作拒绝证据。

## 2. 证明类级开关和每条断言

- [ ] 2.1 从同一次类读取的物理 Fieldref/`<clinit>` CFG/SSA 证明开关字段唯一、flags/descriptor 可由 Java 8 `assert` 重建、初始化来自当前类 `desiredAssertionStatus()` 的标准布尔取反且相对其他静态效果顺序等价；wrong-owner、非 0/1、重复写入和不匹配字段名均拒绝。定向测试核对真实 BCI、flags、拒绝原因和按需读取数。
- [ ] 2.2 用已有区域/AST/SSA 证明每条开关读取及其条件、可选消息、`AssertionError` 构造/抛出、直通/汇合与异常边完整闭合，表达式只在原路径求值一次；无消息/带消息和副作用计数正例准入，旁路入口、未知 overload 或处理器跨越反例拒绝。定向测试比较认领 BCI 与来源。
- [ ] 2.3 扫描本类所有可达开关读取与写入，只在它们被 2.1/2.2 整组覆盖时发布类级证明；另一方法额外读、第二次写或单条未证明时全部拒绝，定向测试确认不出现部分准入。

## 3. 类源码原子投影

- [ ] 3.1 在既有类级投影接缝加最小 `assert` 语句形状：成功时整组替换守卫并省去由编译器重新生成的字段及赋值，保留其余 `<clinit>` 语句、物理成员和独立 `RecoveryReport`；原类与输出类的字段反射、断言开关和异常行为定向测试逐项相同。
- [ ] 3.2 对候选、类内闭包、来源和二次发射按现有预算计费并轮询取消；低读取/IR/输出预算或取消时不发布半投影，essential/all 正文相同，库与 CLI 测试核对停止原因、物理记录和源码哈希。

## 4. 架构师独立验收

- [ ] 4.1 root 在独立目录从冻结 Java 8 源重编，对原/JADX/Jarde 完整类分别运行 `javac --release 8` 和 `java -Xverify:all` 三种断言模式；逐项比较 guard/detail 次数、消息/异常身份和合成字段反射，并复跑 1.2 每条有效反例，确认未错误写 `assert`。
- [ ] 4.2 root 审读全类闭包、来源、预算及物理/投影分离；运行相邻条件值、字段初始化、类源码、reader census/fingerprint 定向回归、`cargo fmt --all -- --check`、适用严格 Clippy 与 `openspec validate project-proved-assert-statements --strict`，只在证据满足后勾选任务。
