# 窄整数数组写入 fixture

`v8/NarrowArrayStores.class` 是唯一永久 class 输入。旁边的 Java 文件是生成源码、source-only helper 或 runner；布尔与求值顺序边界 class 均在重放时生成到临时目录，不进入永久 reader census。重放脚本退出时自动清理临时 class、JADX/Jarde 源码与编译目录。

## 生成和验证主类

生成器为 `javac 23.0.1`，输入是 Java 8 源码。核心命令：

```sh
python3 tests/fixtures/p3-narrow-array-stores/run_fixture.py
```

脚本先运行 `javac --release 8 -g:none` 编译 `NarrowArrayStores.java` 与
`NarrowArrayStoreEffects.java`，再运行 `patch_array_stores.py`。它也接受可选的冻结 CLI，重放
完整恢复源码、javac 和 JVM 阶段：

```sh
python3 tests/fixtures/p3-narrow-array-stores/run_fixture.py \
  --jarde-cli /path/to/frozen/jarde-cli
```

生成的原 class 为 Java 8 major 52、697 字节，SHA-256
`3a720dffd2907a2d027280a89568368d5e4e14f8b138fc37f3b0a6972aebdb80`；永久 patched class
为 52.0、760 字节，SHA-256
`a3464f4b62da257e4cd4ff70970a05360475e51502a938c675392b334b359803`。主类含 10 个
`Code` 方法：构造器、6 个 B/C/S 核心 store 和 3 个同型局部/常量对照。核心 patch 只为 6 个
方法增加窄数组 descriptor，并将该方法唯一的 `iastore` 改成匹配的 `bastore`、`castore`
或 `sastore`。Code 长度、max stack、max locals 不变。逐方法 CP/Code 偏移和 patch 前后 Code
hash 由脚本的 JSON 报告现算；不得手改冻结 class。

source-only `NarrowArrayStoresRunner` 覆盖负数、边界外 int、极值、直接 store、一次调用的
int producer、producer 异常、null/OOB 以及普通窄局部/常量控制。原 source class 和 patched
class 都在 `java -Xverify:all` 下执行；每边输出 147 行，分别涵盖同一组调用，前者是源码编译
输入，后者才是 B/C/S store 指令的语义 oracle。

## 求值顺序与边界

`NarrowArrayStoreOrder.java` 和 `NarrowArrayStoreOrderRunner.java` 是 source-only 的 Java 8
赋值顺序控制。八个断言逐项覆盖数组、下标和值生产者成功或抛错，null/OOB 与 RHS 抛错优先级、
三方各执行一次以及失败时数组原值。它以原始 Java source 形状运行，当前尚未把有副作用的
数组表达式和下标表达式编码进六个 patched B/C/S 方法，因此这八项只固定 Java/JVM 顺序 oracle，
不算 recovered B/C/S 的对照验收；主类中真实 patched store 与 Jarde 完整类阶段由上述可选 CLI
重放并独立记录。

同一 `bastore` 的 verifier-valid Z 边界在临时副本上生成：仅将主类 `storeByte` descriptor
从 `([BII)V` 换为 `([ZII)V`，保留 `bastore` 和五字节 Code。source-only runner 对 17 个负值、
奇偶和边界值执行 `java -Xverify:all`，验证 JVM 的低位 boolean 语义。该变体已证明数组是
`boolean[]`，不等同于“恢复端无法判定 B/Z”的未知元素情况。

另一个 source-only 变体 `NarrowArrayStoreBooleanOperand.java` 直接以 boolean 操作数写入
`boolean[]`，再只将参数 descriptor 从 `([ZIZ)V` 改为 `([BIZ)V`；`bastore` 与 Code 原样保留。
runner 在 `-Xverify:all` 下检查 false/true 分别写入 0/1。它保留 boolean 操作数证据，不授权
把 boolean 到 byte 的赋值猜成普通 Java cast。

`NarrowArrayStoreUnknownElement.java` 是“恢复时 B/Z 元素类型未知”的合法 JVM 候选：
`unknown()V` 的 Code 使用 null verifier type 执行 `bastore`，descriptor 没有 B/Z 信息；
runner 在 `java -Xverify:all` 下观察到 `NullPointerException`。历史 frozen CLI
`948a6f9…` 曾将局部恢复为 `Object` 并生成 javac 不接受的 `local0[0] = 1`。中间实现 CLI
`982ae78…` 已在 BCI 5 输出拒绝 marker 且生成源码可编译，但拒绝路径是 no-op，运行结果仍
与原类不同。最终 CLI 的来源和完整类结果以
`openspec/changes/recover-narrow-array-stores/verification-fixture.md` 的最终复放记录为准。

含数组表达式和下标副作用的 B/C/S recovered 完整源码端到端对照由独立的
`tests/fixtures/p3-narrow-array-store-order-e2e/` 覆盖。不要将本目录的 source-only
求值顺序控制或两个已证明类型的合法 B/Z 变体冒充该项验收；详细结果见
`openspec/changes/recover-narrow-array-stores/verification-order.md`。
