# EM-18 construction-element baseline freeze

任务 1.1 的只读冻结核查。未修改产品、测试或 fixture 输入；未运行 Cargo、Git、rustfmt 或 fixture target，也未运行生成程序。规划任务仍保持未勾选。

## Frozen authorities and fingerprints

前一类型切片的验收记录是 `openspec/changes/recover-heterogeneous-array-init/verification-root.md`。fixture-v3 证据清单为 `results/candidate-v1-fixture-v3/manifest.json`，SHA-256 `d3dd331861d79d32cc64b2fd2e2c3677606868d54767e157fa99e5db7f508b87`；root 核验为 `results/candidate-v1-fixture-v3-root-verification.json`，SHA-256 `96087151f77d0dfcde9f4503c0479f686eb250e99b5fc3437ab1498346c193cb`（检查 246 项 hash，0 issues）。fixture 清单冻结了完整 factory/direct 两组及编译器腿、隔离编译参数、逐类源码输出和原程序精确运行流。旧报告中的源代码行号不作为证据。

可持久回查的旧 EM-18 基线归档为 `openspec/changes/recover-heterogeneous-array-init/evidence/baseline-20261009.tar.gz`，SHA-256 `317fff979cd7c53cbcf44b09b4553e13d578eb44abdfdff977f8a7e34a7808d9`（540,815 bytes，1,225 regular files）。tar 内主清单成员 `jarde-em18-baseline-20261009/manifest.json` 的 SHA-256 为 `16ca3cd8a08bce34b32d9f7a4bca6e6b44efb2abd94a23cf260b8f8388cc9705`；例如 `jarde-em18-baseline-20261009/cases/string-builder-charsequence/javac8/logs/string-builder-charsequence_javac8_javap-Main.stdout` 的 SHA-256 为 `93f3fc4fd520b0954e2c92324f5adbee8ef6551fc30064fcae5a78d35eaa9a2c`，对应报告成员 `jarde-em18-baseline-20261009/cases/string-builder-charsequence/javac8/jarde-reports/00-Main.json` 的 SHA-256 为 `565868d12ff88b88b10cc4c406bb5943f365649db1f5a24bf2e15f9aa19e4d0a`。早期临时路径 `/private/tmp/jarde-em18-baseline-20261009` 的原始输入、javap、CLI JSON、源码、编译日志和原始流均可由这个归档及主清单成员长期复核；归档元数据位于 `results/baseline-archive.json`。

参考版 JADX 的永久归档为 `openspec/changes/recover-heterogeneous-array-init/evidence/jadx-fixture-v3-reference.tar.gz`，SHA-256 `7d828c23041fddbb9b895bd45c0683e6b3c06c17dde016ea6f3da24fcb6124fc`（78,261 bytes，213 regular files）。归档元数据 `results/jadx-fixture-v3-archive.json` 的 SHA-256 是 `66763c42392cfcc65a25bbece3ab3c01915861ea76b4a5654c1fd7ce1c9cbee6`；root 核验 `results/jadx-fixture-v3-root-verification.json` 的 SHA-256 是 `353a225a9894a6af2e812f8b1928957f70a87948160eb4e50ed63e4ecea6cf9c`。八条完整 generated-source/compiler/runtime 腿是 factory/direct × javac8/javac23 × default/`--rename-flags none`。只有 `none` profile 的四条腿双流匹配；default 的四条腿虽编译并运行，但默认包重命名改变 `getClass().getName()`，原始 stdout 不同。root 记录 `full_semantic_accepted=4`、`syntax_observations=72`；72 项只是语法观察数，不是语义验收数。

