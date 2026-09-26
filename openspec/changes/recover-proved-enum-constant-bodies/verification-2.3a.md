# 2.3a 构造器逐边证明验收

2026-09-26，root 审查 `src/facade.rs` 的同次 Code 与物理子类 Code 路径，并独立执行 `cargo test --locked --target-dir /tmp/jarde-enum-body-root-acceptance-target -p jarde --lib enum_constant_body_relation_tests`：6/6 通过。`cargo fmt --all -- --check` 与 `git diff --check` 通过。

主类私有 `(String,int)` 构造器只能按位置向 `java/lang/Enum.<init>` 转发；唯一 synthetic 访问构造器只能按位置向该私有构造器转发，不读 marker；子类构造器只能按位置传入原 name/ordinal 和 `aconst_null`。三条边均要求完整无 handler 的 Code、精确调用 owner/name/descriptor、连续 BCI 和无额外指令。普通 `Mixed` 常量的 `<clinit>` 直接调用仍由 2.2c 精确前缀验证；`Plain` 的直接物理构造器由同一 Code 检查器验证。`-g` 与 `-g:none` 下，ordinal 改写、非 null marker、额外效果、错误调用目标，以及 `Mixed`/`Plain` 直接路径改写均拒绝。

结果仅保存私有 `constructor_chain`、`constructor_bridge`、`direct_constant_bcis` 待证事实；2.3b/2.3c 未闭合，类源码仍不内联专属体。
