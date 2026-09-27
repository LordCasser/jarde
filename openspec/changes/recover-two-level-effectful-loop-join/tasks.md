## 1. 冻结二层分支基线

- [x] 1.1 用 `two-level-if-baseline/replay.py` 固定输入/class SHA、JADX revision/算法哈希与实际 BCI，root 独立复放原/JADX 四行 `-1:0 / -1:0 / 3:0 / 8:1` 和当前 Jarde 外层拒绝/未覆盖块。

## 2. 有界三来源 Region 与值证明

- [x] 2.1 仅为已证明的内层直接 `if` 臂延伸现有 `if_arm` 上下文，保留外层边界；新正例 Rust 测试核循环头、两出口、兄弟空臂及内外 join 的唯一归属。
- [x] 2.2 在原带效果双出口证书内证明空臂、效果臂和直接 break 的三输入局部 φ，内层 join 不归循环、尾段单次续接至外层边界；用正例来源/效果次数断言和额外 consumer 负例验收。
- [x] 2.3 若 Region 闭合后 Builder 仍因三来源局部作用域拒绝，只在已证结构范围内修正声明呈现；以完整方法无 `@bytecode`、source map 所需 BCI 和既有局部安全负例验证，不放宽通用门。

  Region 闭合后 Builder 直接呈现完整方法，故本项无需修改 `build.rs`。

## 3. 负例及原子性

- [x] 3.1 编译同层级的额外循环入口、异向出口、绕过内层 join、join 第四正常入边、效果双调用和异常边负例；`java -Xverify:all` 确认可加载，Rust 测试逐个断言安全拒绝及物理 BCI 保留。
- [x] 3.2 在证明末段预算耗尽与预先取消时断言停止、空源码和空来源图；复放已受证顶层效果循环与单层外部 `if` 循环 Rust 测试，不引入回归。

## 4. 完整类集成验收

- [x] 4.1 `two-level-if-baseline/replay.py --require-jarde` 的原/JADX/Jarde 完整 Java 8 类各自重编和 `java -Xverify:all` 四行一致；固定 `TestNotIndexedLoop` 继续红，不把首片记作 CF-08 整体追平。
- [x] 4.2 通过 `cargo test -p jarde-java --tests --locked`、`cargo fmt --all -- --check`、`cargo check --workspace --locked`、`git diff --check` 和 `openspec validate recover-two-level-effectful-loop-join --strict`；清理专用 Cargo target，root 独立审阅并记录验收。