冻结 CLI 是 `/private/tmp/jarde-em18-candidate-v1-cli`，SHA-256 `0ebf4e6189c02d1d84072c9aeacd380303e71e6885f95408e054e8f843154f79`（也记录于 `results/candidate-cli-v1.json`）。fixture-v3 清单保存其确切 `class-source` 调用和 JSON 输出。候选 CLI 构建来源的已登记源码指纹为 `build.rs` `3a1eb15d8563c074dab8303fbfb70a09e4e9bc0bc1ea54facb03f6a85d26a15f`、`report.rs` `88f88541154ea4881f3dd0545b71660046e6ae0a3746f9aae1876cfe9188a3a8`、`facade.rs` `f00a75f147542e4663b10fafff8daec28275dcd07996d8c01436c41ba145a16b`。本次冻结时当前工作区源码指纹为：`crates/jarde-java/src/build.rs` `3a1eb15d8563c074dab8303fbfb70a09e4e9bc0bc1ea54facb03f6a85d26a15f`、`crates/jarde-java/src/init.rs` `1af8b71c8bd134094e3e1e4b7d327cbc881723378e544d76c9c636ebf9e15e8b`、`crates/jarde-java/src/report.rs` `88f88541154ea4881f3dd0545b71660046e6ae0a3746f9aae1876cfe9188a3a8`、`src/facade.rs` `f00a75f147542e4663b10fafff8daec28275dcd07996d8c01436c41ba145a16b`、`crates/jarde-jvm/src/ssa.rs` `03e01dcda42e6022f217b4238184fc1b5d45a63b05811f7692b39d7fefa2c90a`、`crates/jarde-jvm/src/method_ir.rs` `cb4fd073be36ae09081e897f1a37292ee1b3821d7257c79238fabdf1fbe1c286`。这些当前源码 hash 不代表冻结 CLI 的构建来源。

Direct fixture 的输入 JAR SHA-256：javac8 `78edae441cc1114e96c34feee027f862967adcf7bf606427e408bfbc0caf3788`；javac23 `edfce48db43b09de0df8ded0d3b29dfe51d73e2714b8d2e6bdfc301614a8bf79`。原始输入源码以 `openspec/changes/recover-heterogeneous-array-init/evidence/heterogeneous-array-initializers-v3-pathfix/direct/` 为 canonical 路径；`v3-initial/direct/` 的六份源码逐字节相同。其 SHA-256：`Base.java` `d32b225a5e129346a2bef89edfc9ab7388336af1d65c1c9f6a4286572c4d2df2`；`DerivedA.java` `d33f1ed9b05f59e5d5e077c34c2e056df286e2b63962ddac5c51be13ed9f9977`；`DerivedB.java` `62ea0c550b93e2e50cfa97e079ef7036445c669929d6e0b485aa6bceb0b4c47c`；`LocalInterface.java` `425f56e2492e46e3f32206b6e338f5faebd5fadb6dba2391a4048164f73cc621`；`Mid.java` `c6f88e2194880bedd5fff9bf2ef1757af27b7dd6b48efc239cf3ed16cfc1b770`；direct `Main.java` `e4e9189086f9417c523c4db0cadce69739c66f9e6c297ccc4a7914768f29a017`。注意：v1 中误标为输入源码的那组值实际上来自 `candidate-v1-fixture-v3/manifest.json` 的 `reports[*].source_sha256`，身份是 Jarde 生成源码；其生成路径为 `results/candidate-v1-fixture-v3/fixture/direct/javac8/sources/` 与 `javac23/sources/`。完整 class bytes hash（从冻结 JAR 只读提取）如下：

| Class | javac8 | javac23 |
|---|---|---|
| Main | `b33bede03f694fe68cfc258a49dccb0ba39b8670bff4c6471d2b8acda0510677` | `787dd681668d9d2e1c3ecb59208a1e77a164d736ee7fb4a9c59aff752f59f726` |
| Base | `bfe74c8011daf92fe9c7580e1a25ccbb2eb9a6eca688b0a35a29159f12c54654` | `42659d89c3f93095d5af59b536602b083430543804d5cf36668379dc6f327384` |
| DerivedA | `e43a63c1e90282339e8714bbd3283874553a4283928ad1d989a93b2c3cb63b30` | `553c29faf0aeb8d00a0079d8e1dcbfb1436b3a55252eb9afd3e98158b8ed3fc5` |
| DerivedB | `6fc600e1e2cef803943662e4f73fbb1c95c03a46959309b51c528d0fca0c839d` | `eb40d297ff785c0358d67ffb493956eedfbf262fefd60ac98c6693cf6fbca1a9` |
| Mid | `37adeff6467e1bac2312aed5b944f5cbbbc8dbd96839a4ec1c4bf20d6ce1916a` | `0c9baa21caae1a01cfbd8008f4e354522a8d79d71148c1fcadda5fc5b809b66f` |
| LocalInterface | `1a981d32cf0cf13302e6ead05acbd9857968236baeca4e953efd445fe7660fcb` | `cdfae937ae17747f4e217ce41d292ee54280edb88080285908846b7bf0b9d64d` |

## Direct constructor/store bytecode and boundary

