## 1. 证明输入与类关系

- [x] 1.1 在冻结 `TestEnums2a` Java 8 `-g`/`-g:none` class 上记录双常量顺序、String 常量值、匿名类 owner 及构造桥字节码来源；以 `replay.py` 和 `javap` 输出复核。
- [x] 1.2 扩展既有同次 enum-body 成功计划，只接受本 proposal 指定的两个有序匿名体和 ASCII String 字面量；复用 raw MUTF-8 引用到 Java literal 的既有无损拼写能力但独立合证匿名 owner，逐边证明 name/ordinal 未变且 bridge 无附加效果；若 enum 抽象性来自接口，仅接受一个直接接口的单一 `public abstract` 方法且两个匿名体均实现它。运行关系单测覆盖准入与拒绝。
- [x] 1.3 加入 verifier-valid 的 ordinal/实参转发突变或等价真实负例，确认物理 bridge 不能被隐藏；`java -Xverify:all` 验证负例 class，运行证书测试断言拒绝投影。

## 2. 原子源码投影与边界

- [x] 2.1 由成功计划同时写出两个 String 常量实参及各自匿名体，失败时保留原物理 enum/子类来源；直接断言完整输出能表达源 enum 常量且无物理 `$` owner 构造参数，运行 class-source 投影测试。
- [x] 2.2 覆盖执行不完整、低输出预算和预取消；每种情况均断言不输出部分双常量体投影，运行预算/停止测试并检查半开来源区间。
- [x] 2.3 对错误常量数、非字面量/非 String/非 ASCII 参数、多构造链与匿名体捕获增加拒绝控制；确认非 ASCII raw MUTF-8 不发生 lossy 解码，运行 enum-body 定向测试及零参数 `Op`/`Mixed` 回归。

## 3. 三方重编验收

- [x] 3.1 固定原版、JADX、Jarde 完整源码及 `javac --release 8` 输出；分别以 `-g`、`-g:none` 重编并 `java -Xverify:all` 运行，断言常量顺序/name/ordinal/getOp/覆写行为三方一致。
- [x] 3.2 运行 enum-body、enum-constructor 和 class-source 定向回归，执行 `cargo fmt --all -- --check`、`git diff --check` 与 `openspec validate recover-proved-string-arg-enum-constant-bodies --strict`；保存稳定脚本、工具链、来源哈希与拒绝证据。
