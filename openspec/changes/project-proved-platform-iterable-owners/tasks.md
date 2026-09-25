## 1. 固定准入与拒绝基线

- [x] 1.1 记录 `List`、`Collection`、用户自定义子接口在 `-g`/`-g:none` 下的调用 owner、原/JADX/Jarde 完整类形态与 `javac --release 8`、`java -Xverify:all` 结果；以[冻结三方证据](../../evidence/java-syntax-2026-09-24/iterable-subtype-owners/analysis.md)及 root 对原 class SHA、owner 和 3 行运行结果的复核验收。
- [x] 1.2 冻结平台集合的空/null、坏元素 cast、iterator 额外消费及 handler 不一致反例；以 Java 8 原 class 可编译运行、现有 Jarde 形态及 JADX 对照记录为验收，不能把 explanation-only 当作可执行回归。

## 2. 有限平台 owner 扩展

- [x] 2.1 在现有 `iterable_for_each_candidate` 只添加 `java/util/List` 和 `java/util/Collection` 的精确调用与接收者源类型准入，复用全部 direct `Iterable` 证明与原子提交；以有/无调试信息两种 owner 均投影、自定义接口和同名非 `Iterable` 均拒绝的定向测试验收。
- [x] 2.2 保留 raw `Object` 绑定和原 cast，覆盖空/null、坏元素、副作用、异常/额外消费者、`continue`、真实来源与预算/取消；以完整 Java 8 类重编/`-Xverify:all` 对照和相邻循环测试验收。

## 3. 独立验收

- [x] 3.1 root 独立重建 CLI，并用冻结的 `List`/`Collection` 有/无调试 class 和拒绝反例执行原/JADX/Jarde 完整类编译运行；[独立验收](verification-root.md)记录三行及十一行原/Jarde逐字等价、JADX 重编、异常拒绝、调用次数、语法和 SHA。
- [x] 3.2 root 审读精确 owner 与源类型对应、共享证明和原子提交，运行定向 Cargo、相邻循环、格式、`openspec validate project-proved-platform-iterable-owners --strict` 与差异检查，并清理临时 Cargo target；[独立验收](verification-root.md)记录 18 项集成、140 项库测试与两轮 target 清理。