针对两份冻结 direct JAR，只读执行了 `javap -c -p Main`。完整反汇编已保存于 `results/baseline-javap-direct-javac8.txt`（SHA-256 `ae496a7fa34352d5cd68e49ce81e2e801b657752a76222785010b497aea24424`）及 `results/baseline-javap-direct-javac23.txt`（SHA-256 `26fe4e5c64e66942e93fe9da460f9b1de44be63fd5221c6b2639e4666dbe11e8`）；两编译器的相关方法 BCI 一致。下表每组依次为分配 `new` / 复制 `dup` / 匹配的 `<init>` / 消费它的 `aastore`。表中的每个构造器在冻结 CLI 报告里只被该 paired store 读取；数组分配、复制、索引和 store scaffolding 是另一组事实。原始 JSON 输出在 `results/candidate-v1-fixture-v3/logs/fixture_direct_javac{8,23}_render-Main.stdout`，报告中的 sole reader BCI 与下表 store 一致。

| 方法 / 构造对象 | `new / dup / <init> / aastore` BCIs | 冻结 new-site 结果 |
|---|---:|---|
| `sequenceDirect`: `String` | `6 / 9 / 17 / 20` | `jre_new_shape`；sole reader BCI 20 是被引用的 `aastore` |
| `sequenceDirect`: `StringBuilder` | `23 / 26 / 34 / 37` | `jre_new_shape`；sole reader BCI 37 是被引用的 `aastore` |
| `collectionDirect`: `ArrayList` | `6 / 9 / 20 / 23` | `jre_new_shape`；sole reader BCI 23 是被引用的 `aastore` |
| `collectionDirect`: `HashSet` | `26 / 29 / 40 / 43` | `jre_new_shape`；sole reader BCI 43 是被引用的 `aastore` |
| `throwableDirect`: `IllegalStateException` | `6 / 9 / 17 / 20` | `jre_new_shape`；sole reader BCI 20 是被引用的 `aastore` |
| `throwableDirect`: `IllegalArgumentException` | `23 / 26 / 34 / 37` | `jre_new_shape`；sole reader BCI 37 是被引用的 `aastore` |
| `ownTwoHopDirect`: `DerivedA` | `6 / 9 / 14 / 17` | `jre_new_shape`；sole reader BCI 17 是被引用的 `aastore` |
| `ownTwoHopDirect`: `DerivedB` | `20 / 23 / 28 / 31` | `jre_new_shape`；sole reader BCI 31 是被引用的 `aastore` |
| `ownInterfaceDirect`: `DerivedA` | `6 / 9 / 14 / 17` | `jre_new_shape`；sole reader BCI 17 是被引用的 `aastore` |
| `ownInterfaceDirect`: `DerivedB` | `20 / 23 / 28 / 31` | `jre_new_shape`；sole reader BCI 31 是被引用的 `aastore` |
| `ownGridDirect`: 内层 `DerivedA` | `12 / 15 / 20 / 23` | `jre_new_shape`；sole reader BCI 23 是内层数组 `aastore`；外层在 BCI 24 消费 row |
| `ownGridDirect`: 内层 `DerivedB` | `33 / 36 / 41 / 44` | `jre_new_shape`；sole reader BCI 44 是内层数组 `aastore`；外层在 BCI 45 消费 row |

多维 `numberGridDirect` 和 `collectionGridDirect` 把数组作为元素构造，不是 inline `new Class(...)` 的 reference element；其中通过调用产生的数组元素不能作为构造器组合的直接证据。direct family 完整保留 12 个方法，以及 two-hop/base、interface 和嵌套数组形状。

六个 `boxedDirect` 元素是独立负边界。五个 primitive conversion 位于 `dup` 与构造器调用之间，得到 `jre_new_interleaved_effect`：Byte `new 7 / dup 10 / i2b 15 / <init> 16 / store 19`；Short `22 / 25 / i2s 30 / <init> 31 / 34`；Long `51 / 54 / i2l 59 / <init> 60 / 63`；Float `66 / 69 / i2f 74 / <init> 75 / 78`；Double `81 / 84 / i2d 90 / <init> 91 / 94`。Integer 为 `37 / 40 / <init> 45 / store 48`，得到 `jre_new_shape`，因为唯一 reader 是被引用的 BCI 48 store。不要把这些结果归因为 Number 兼容性：primitive conversions 是构造表达式区间内独立的 effect，组件兼容判定尚未发生。

### What the frozen evidence proves

