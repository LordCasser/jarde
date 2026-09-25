# 枚举常量体访问构造器：ordinal 转发反例

`Op.java` 是 Java 8 的双专属类体枚举。`javac --release 8` 生成主类私有 `(String,int)` 构造器、synthetic `(String,int,Op$1)` 访问构造器和各子类的 `(String,int)` 构造器。子类以 `null` marker 调访问构造器，后者原本将入口 name/ordinal 原样传给主构造器。[重放脚本](replay.py)只把访问构造器 Code 的 BCI 2 从 `iload_2` 改为等宽、同栈类型的 `iconst_1`。方法目标、`null` marker、常量列表及 `$VALUES` 均不变。`-g`/`-g:none` 两份改写后的 class SHA-256 及完整 `javap -v -c -p` 在 [结果摘要](summary.json)和 `results/`。

两份改写 class 均经 `java -Xverify:all` 实际执行。原 class 的 `Probe` 输出为 `ADD:1=10`、`MULTIPLY:1=21`；JADX 1.5.6 从同一 JAR 生成的完整 `Op.java` 可用 `javac --release 8` 重编且通过 verifier，但输出 `ADD:0=10`、`MULTIPLY:1=21`。差异在 `ADD.ordinal()`：JADX 将专属子类和 synthetic 访问构造器折入源级常量体后，让 Java 编译器重新给第一个常量赋 ordinal 0。`apply` 结果仍一致，故只比较覆写方法会漏掉错误。

冻结 Jarde CLI SHA-256 为 `ca04265a4f412d59c29d6bd4a26b7d9cb961f72ae13e77684831c0b9e57b5145`。其两份类源码仍保留普通字段、物理构造器和 `<clinit>`，Java 8 重编退出 1；因此本证据**不**宣称当前 Jarde 已恢复该类体，而是为后续的 [常量专属类体 OpenSpec](../../../changes/recover-proved-enum-constant-bodies/design.md) 固定一个必须拒绝的边界。JADX 的 `EnumVisitor.processEnumCls` 直接标记子类构造器不生成，`removeEnumMethods` 又跳过主构造器前两个注入参数；这些步骤可启发常量构造点到类体的映射，但不能代替对访问桥及子类调用链的逐参数证明。

执行 `python3 replay.py --jarde-cli /tmp/jarde-generic-accepted-cli` 会在自动清理的临时目录编译、改写、反编译、重编和运行两种 debug 模式，重写稳定的 `summary.json`、`results/` 与 `manifest.sha256`。Root 将整个证据目录复制到仓库外独立重放，13 个保留文件逐字节一致；`shasum -a 256 -c manifest.sha256` 全部通过。复用算法的决策是：按 `<clinit>` 中精确 constructor owner/Methodref 选定子类，沿子类构造器、synthetic 访问桥和主构造器的每条边检查入口 name/ordinal 的 SSA/Code 身份及 marker 是否纯且未被消费；任一步缺证即保留物理事实，不投影常量体。
