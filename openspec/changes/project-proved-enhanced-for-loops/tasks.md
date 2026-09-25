## 1. 固定当前基线与反例

- [x] 1.1 以当前 Jarde CLI 重放原 class/JADX/Jarde 的 `int[]`、`Object[]`、`Iterable<String>` 完整类；以 [当前证据](../../evidence/java-syntax-2026-09-24/enhanced-for-current/analysis.md)的 class/CLI 哈希、`javac --release 8` 与 `java -Xverify:all` 行数和结果验收。
- [x] 1.2 冻结索引退出后使用、长度/元素错配数组及非 `Iterable` 探针；以原 class/JADX 的 Java 8 重编运行、当前 Jarde 无增强 `for`，以及 Jarde 实际编译/来源状态为验收。[重放记录](../../evidence/java-syntax-2026-09-24/enhanced-for-algorithm/analysis.md)确认前两者 explanation-only、缺返回语句；第三者 `while`，带同输入嵌套类 classpath 重编运行输出 `nonIterable=4`，与原 class 一致。前两者可编译性债务独立追踪，不声称其 Jarde 运行等价。

## 2. 数组增强 `for` 的受限投影

- [x] 2.1 给既有语句 AST 与发射器加最小 Java 8 增强 `for` 形态，保留标签、元素类型/名称、数组表达式及来源；以独立打印单测和 `cargo test -p jarde-java` 的相关编译/测试验收。
- [x] 2.2 在既有循环及 SSA 事实之上证明索引初值/步长、数组身份、长度/读取、元素绑定、独占消费、转移和异常边；以两个数组正例及索引逃逸、错配数组、额外副作用负例的定向测试验收，拒绝时保留原有计数循环或可定位的保守引用。
- [x] 2.3 在完整证明后原子投影已构建循环与前置语句，不重复求值或漏掉元素读取；以空/null/多元素、一次调用取数组、元素调用/抛错的完整 Java 8 类重编及 `-Xverify:all` 对照验收。
- [x] 2.4 对已折叠指令补齐真实来源、预算/取消与默认/all 正文一致性；以来源 BCI、低限停止和拒绝后正文未半折叠的定向测试验收。

## 3. 数组切片独立验收

- [x] 3.1 root 重建 CLI，独立对数组正例的原/JADX/Jarde 执行 `javac --release 8`、`java -Xverify:all` 并检查值、副作用、异常类型/顺序和增强 `for` 形态；对合法负例核对原/JADX 行为、Jarde 不误投影及现有保守引用/编译状态，若产生可执行普通 Java 则另比执行；[独立验收](verification-root-array.md)记录五组 6/7/10/5/4 行与唯一 helpful-NPE 文本差异。
- [x] 3.2 root 审读局部证明、来源与数组/普通循环相邻回归，运行 `cargo fmt --check`、定向 Cargo 测试及 `openspec validate project-proved-enhanced-for-loops --strict`；[独立验收](verification-root-array.md)区分本切片绿灯与并发三参数调用的既存整仓编译债务。root 独立 target 清理 8177 文件/3.0 GiB，可用空间约 16 GiB；实施子任务亦清理其 target。

## 4. `Iterable` 后续子切片

- [x] 4.1 固定参数泛型签名的 `-g`/`-g:none` 成对 class、raw `Iterable` 加显式元素 cast、自定义非 `Iterable.iterator()` 与 `next()` 元素转换场景；[剩余边界证据](../../evidence/java-syntax-2026-09-24/iterable-raw-projection/remaining-4-1/analysis.md)与 [root 独立复核](verification-root-iterable-boundaries.md)分别记录原/JADX/Jarde 完整类的重编、执行或保守回退状态，并明确首批准入子集。Jarde 的受保护区反例当前 explanation-only，故没有其运行结果可比较；调试表正反控制见[独立证据](../../evidence/java-syntax-2026-09-24/iterable-raw-projection/jadx-iterable-for-claim/analysis.md)。
- [x] 4.2 仅对 4.1 已证实的合法类型/迭代器子集复用增强 `for` AST 投影，额外消费者及缺失泛型证据保持 `while`；以完整类 Java 8 重编、原 class 运行轨迹、拒绝反例及来源/预算测试验收。
- [x] 4.3 root 独立复核 `Iterable` 子切片和相邻循环，完成[冻结输入三方执行与严格验收](verification-root-iterable.md)；直接 `Iterable` 可合法投影，`List`／`Collection` 与自定义子接口的类型证据、跨异常区的结构恢复均作为独立后续工作。
