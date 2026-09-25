# 2.1 局部证明验收（2026-09-24）

`stringswitch::prove` 在同一方法的 Code、SSA 和 canonical CFG 上为 javac 两级字符串分派建立私有证书；该模块目前没有被区域构造消费，Java 输出保持两级整数 switch。证书限定 selector 单次保存及生产者、`String.hashCode()`、每个 bucket 的常量 `equals` 和唯一 discriminator 写入、最终整数 key 映射。它按 UTF-16 单元计算 hash，要求所有待折叠指令均被认领、hash/selector/保存值的 SSA 用途完整且无其它入口，拒绝额外效应、重复标签与错误 bucket。第二级未有写入来源的整数 key 仅在与 default 共目标时记为不可达空洞。当前常量解码使用 `lossy`，所以含 U+FFFD 的候选保守拒绝，以免孤立代理项被误写。

Root 对证明审阅后要求补齐精确 use 数、selector 初始化前缀和最终 switch 外部入口、无损字面量门槛及 selector 生产者来源；实现已相应收紧。定向 5 项单测经 root 重跑 5/5，通过冻结的 635 B 正例、碰撞、Unicode、default 空洞，并拒绝额外 hash 消费、桶内副作用、错误 bucket、重复标签、跨桶或最终分派额外前驱及 U+FFFD。负例测试在调用证明前断言 Code/CFG/SSA 均存在，避免把分析失败当成拒绝。

Root 另按测试的字节修改重建跨桶前驱、最终 switch 外部前驱和 U+FFFD 字面量三份反例，分别用 Java 8 `VerifyLoad` 在 `java -Xverify:all` 下加载；三份均 exit 0 并输出 `loaded`。因此这些拒绝不是以无效 class 逃避证明义务。

验收命令：`CARGO_TARGET_DIR=/private/tmp/jarde-string-switch-target cargo test --locked --offline -p jarde-java --lib stringswitch::tests` 为 5/5；同一 target 的完整 `jarde-java --lib` 为 135/135。`cargo fmt --all --check`、`git diff --check` 和 `openspec validate recover-string-switch --strict` 通过。2.2–3.2 仍未开始：证书尚无跨两个 Region 的原子认领、字符串 case 输出和完整类三方执行验收，不能把 2.1 视为 String switch 源码恢复完成。