对 sequence、collection、throwable、custom base/interface families，输入字节码展示 fresh 且有序的数组 store，Java 类型关系合法；但相关构造先被结构规则 `jre_new_shape` 拒绝：`new@1` 发现它的唯一实例 reader 是配对 `aastore`，而 `renders_its_reads` 不把 array store 视为可呈现的构造器 consumer。数组 candidate 因而没有已证明的元素表达式。这里不是赋值兼容拒绝，也不是 Java 源码非法。direct fixture 的完整 Jarde Main 源码隔离 `javac` 失败（exit 1，javac8 stderr SHA `2f513571e213e4abaaf286ea27b3a84b4c1074e17582072e1a14337b28c0c6fc`；javac23 stderr SHA `7c692439e00be9a6fdd13dc9f45ba34600e2d89d1dc636d0ebb85b1e1cf7adef`），因此没有运行。两条原程序完整腿均 exit 0，stdout SHA `fd3a20051507b79c6537ccc6fcc24a40fb1bb10e7fda4ea01ae06a08fdc0a71b`，空 stderr SHA `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`。JADX `none` profile 的完整源码两腿均匹配原始双流；default profile 不匹配。原始日志的 root 审计及归档路径见本报告前文。

早期 EM-18 patrol 的 main 还包含外围问题，必须与 direct constructor/store refusal 分拆：`string-builder-charsequence` 的 concat chain 在 BCI 25 因 BCI 33 的 `arraylength` 交错而拒绝；另一个独立元素构造在 BCI 11 因其唯一 `aastore` reader 是 BCI 20 而拒绝。`arraylist-hashset-collection` 的外围 concat 在 BCI 28 因 BCI 36 的 `arraylength` 而拒绝；对象构造 BCIs 6 和 16 则各因唯一 store readers 13、23 被拒。`exceptions-throwable` 的外围 concat 在 BCI 32 因 BCI 40 的 `arraylength` 而拒绝；异常构造 BCIs 6 和 18 则各因唯一 store readers 15、27 被拒。`System.out` field write 省略也是另一问题。旧 patrol 主清单 `/private/tmp/jarde-em18-baseline-20261009/manifest.json` SHA `16ca3cd8a08bce34b32d9f7a4bca6e6b44efb2abd94a23cf260b8f8388cc9705` 已作为 tar member 保存；root audit `openspec/changes/recover-heterogeneous-array-init/results/em18-baseline-root-audit-v2.json` SHA `987f5b93bc7f37b50909ffd473e658c018fce553c2a26a132282879b03247eab`。旧案例、逐腿 stdout/stderr fingerprint 汇总在 `results/em18-baseline-failure-analysis.md`（SHA `7ed82b7ecbf5282078cc39c2bf7ecd3ed2ebd0f94459b77bb94a917542258b04`）；旧 patrol 与新版完整 fixture-v3 是两份不同对照，不能合并分母。

### SSA identity evidence limit

冻结 CLI 的 class-source 记录显示分析阶段 `raw_facts → raw_cfg → legacy_normalization → canonical_cfg → frame → ssa`。保存的 JSON/source map 给出操作 BCI 和拒绝信息，但不序列化数字 `ValueId` 或逐条 `SsaInstruction.reads()`。因此本冻结记录准确的 `new/dup/<init>/aastore` BCI 及 CLI 报告的 sole-reader BCI，不声称 CLI 记录已给出 paired store 的 stored `ValueId`。

源码中 `is_the_instance` 的既有调用用于验证构造器 receiver 确实来自该 allocation，以及嵌套构造结果确实是外层构造器参数；它没有在当前 proof 中验证 initializer candidate 的 paired `aastore` operand 是同一个 stored `ValueId`。另有 `outside_readers` 遍历现有 SSA instruction 的 reads，通过 `SsaTable::value(read).def()` 把 reader 归到构造的 allocation/copy/constructor BCI 集合，得到唯一 reader 指令；`renders_its_reads` 刻意不接受 array store。`ValueId` 是单个 `SsaTable` 内的索引，不是跨运行持久 ID。后续组合必须在同一个 SSA table 中把配对 store 的真实 operand `ValueId` 与已验证的构造实例关联；BCI 邻接或 CLI sole-reader 记录都不能替代这项 paired stored-value 验证。

本次冻结的边界是：构造器与配对 store 的结构关系先于赋值兼容失败，因此 task 1.1 建立了合法组合目标，但没有理由改变类型 predicate。main 外围 concat/arraylength/field 问题与五个 wrapper conversion 继续作为不同控制项处理。
