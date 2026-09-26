# 4.1 枚举常量体总验收

2026-09-26，root 校验冻结证据 `manifest.sha256` 的全部 79 项；独立运行 `CARGO_TARGET_DIR=/tmp/jarde-enum-root-target cargo test -p jarde --lib --locked`，75/75 通过。定向行为测试以冻结的 `Op`/`Mixed` Java 8 源和 JADX 1.5.6 源，分别在 `-g`、`-g:none` 下编译，并将 Jarde 当前同次类报告重编；三者用 `java -Xverify:all` 比较 `values` 拷贝、实例身份、name/ordinal、方法结果、运行时类、声明类及 `valueOf` 异常，四组均一致。`Plain` 作为普通常量控制没有被误加常量体，其零参数完整源码仍在本变更的范围外。库测试还覆盖桥转发、缺抽象实现、子类额外字段/效果、错误 owner/属性/使用点、预算与取消等拒绝控制。

投影依赖主类构造 BCI、同次选定物理子类、独占使用、完整构造链和子类方法 Code 的整组证明；`$VALUES`、synthetic 标志和 `$1` 名称均不是单独的隐藏条件。主类 JSON 与子类独立请求保持物理身份和 Code；投影仅改类源码。

`cargo fmt --all -- --check`、`git diff --check`、`openspec validate recover-proved-enum-constant-bodies --strict` 通过。`cargo clippy -p jarde --all-targets -- -D warnings` 被未改动的 `jarde-java` 20 项旧 lint 阻断；无 `-D warnings` 的可归因扫描另暴露仓库既有 `bulk_recovery_handover`/`bulk_recovery_workers` 测试缺少 `BulkFaults`/`BulkProbe` 接口而不能完成全目标检查。本分支新增测试中的一处 `descriptor` 冗余字段写法已改正；其余 `src/class_source.rs`、`src/facade.rs`、`src/enum_constants.rs` 告警在分支起点已有，应作为单独 lint/测试配置债务处理，不混入本语法改动。验收 target 在提交后清理。
