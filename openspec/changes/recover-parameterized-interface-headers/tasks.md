## 1. 取证与基线

- [ ] 1.1 重放固定 BR 家族（SHA 核对）：读 `src/class_source.rs:6349`（`parameterized_superclass`）/6354（`direct_parent_candidate`，注意其硬编码 `java/lang/String` 是既有片的刻意窄边界，**本片不得顺手放宽**）/6360（总门）与 `crates/jarde-reader/src/signature.rs:134` 的 `parse_class_signature`（`interfaces` 已解析、`class_references()` 已收集），确认 `ClassSignatureErasureProof`（signature.rs:222）是否已含 interfaces 擦除项；记录 `BR$Impl`（`implements Comparable` 裸）与 `BR$StrBox`（`extends BR$Box`）基线类头文本。
- [ ] 1.2 冻结至少四个变体/负例：单接口参数化（`implements Comparable<Impl>`，正例）、多接口部分可证（`implements A<X>, B`）、接口不可解析（保留裸类型 + 桥可见）、arity/擦除不符（拒绝）；各自 `java -Xverify:all` 通过并记录实现前后类头文本。

## 2. 类头接口投影

- [ ] 2.1 复用既有类头投影通道把边界从父类扩到 `interfaces` 列表（design 决策 1）；`BR$Impl` 类头呈现 `implements java.lang.Comparable<BR$Impl>`，整类 `javac --release 8` 通过、`-Xverify:all` 运行与 orig.out 一致（`0`，含经接口引用调用 `compareTo`）。
- [ ] 2.2 消隐前置不变量落地（决策 2，与 `recover-bridge-admission-gates` 的共享契约）：类头未带类型实参时拒绝桥投影、保持桥可见；负例钉死"契约丢失 + 桥隐藏"的双重损失不出现。
- [ ] 2.3 负例保留裸类型与物理来源；既有父类参数化投影、成员声明参数化（`recover-ordinary-parameterized-signatures` 9/9）与裸类型回退路径 diff 逐字不变；预算/取消原子性不变。

## 3. 回归与验收

- [ ] 3.1 `cargo test --workspace --tests --locked --no-fail-fast` 全绿（含 direct-parameterized-superclass、ordinary-parameterized-signatures、bridge 家族、nested-headers 全部既有测试）、fmt、CI 完整 `-A` clippy、`openspec validate --all --strict`、diff check；磁盘纪律：每轮构建前 `df -h /`，低于 12Gi 先 `cargo clean`，报告前必 clean。
- [ ] 3.2 BR 家族与变体三方对照：原 class/固定 JADX（dev）/Jarde 重编 `java -Xverify:all` 逐路径一致；记录输出 SHA；corpus 双腿扫描（差异应仅类头与桥家族）。
- [ ] 3.3 root 独立复核类头投影判据、消隐前置不变量与三方行为，更新 EM 账本（泛型接口/类头域）与巡查记录。
